use axum::extract::{Json, State};
use serde_json::Value;
use std::sync::Arc;

use crate::{
    app_state::UserRole,
    error::{AppError, AppResult},
    handlers::auth::BotAccessWithPayload,
    json, AppState,
};

use super::auth::BotOwnerOrAdmin;

/// Send a fake donation event for overlay testing.
pub async fn make_test_donation(
    State(state): State<Arc<AppState>>,
    BotAccessWithPayload { access, payload }: BotAccessWithPayload<
        json::MakeTestDonationQueryParam,
    >,
) -> AppResult<Json<Value>> {
    if access.role != UserRole::Owner && access.role != UserRole::Admin {
        return Err(AppError::Forbidden(
            "only owner or admin can make test donation".into(),
        ));
    }

    let ws_donation_event = json::WSEvent::Success(Box::new(json::WSEventSuccess {
        ok: true,
        data: json::WSEventData::Donation {
            from: "Test User".to_string(),
            total_amount: payload.amount,
            invoice_payload: payload.media_source.clone(),
            message: "Its just test donation".to_string(),
        },
    }));

    state
        .send_event_to_room_members(&payload.target_bot_id, ws_donation_event)
        .await?;

    Ok(Json(serde_json::json!({"status": "ok"})))
}

pub async fn update_goal_config(
    BotOwnerOrAdmin { bot, .. }: BotOwnerOrAdmin,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<json::GoalPropsQueryParam>,
) -> AppResult<Json<Value>> {
    tracing::debug!(bot_id = %bot.id, "update_goal_config");
    let ws_token = state.get_bot_ws_token(bot.id.clone()).await.map_err(|_| {
        AppError::BadRequest("security check failure while getting bot ws token".into())
    })?;

    state
        .update_goal_config(payload.target_bot_id, &ws_token, payload.goal_config)
        .await?;

    Ok(Json(serde_json::json!({"status": "ok"})))
}

pub async fn refresh_layer_token(
    State(state): State<Arc<AppState>>,
    BotAccessWithPayload { access, payload }: BotAccessWithPayload<
        json::RefreshLayerTokenQueryParams,
    >,
) -> AppResult<Json<Value>> {
    if access.role != UserRole::Owner && access.role != UserRole::Admin {
        return Err(AppError::Forbidden(
            "only owner or admin can refresh layer token".into(),
        ));
    }

    state
        .refresh_layer_token(payload.target_bot_id.clone())
        .await?;
    Ok(Json(serde_json::json!({"status": "success"})))
}

pub async fn get_bot_ws_token(
    BotOwnerOrAdmin { bot, .. }: BotOwnerOrAdmin,
    State(_state): State<Arc<AppState>>,
) -> AppResult<Json<Value>> {
    Ok(Json(
        serde_json::json!({"status": "success", "ws_token": bot.ws_token}),
    ))
}
