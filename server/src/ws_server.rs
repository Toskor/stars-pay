pub use anyhow::{anyhow, Result};

use fastwebsockets::{upgrade, Frame, OpCode, Payload, WebSocketError};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};

use crate::{app_state::AppState, json};

fn get_next_client_id() -> usize {
    static NEXT_CLIENT_ID: AtomicUsize = AtomicUsize::new(1);
    NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed)
}

//just for test, removed later
#[derive(Debug, Serialize, Deserialize)]
struct EchoResponse {
    bot_id: String,
    message: String,
}

pub async fn handle_client(
    fut: upgrade::UpgradeFut,
    app_state: std::sync::Arc<crate::app_state::AppState>,
    bot_id: String,
) -> Result<(), WebSocketError> {
    let cid = get_next_client_id();

    let (room_tx, room_members_count) = match app_state.get_or_create_room(&bot_id).await {
        Ok(result) => result,
        Err(e) => {
            println!("Error joining room for bot_id {}: {}", bot_id, e);

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
    println!(
        "Client {} connected with bot_id: {} (room size: {})",
        cid,
        bot_id,
        room_members_count + 1
    );

    let mut ws = fastwebsockets::FragmentCollector::new(fut.await?);
    let mut tmp_buf = vec![];
    let mut last_handshake_time = tokio::time::Instant::now();

    loop {
        tokio::select! {
            Ok(frame) = ws.read_frame() => {
                // println!("OpCode: {:?}", frame.opcode);
                match frame.opcode {
                    OpCode::Close => {
                        println!("OpCode::Close received from client {}", cid);
                        break;
                    }
                    OpCode::Ping => {
                        // println!("OpCode::Ping received from client {}", cid);
                        last_handshake_time = tokio::time::Instant::now();
                        let pong = Frame::pong(frame.payload);
                        let _res = ws.write_frame(pong).await;
                    }
                    OpCode::Pong => {
                        // println!("OpCode::Pong received from client {}", cid);
                        last_handshake_time = tokio::time::Instant::now();
                    }
                    OpCode::Text => {
                        last_handshake_time = tokio::time::Instant::now();
                        let text = match String::from_utf8(frame.payload.to_vec()) {
                            Ok(text) => text,
                            Err(e) => {
                                eprintln!("Failed to decode UTF-8 text frame: {}", e);
                                continue;
                            }
                        };

                        let response = EchoResponse {
                            bot_id: bot_id.clone(),
                            message: text.clone(),
                        };
                        tmp_buf.clear();
                        if let Err(e) = serde_json::to_writer(&mut tmp_buf, &response) {
                            eprintln!("Failed to serialize echo response: {}", e);
                            continue;
                        }
                        let payload = Payload::Borrowed(&tmp_buf);
                        let frame = Frame::text(payload);
                        let _res = ws.write_frame(frame).await;
                    }
                    _ => {}
                }
            }
            Ok(msg) = room_rx.recv() => {
                match msg {
                    json::RoomMessage::CloseConnection(target_cid) => {
                        if target_cid == cid {
                            println!("Received close signal for client {}", target_cid);
                            let close_frame = Frame::close(1000, b"Server initiated shutdown");
                            let _ = ws.write_frame(close_frame).await;
                            break;
                        }
                    }
                    json::RoomMessage::Text(text_data) => {
                        let frame = Frame::text(Payload::Owned(text_data));
                        if let Err(e) = ws.write_frame(frame).await {
                            println!("Error sending message to client {} in room {}: {}", cid, bot_id, e);
                            break;
                        }
                    }
                    json::RoomMessage::CloseRoom(bot_id) => {
                        println!("Received close signal for room_id: {}", bot_id);
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
                        println!("Error sending ping: {}", e);
                        break;
                    }
                } else {
                    println!("No pong received within 4 seconds, closing connection");
                    break;
                }
            }
        }
    }

    app_state.remove_client_from_room(&bot_id, cid).await;
    println!("Client {} disconnected from bot_id: {}", cid, bot_id);
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
            println!("Removed empty room for bot_id: {}", bot_id);
        } else {
            println!(
                "Client {} left room {}, {} clients remaining",
                cid, bot_id, left_in_room
            );
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
