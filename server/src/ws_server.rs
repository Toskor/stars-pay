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
use tokio::{
    net::TcpListener,
    sync::broadcast,
    time::{sleep, timeout, Duration},
};

use crate::{app_state, json};



//just for test, remove later
#[derive(Debug, Serialize, Deserialize)]
struct EchoResponse {
    bot_id: String,
    message: String,
}

async fn handle_client(
    fut: upgrade::UpgradeFut,
    event_tx: broadcast::Sender<json::WSDonationEvent>,
) -> Result<(), WebSocketError> {
    println!("start client");
    let mut ws = fastwebsockets::FragmentCollector::new(fut.await?);
    let mut event_rx = event_tx.subscribe();
    let mut tmp_buf = vec![];
    let mut last_ping_time = std::time::Instant::now();

    // First message should be bot username like "@StarDonationServiceBot"
    let frame = ws.read_frame().await?;
    let bot_id = if let OpCode::Text = frame.opcode {
        let bot_id = String::from_utf8(frame.payload.to_vec()).unwrap();
        app_state::get_bot_id_from_username(&bot_id)
    } else {
        println!("Expected bot_id as first message");
        return Ok(());
    };

    println!("Client connected with bot_id: {}", bot_id);

    loop {
        tokio::select! {
            Ok(frame) = ws.read_frame() => {
                match frame.opcode {
                    OpCode::Close => {
                        println!("OpCode::Close received");
                        break;
                    }
                    OpCode::Ping => {
                        println!("handle client ping");
                        let pong = Frame::pong(frame.payload);
                        let _res = ws.write_frame(pong).await;
                    }
                    OpCode::Pong => {
                        last_ping_time = std::time::Instant::now();
                    }
                    OpCode::Text => {
                        let text = String::from_utf8(frame.payload.to_vec()).unwrap();

                        // Echo back with bot_id
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
            Ok(donation) = event_rx.recv() => {
                // Only send messages to matching bot_id
                if donation.bot_id == bot_id {
                    tmp_buf.clear();
                    serde_json::to_writer(&mut tmp_buf, &donation).unwrap();
                    let payload = Payload::Borrowed(&tmp_buf);
                    let frame = Frame::text(payload);
                    let _res = ws.write_frame(frame).await;
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

    println!("end client");
    Ok(())
}

async fn server_upgrade(
    mut req: Request<Incoming>,
    event_tx: broadcast::Sender<json::WSDonationEvent>,
) -> Result<Response<Empty<Bytes>>, WebSocketError> {
    let (response, fut) = upgrade::upgrade(&mut req)?;

    tokio::spawn(async move {
        if let Err(e) = handle_client(fut, event_tx).await {
            eprintln!("Error in websocket connection: {}", e);
        }
    });

    Ok(response)
}

pub async fn start(event_tx: broadcast::Sender<json::WSDonationEvent>, addr: &str) -> Option<u16> {
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("start server with addr! {}", listener.local_addr().unwrap());
    let port = listener.local_addr().ok().map(|a| a.port());

    tokio::spawn({
        async move {
            while let Ok((stream, _)) = listener.accept().await {
                println!("ws server listener accept");

                let service = service_fn(|r| {
                    let sender = event_tx.clone();
                    async move { server_upgrade(r, sender).await }
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
    use tokio::sync::broadcast;

    use crate::{json::WSDonationEvent, *};

    #[tokio::test]
    async fn base() {
        let (event_tx, _) = broadcast::channel::<WSDonationEvent>(100);
        let port = ws_server::start(event_tx, "localhost:5002").await.expect("msg");
        println!("port {}", port);
        _ = tokio::signal::ctrl_c().await;
    }
}
