//! WebSocket overlay server: streamers' OBS browser sources subscribe to a
//! per-bot room and receive donation events in real time.

pub use anyhow::{anyhow, Result};

use fastwebsockets::{upgrade, Frame, OpCode, Payload, WebSocketError};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};

use crate::{app_state::AppState, json};

fn get_next_client_id() -> usize {
    static NEXT_CLIENT_ID: AtomicUsize = AtomicUsize::new(1);
    NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed)
}

/// Run the per-client WebSocket loop: ping/pong heartbeat, forward room
/// broadcasts to the socket, and clean up on disconnect.
pub async fn handle_client(
    fut: upgrade::UpgradeFut,
    app_state: std::sync::Arc<crate::app_state::AppState>,
    bot_id: String,
) -> Result<(), WebSocketError> {
    let cid = get_next_client_id();

    let (room_tx, room_members_count) = match app_state.get_or_create_room(&bot_id).await {
        Ok(result) => result,
        Err(e) => {
            tracing::warn!(bot_id = %bot_id, error = %e, "error joining room");

            let mut ws = fastwebsockets::FragmentCollector::new(fut.await?);

            let error_msg = format!(
                "{{\"ok\": false, \"error\": \"maxout: {bot_id} already has maximum clients\"}}"
            );
            let frame = Frame::text(Payload::Owned(error_msg.into_bytes()));

            let _ = ws.write_frame(frame).await;
            let _ = ws.write_frame(Frame::close(1000, &[])).await;

            return Ok(());
        }
    };

    let mut room_rx = room_tx.subscribe();
    tracing::info!(
        cid,
        bot_id = %bot_id,
        room_size = room_members_count + 1,
        "ws client connected"
    );

    let mut ws = fastwebsockets::FragmentCollector::new(fut.await?);
    let mut last_handshake_time = tokio::time::Instant::now();

    loop {
        tokio::select! {
            Ok(frame) = ws.read_frame() => {
                match frame.opcode {
                    OpCode::Close => {
                        tracing::debug!(cid, "ws close frame received");
                        break;
                    }
                    OpCode::Ping => {
                        last_handshake_time = tokio::time::Instant::now();
                        let pong = Frame::pong(frame.payload);
                        let _res = ws.write_frame(pong).await;
                    }
                    OpCode::Pong => {
                        last_handshake_time = tokio::time::Instant::now();
                    }
                    OpCode::Text => {
                        last_handshake_time = tokio::time::Instant::now();
                    }
                    _ => {}
                }
            }
            Ok(msg) = room_rx.recv() => {
                match msg {
                    json::RoomMessage::CloseConnection(target_cid) => {
                        if target_cid == cid {
                            tracing::debug!(cid = target_cid, "close signal for client");
                            let close_frame = Frame::close(1000, b"Server initiated shutdown");
                            let _ = ws.write_frame(close_frame).await;
                            break;
                        }
                    }
                    json::RoomMessage::Text(text_data) => {
                        let frame = Frame::text(Payload::Owned(text_data));
                        if let Err(e) = ws.write_frame(frame).await {
                            tracing::warn!(cid, bot_id = %bot_id, error = %e, "error writing ws frame");
                            break;
                        }
                    }
                    json::RoomMessage::CloseRoom(bot_id) => {
                        tracing::debug!(bot_id = %bot_id, "close signal for room");
                        let close_frame = Frame::close(1000, b"Server initiated shutdown");
                        let _ = ws.write_frame(close_frame).await;
                        break;
                    }
                    _ => {}
                }
            }
            _ = sleep(Duration::from_secs(4)) => {
                if last_handshake_time.elapsed() < Duration::from_secs(8) {
                    let ping = Frame::new(true, OpCode::Ping, None, Payload::Borrowed(b""));
                    if let Err(e) = ws.write_frame(ping).await {
                        tracing::warn!(cid, error = %e, "error sending ws ping");
                        break;
                    }
                } else {
                    tracing::debug!(cid, "no pong within timeout, closing");
                    break;
                }
            }
        }
    }

    app_state.remove_client_from_room(&bot_id, cid).await;
    tracing::info!(cid, bot_id = %bot_id, "ws client disconnected");
    Ok(())
}

impl AppState {
    pub async fn get_or_create_room(
        &self,
        bot_id: &str,
    ) -> Result<(broadcast::Sender<json::RoomMessage>, usize)> {
        {
            // Hashmap.get() cause RWLock.read()
            let rooms = self.rooms.read().await;
            if let Some(tx) = rooms.get(bot_id) {
                let client_count = tx.receiver_count();

                if client_count > self.config.room_capacity {
                    return Err(anyhow!("maxout: room already has maximum clients"));
                }

                return Ok((tx.clone(), client_count));
            }
        }

        let mut rooms = self.rooms.write().await;
        let (tx, _rx) = broadcast::channel(32); // Same capacity as in gist
        rooms.insert(bot_id.to_string(), tx.clone());

        Ok((tx, 0))
    }

    pub async fn remove_client_from_room(&self, bot_id: &str, cid: usize) {
        let mut should_remove_room = false;
        let left_in_room = {
            let rooms = self.rooms.read().await;
            if let Some(tx) = rooms.get(bot_id) {
                let count = tx.receiver_count();
                if count <= 1 {
                    should_remove_room = true;
                }
                count
            } else {
                0
            }
        };

        if should_remove_room {
            let mut rooms = self.rooms.write().await;
            rooms.remove(bot_id);
            tracing::debug!(bot_id = %bot_id, "removed empty room");
        } else {
            tracing::debug!(cid, bot_id = %bot_id, remaining = left_in_room, "client left room");
        }
    }

    pub async fn send_event_to_room_members(
        &self,
        room_id: &str,
        event: json::WSEvent,
    ) -> Result<()> {
        let data = serde_json::to_vec(&event)?;
        let rooms = self.rooms.read().await;
        if let Some(tx) = rooms.get(room_id) {
            tx.send(json::RoomMessage::Text(data))?;
        }

        Ok(())
    }
}
