pub use anyhow::Result;

use fastwebsockets::{upgrade, Frame, OpCode, Payload, WebSocketError};
use http_body_util::Empty;
use hyper::{
    body::{Bytes, Incoming},
    server::conn::http1,
    service::service_fn,
    Request, Response,
};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::{
    net::TcpListener,
    sync::broadcast,
    time::{sleep, timeout, Duration},
};

use crate::{app_state, json};

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

            let error_msg =
                format!("{{\"error\": \"maxout: {bot_id} already has maximum clients\"}}");
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
    let mut last_ping_time = std::time::Instant::now();

    loop {
        tokio::select! {
            Ok(frame) = ws.read_frame() => {
                match frame.opcode {
                    OpCode::Close => {
                        println!("OpCode::Close received from client {}", cid);
                        break;
                    }
                    OpCode::Ping => {
                        let pong = Frame::pong(frame.payload);
                        let _res = ws.write_frame(pong).await;
                    }
                    OpCode::Pong => {
                        last_ping_time = std::time::Instant::now();
                    }
                    OpCode::Text => {
                        let text = String::from_utf8(frame.payload.to_vec()).unwrap();

                        let response = EchoResponse {
                            bot_id: bot_id.clone(),
                            message: text.clone(),
                        };
                        tmp_buf.clear();
                        serde_json::to_writer(&mut tmp_buf, &response).unwrap();
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
                if last_ping_time.elapsed() < Duration::from_secs(8) {
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

//for testing ws server alone
async fn server_upgrade(
    mut req: Request<Incoming>,
    event_tx: broadcast::Sender<(String, json::WSDonationEvent)>,
    app_state: std::sync::Arc<crate::app_state::AppState>,
    bot_id: String,
) -> Result<Response<Empty<Bytes>>, WebSocketError> {
    let (response, fut) = upgrade::upgrade(&mut req)?;

    tokio::spawn(async move {
        if let Err(e) = handle_client(fut, app_state, bot_id).await {
            eprintln!("Error in websocket connection: {}", e);
        }
    });

    return Ok(response);
}

//for testing ws server alone
async fn start(
    event_tx: broadcast::Sender<(String, json::WSDonationEvent)>,
    addr: &str,
    app_state: std::sync::Arc<crate::app_state::AppState>,
) -> Option<u16> {
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("start server with addr! {}", listener.local_addr().unwrap());
    let port = listener.local_addr().ok().map(|a| a.port());

    tokio::spawn({
        async move {
            while let Ok((stream, _)) = listener.accept().await {
                println!("ws server listener accept");

                let service = service_fn(|req: Request<Incoming>| {
                    let sender = event_tx.clone();
                    let app_state = app_state.clone();

                    // easier with axum
                    // Extract bot_id from the request query parameters
                    let bot_id = if let Some(query) = req.uri().query() {
                        // Parse the query string
                        query
                            .split('&')
                            .filter_map(|kv| {
                                let mut parts = kv.splitn(2, '=');
                                if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                                    if k == "bot_id" {
                                        return Some(v.to_string());
                                    }
                                }
                                None
                            })
                            .next()
                            .unwrap_or_else(|| {
                                println!("No bot_id found in query params, using default");
                                "123".to_string()
                            })
                    } else {
                        println!("No query parameters found, using default bot_id");
                        "123".to_string()
                    };

                    println!("WebSocket connection request with bot_id: {}", bot_id);

                    async move { server_upgrade(req, sender, app_state, bot_id).await }
                });

                let io = hyper_util::rt::TokioIo::new(stream);

                let conn_fut = http1::Builder::new()
                    .serve_connection(io, service)
                    .with_upgrades();
                if let Err(e) = conn_fut.await {
                    println!("err with conn_fut {e}");
                }
            }
        }
    });
    port
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use tokio::sync::broadcast;

    use crate::{app_state::AppState, json::WSDonationEvent, ws_server};

    #[tokio::test]
    async fn base() {
        let (event_tx, _) = broadcast::channel::<(String, WSDonationEvent)>(100);
        let app_state = Arc::new(AppState::new().await);
        let port = ws_server::start(event_tx, "localhost:5002", app_state.clone())
            .await
            .expect("msg");

        _ = tokio::signal::ctrl_c().await;

        let rooms = app_state.rooms.read().await;
        println!("Rooms after shutdown: {} rooms remaining", rooms.len());
    }
}
