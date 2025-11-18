
use axum::{
    extract::{Json, Path, Query, State},
    response::{IntoResponse, Response},
};
use fastwebsockets::upgrade;
use hyper::{header, StatusCode};
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::{
    app_state::get_bot_id_from_username,
    json, ws_server::handle_client,
    AppState,
};

mod auth;
pub mod bot;
mod layer;
pub mod webhook;

pub use layer::{get_bot_ws_token, make_test_donation, refresh_layer_token, update_goal_config};

// #[axum::debug_handler]
pub async fn ws_handler(
    ws: upgrade::IncomingUpgrade,
    Path(bot_username): Path<String>,
    Query(params): Query<json::WSConnectionParams>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let bot_id = get_bot_id_from_username(&bot_username);
    let ws_token = params.ws_token;

    let security_check = state
        .with_record(&bot_id, |bot| {
            if bot.ws_token == ws_token {
                Some(())
            } else {
                None
            }
        })
        .await;

    match security_check {
        Ok(Some(())) => match ws.upgrade() {
            Ok((response, fut)) => {
                tokio::task::spawn(async move {
                    if let Err(e) = handle_client(fut, state.clone(), bot_id).await {
                        eprintln!("Error in websocket connection: {}", e);
                    }
                });

                response.into_response()
            }
            Err(e) => {
                eprintln!("Failed to upgrade websocket: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Failed to upgrade websocket connection"})),
                )
                    .into_response()
            }
        },
        Ok(None) => {
            println!("Ws token mismatch");
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Ws token mismatch"})),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

//handle test cdn
pub async fn sound_handler(Path(sound_name): Path<String>) -> impl IntoResponse {
    let sound_path = format!("server/src/sounds/{}", sound_name);

    match File::open(sound_path).await {
        Ok(file) => {
            let stream = ReaderStream::new(file);
            let body = axum::body::Body::from_stream(stream);

            match Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "audio/mpeg")
                .body(body)
            {
                Ok(response) => response.into_response(),
                Err(e) => {
                    eprintln!("Failed to build sound response: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": "Failed to build response"})),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open sound file: {}", e);
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Sound file not found"})),
            )
                .into_response()
        }
    }
}
