use axum::{
    extract::{Json, Path, State},
    response::IntoResponse,
};
use hyper::{HeaderMap, StatusCode};
use std::sync::Arc;

use crate::{json, AppState};

use super::auth;

/// Make a test donation event for testing WebSocket functionality
pub async fn make_test_donation(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<json::MakeTestDonationQueryParam>,
) -> impl IntoResponse {
    let security_check = state
        .with_record(&state.config.main_bot_id, |bot| {
            if let Some(_user) = auth::check_hash_in_headers(&headers, &bot.token) {
                return Some(());
            }
            None
        })
        .await;

    match security_check {
        Ok(Some(_)) => {
            let ws_donation_event = json::WSEvent::Success(json::WSEventSuccess {
                ok: true,
                data: json::WSEventData::Donation {
                    from: "Test User".to_string(),
                    total_amount: payload.amount,
                    invoice_payload: payload.media_source.clone(),
                    message: "Its just test donation".to_string(),
                },
            });

            match state
                .send_event_to_room_members(&payload.target_bot_id, ws_donation_event)
                .await
            {
                Ok(_) => (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))),
                Err(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": e.to_string()})),
                ),
            }
        }
        Ok(None) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "security check failure while making test donation"})),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

/// Update goal configuration for a bot
pub async fn update_goal_config(
    headers: HeaderMap,
    Path(bot_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<json::GoalPropsQueryParam>,
) -> impl IntoResponse {
    println!("start update_goal_config");
    let security_check = state
        .with_record(&bot_id, |bot| {
            if let Some(user) = auth::check_hash_in_headers(&headers, &bot.token) {
                if user.id == bot.owner || bot.admins.contains(&user.id) {
                    return Some(());
                }
            }
            None
        })
        .await;

    match security_check {
        Ok(Some(_)) => {
            let Ok(ws_token) = state.get_bot_ws_token(bot_id).await else {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(
                        serde_json::json!({"error": "security check failure while getting bot ws token"}),
                    ),
                );
            };
            match state
                .update_goal_config(payload.target_bot_id, &ws_token, payload.goal_config)
                .await
            {
                Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))),
                Err(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": e.to_string()})),
                ),
            }
        }
        Ok(None) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "security check failure while getting goal config"})),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

/// Refresh layer token for a bot
pub async fn refresh_layer_token(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<json::RefreshLayerTokenQueryParams>,
) -> impl IntoResponse {
    let security_check = state
        .with_record(&state.config.main_bot_id, |bot| {
            if let Some(user) = auth::check_hash_in_headers(&headers, &bot.token) {
                if user.id == bot.owner || bot.admins.contains(&user.id) {
                    return Some(());
                }
            }
            None
        })
        .await;

    match security_check {
        Ok(Some(_)) => {
            let res = state
                .refresh_layer_token(payload.target_bot_id.clone())
                .await;

            match res {
                Ok(()) => (
                    StatusCode::OK,
                    Json(serde_json::json!({"status": "success"})),
                ),
                Err(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": e.to_string()})),
                ),
            }
        }
        Ok(None) => (
            StatusCode::BAD_REQUEST,
            Json(
                serde_json::json!({"error": "security check failure while refreshing layer token"}),
            ),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

/// Get WebSocket token for a bot
pub async fn get_bot_ws_token(
    headers: HeaderMap,
    Path(bot_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let security_check = state
        .with_record(&bot_id, |bot| {
            if let Some(user) = auth::check_hash_in_headers(&headers, &bot.token) {
                if user.id == bot.owner || bot.admins.contains(&user.id) {
                    return Some(bot.ws_token.clone());
                }
            }
            None
        })
        .await;

    match security_check {
        Ok(Some(ws_token)) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "success", "ws_token": ws_token})),
        ),
        Ok(None) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "security check failure while getting bot ws token"})),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}
