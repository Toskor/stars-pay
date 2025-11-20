use axum::{
    extract::{Json, State},
    response::IntoResponse,
};
use hyper::StatusCode;
use std::sync::Arc;

use crate::{AppState, app_state::UserRole, handlers::auth::BotAccessWithPayload, json};

use super::auth::{AuthenticatedUser, BotOwnerOrAdmin};

/// Make a test donation event for testing WebSocket functionality
pub async fn make_test_donation(
    State(state): State<Arc<AppState>>,
    BotAccessWithPayload { access, payload }: BotAccessWithPayload<json::MakeTestDonationQueryParam>,
) -> impl IntoResponse {
    if access.role != UserRole::Owner && access.role != UserRole::Admin {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Only owner or admin can make test donation"})),
        );
    }

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

/// Update goal configuration for a bot
pub async fn update_goal_config(
    BotOwnerOrAdmin { bot, .. }: BotOwnerOrAdmin,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<json::GoalPropsQueryParam>,
) -> impl IntoResponse {
    println!("start update_goal_config");
    let Ok(ws_token) = state.get_bot_ws_token(bot.id.clone()).await else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "security check failure while getting bot ws token"})),
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

/// Refresh layer token for a bot
pub async fn refresh_layer_token(
    State(state): State<Arc<AppState>>,
    BotAccessWithPayload { access, payload }: BotAccessWithPayload<json::RefreshLayerTokenQueryParams>,
) -> impl IntoResponse {
    if access.role != UserRole::Owner && access.role != UserRole::Admin {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Only owner or admin can refresh layer token"})),
        );
    }

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

/// Get WebSocket token for a bot
pub async fn get_bot_ws_token(
    BotOwnerOrAdmin { bot, .. }: BotOwnerOrAdmin,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({"status": "success", "ws_token": bot.ws_token})),
    )
}
