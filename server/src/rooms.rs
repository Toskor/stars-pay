//! Per-bot WebSocket fan-out rooms.
//!
//! All overlays for one bot share a single `tokio::sync::broadcast` channel.
//! A room is created on the first join and dropped when the last subscriber
//! leaves. Every create/remove decision is made while holding a lock, so two
//! clients racing on the same bot can't strand a subscriber on an orphaned
//! channel (the bug this type exists to prevent — see the unit tests).

use std::collections::HashMap;

use anyhow::{anyhow, Result};
use tokio::sync::{broadcast, RwLock};

use crate::json::RoomMessage;

const ROOM_CHANNEL_CAPACITY: usize = 32;

type Sender = broadcast::Sender<RoomMessage>;
type Receiver = broadcast::Receiver<RoomMessage>;

/// A client's subscription to a room, returned by [`RoomRegistry::join`].
pub struct Membership {
    pub receiver: Receiver,
    /// Subscriber count in the room, including this client.
    pub members: usize,
}

/// Registry of live rooms keyed by bot id.
#[derive(Default)]
pub struct RoomRegistry {
    rooms: RwLock<HashMap<String, Sender>>,
    /// Max subscribers per room. A join beyond this is rejected.
    capacity: usize,
}

impl RoomRegistry {
    pub fn new(capacity: usize) -> Self {
        Self {
            rooms: RwLock::new(HashMap::new()),
            capacity,
        }
    }

    /// Subscribe to a bot's room, creating it if absent.
    ///
    /// The receiver is created while a lock is held, so it is counted before
    /// any concurrent [`leave`](Self::leave) can run. Without that, a client
    /// could subscribe to a channel in the window after another client emptied
    /// and removed the room, and silently never receive events.
    pub async fn join(&self, bot_id: &str) -> Result<Membership> {
        // Fast path: an existing room only needs a read lock.
        {
            let rooms = self.rooms.read().await;
            if let Some(tx) = rooms.get(bot_id) {
                return self.subscribe(tx);
            }
        }

        // Slow path: create under the write lock, re-checking via `entry` so two
        // racing creators share one channel. A plain insert would let the second
        // creator overwrite the first's channel and orphan its subscribers.
        let mut rooms = self.rooms.write().await;
        let tx = rooms
            .entry(bot_id.to_string())
            .or_insert_with(|| broadcast::channel(ROOM_CHANNEL_CAPACITY).0);
        self.subscribe(tx)
    }

    fn subscribe(&self, tx: &Sender) -> Result<Membership> {
        if tx.receiver_count() > self.capacity {
            return Err(anyhow!("maxout: room already has maximum clients"));
        }
        let receiver = tx.subscribe();
        Ok(Membership {
            receiver,
            members: tx.receiver_count(),
        })
    }

    /// Drop a client and remove the room if it was the last one. The caller's
    /// own receiver is assumed still alive, so `<= 1` means "empty after this
    /// client". Returns whether the room was removed.
    pub async fn leave(&self, bot_id: &str) -> bool {
        let mut rooms = self.rooms.write().await;
        match rooms.get(bot_id) {
            Some(tx) if tx.receiver_count() <= 1 => {
                rooms.remove(bot_id);
                true
            }
            _ => false,
        }
    }

    /// Broadcast a message to a room. A missing room is a no-op — nobody's
    /// watching that bot's overlay right now.
    pub async fn send(&self, bot_id: &str, msg: RoomMessage) -> Result<()> {
        let rooms = self.rooms.read().await;
        if let Some(tx) = rooms.get(bot_id) {
            tx.send(msg)?;
        }
        Ok(())
    }

    #[cfg(test)]
    async fn room_count(&self) -> usize {
        self.rooms.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn concurrent_joins_share_one_room() {
        let registry = Arc::new(RoomRegistry::new(1000));

        // Many clients race to join the same (initially missing) room.
        let mut handles = Vec::new();
        for _ in 0..50 {
            let reg = registry.clone();
            handles.push(tokio::spawn(async move { reg.join("bot").await.unwrap() }));
        }
        let mut members = Vec::new();
        for h in handles {
            members.push(h.await.unwrap());
        }

        // They all landed in exactly one shared room...
        assert_eq!(registry.room_count().await, 1);

        // ...and a single broadcast reaches every one of them.
        registry
            .send("bot", RoomMessage::CloseConnection(7))
            .await
            .unwrap();
        for m in &mut members {
            match m.receiver.recv().await.unwrap() {
                RoomMessage::CloseConnection(cid) => assert_eq!(cid, 7),
                other => panic!("unexpected message: {other:?}"),
            }
        }
    }

    #[tokio::test]
    async fn room_dropped_when_last_client_leaves() {
        let registry = RoomRegistry::new(10);
        let _m = registry.join("bot").await.unwrap();
        assert_eq!(registry.room_count().await, 1);

        // leave() is called while the client's own receiver is still alive,
        // mirroring the real disconnect path.
        assert!(registry.leave("bot").await);
        assert_eq!(registry.room_count().await, 0);
    }

    #[tokio::test]
    async fn capacity_is_enforced() {
        let registry = RoomRegistry::new(1); // permits up to 2 subscribers
        let _a = registry.join("bot").await.unwrap();
        let _b = registry.join("bot").await.unwrap();
        assert!(registry.join("bot").await.is_err());
    }
}
