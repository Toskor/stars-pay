use anyhow::Result;
use axum::{
    extract::{Json, Path, State},
    response::IntoResponse,
};
use hyper::{HeaderMap, StatusCode, Uri};
use serde_json::Value;
use std::{str::FromStr, sync::Arc};

use super::auth;
use crate::{app_state::AppState, http, json, main_bot, tg_api};

// #[axum::debug_handler]
pub async fn webhook_handler(
    headers: HeaderMap,
    Path(bot_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<json::Update>,
) -> impl IntoResponse {
    //todo bad to_string()
    let res = state
        .with_record(&bot_id, |bot| {
            //safety check
            if !auth::check_secret_token(&bot.secret_token, &headers) {
                return (false, "".to_string());
            }

            let token = bot.token.to_string();
            (true, token)
        })
        .await;

    let mut error: anyhow::Error = anyhow::anyhow!("Some error");

    if let Ok((security_check_result, token)) = res {
        //safety check
        if !security_check_result {
            println!("Failed security check for {}", bot_id);
            return (StatusCode::UNAUTHORIZED, Json(Value::Null));
        }

        //todo redo
        if bot_id == state.config.main_bot_id {
            println!("Main bot webhook");
            match main_bot::parse_update(&payload, &token, &state).await {
                Ok(json) => return (StatusCode::OK, Json(json)),
                Err(e) => error = e,
            }
        } else {
            match parse_update(&payload, &token, &state, bot_id).await {
                Ok(json) => return (StatusCode::OK, Json(json)),
                Err(e) => error = e,
            }
        }
    }

    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": error.to_string()})),
    )
}

pub async fn parse_update(
    update: &json::Update,
    token: &str,
    state: &Arc<AppState>,
    bot_id: String,
) -> Result<Value> {
    let tg_api_url = tg_api::get_tg_api_url(token);
    // println!("parse_update: {:?}", update);
    match &update.data {
        json::UpdateData::PreCheckoutQuery(pre_checkout_query) => {
            let checkout_id = &pre_checkout_query.id;

            let uri = Uri::from_str(&format!(
                "{}answerPreCheckoutQuery?pre_checkout_query_id={}&ok=true",
                tg_api_url, checkout_id
            ))
            .unwrap();

            if let Ok(_res) = http::get(&uri, None).await {
                // example {"ok":true,"result":true}
                // println!("answerPreCheckoutQuery res: {}", res.to_str().unwrap());

                let ws_donation_event = json::WSEvent::Success(json::WSEventSuccess {
                    ok: true,
                    data: json::WSEventData::Donation {
                        from: pre_checkout_query.from.username.clone(),
                        total_amount: pre_checkout_query.total_amount,
                        invoice_payload: pre_checkout_query.invoice_payload.clone(),
                        message: "some message".to_string(),
                    },
                });

                let state_c = state.clone();
                let bot_id_c = bot_id.clone();
                let stars = pre_checkout_query.total_amount;

                let (res_send, res_increase) = tokio::join!(
                    state_c.send_event_to_room_members(&bot_id_c, ws_donation_event,),
                    //todo can throw db error
                    state_c.increase_stars_debt_for(bot_id_c.clone(), stars)
                );

                res_increase?;
                res_send?;
            }
        }
        json::UpdateData::Message(message) => {
            let is_command = message.entities.as_ref().map_or(false, |entities| {
                entities
                    .iter()
                    .any(|entity| entity.entity_type == "bot_command")
            });

            let ans_text = if is_command {
                parse_bot_command(&message, state, &bot_id).await?
            } else {
                parse_message(&message, state, &bot_id).await?
            };

            if let Some(chat) = message.chat.as_ref() {
                let chat_id = chat.id;

                let inline_keyboard = serde_json::json!({
                    "inline_keyboard": [
                        [{
                            "text": "Donate",
                            "web_app": {"url": "https://advanced-oddly-herring.ngrok-free.app/app"}
                        }]
                    ]
                });

                let ans = serde_json::json!({
                    // "method": "sendMessage", // field for webhook ans
                    "text": ans_text,
                    "chat_id": chat_id,
                    "reply_markup": inline_keyboard,
                });

                let uri = Uri::from_str(&format!("{}sendMessage", tg_api_url)).unwrap();
                let headers: HeaderMap = HeaderMap::from_iter([(
                    hyper::header::CONTENT_TYPE,
                    hyper::header::HeaderValue::from_static("application/json"),
                )]);

                let _res = http::post(&uri, Some(&headers), ans.to_string()).await?;
            }
        } // _ => {}
    }

    Ok(Value::Null)
}

async fn parse_bot_command(
    message: &json::Message,
    state: &Arc<AppState>,
    bot_id: &str,
) -> Result<String> {
    //find first command that has type bot_command
    let command = message
        .entities
        .as_ref()
        .and_then(|entities| {
            entities
                .iter()
                .find(|entity| entity.entity_type == "bot_command")
        })
        .map(|entity| entity);

    if let Some(command) = command {
        let Some(text) = message.text.as_ref() else {
            return Err(anyhow::anyhow!("Command found but message has no text"));
        };
        let command_text = text
            .chars()
            .skip(command.offset as usize)
            .take(command.length as usize)
            .collect::<String>();

        match command_text.as_str() {
            "/start" => Ok("start".to_string()),
            "/help" => Ok("help".to_string()),
            "/donate" => Ok("donate".to_string()),
            "/layer" => match state.generate_layer_url(bot_id).await {
                Ok(layer_url) => Ok(layer_url),
                Err(e) => Err(anyhow::anyhow!("Failed to generate layer URL: {}", e)),
            },
            _ => Err(anyhow::anyhow!("Unknown command")),
        }
    } else {
        Err(anyhow::anyhow!("No command found"))
    }
}

async fn parse_message(
    message: &json::Message,
    _state: &Arc<AppState>,
    _bot_id: &str,
) -> Result<String> {
    let text = message
        .text
        .clone()
        .ok_or(anyhow::anyhow!("no text in message"))?;
    Ok(text)
}
