//! WebSocket overlay server: streamers' OBS browser sources subscribe to a
//! per-bot room and receive donation events in real time.

pub use anyhow::Result;

use fastwebsockets::{upgrade, Frame, OpCode, Payload, WebSocketError};
use std::sync::atomic::{AtomicUsize, Ordering};
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

    let membership = match app_state.rooms.join(&bot_id).await {
        Ok(membership) => membership,
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

    let mut room_rx = membership.receiver;
    tracing::info!(cid, bot_id = %bot_id, room_size = membership.members, "ws client connected");

    let mut ws = fastwebsockets::FragmentCollector::new(fut.await?);
    let mut last_handshake_time = tokio::time::Instant::now();

    // Each branch must be cancellation-safe: when one fires, the others are
    // dropped mid-poll and re-polled next iteration.
    //   - broadcast::recv  — safe: a dropped recv future loses no buffered item.
    //   - sleep            — safe: a fresh timer is created each iteration.
    //   - ws.read_frame    — assumed safe (fastwebsockets buffers internally);
    //     we never split a frame across iterations because we await it fully.
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
                    json::RoomMessage::Binary(data) => {
                        let frame = Frame::binary(Payload::Owned(data));
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

    let removed = app_state.rooms.leave(&bot_id).await;
    tracing::info!(cid, bot_id = %bot_id, room_removed = removed, "ws client disconnected");
    Ok(())
}

impl AppState {
    /// Encode a domain event as protobuf and broadcast it to a bot's room.
    pub async fn send_event_to_room_members(
        &self,
        room_id: &str,
        event: json::WSEvent,
    ) -> Result<()> {
        let data = crate::proto::encode(&crate::proto::ServerMessage::from(&event));
        self.rooms
            .send(room_id, json::RoomMessage::Binary(data))
            .await
    }
}
