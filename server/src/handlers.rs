//! Axum HTTP handlers. Submodules group routes by concern (`bot`, `webhook`,
//! `layer`, `auth` extractors). Every handler returns `AppResult<_>` so error
//! → HTTP mapping lives in one place (`crate::error`).

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use fastwebsockets::upgrade;
use hyper::{header, StatusCode};
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::{
    app_state::get_bot_id_from_username,
    error::{AppError, AppResult},
    json,
    ws_server::handle_client,
    AppState,
};

mod auth;
pub mod bot;
mod layer;
pub mod webhook;

pub use layer::{get_bot_ws_token, make_test_donation, refresh_layer_token, update_goal_config};

pub async fn ws_handler(
    ws: upgrade::IncomingUpgrade,
    Path(bot_username): Path<String>,
    Query(params): Query<json::WSConnectionParams>,
    State(state): State<Arc<AppState>>,
) -> AppResult<Response> {
    let bot_id = get_bot_id_from_username(&bot_username);
    let ws_token = params.ws_token;

    let matched = state
        .with_record(&bot_id, |bot| bot.ws_token == ws_token)
        .await?;

    if !matched {
        tracing::warn!(bot_id = %bot_id, "ws token mismatch");
        return Err(AppError::BadRequest("ws token mismatch".into()));
    }

    let (response, fut) = ws.upgrade().map_err(|e| {
        tracing::warn!(error = %e, "failed to upgrade websocket");
        AppError::Internal("failed to upgrade websocket".into())
    })?;

    tokio::task::spawn(async move {
        if let Err(e) = handle_client(fut, state.clone(), bot_id).await {
            tracing::warn!(error = %e, "ws connection error");
        }
    });

    Ok(response.into_response())
}

pub async fn sound_handler(Path(sound_name): Path<String>) -> AppResult<Response> {
    let sound_path = format!("server/src/sounds/{}", sound_name);

    let file = File::open(sound_path).await.map_err(|e| {
        tracing::warn!(error = %e, "failed to open sound file");
        AppError::NotFound("sound file not found".into())
    })?;

    let stream = ReaderStream::new(file);
    let body = axum::body::Body::from_stream(stream);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "audio/mpeg")
        .body(body)
        .map_err(|e| {
            tracing::error!(error = %e, "failed to build sound response");
            AppError::Internal("failed to build sound response".into())
        })
}
