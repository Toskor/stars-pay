//! Update handling for the main control bot (StarDonationServiceBot).

use std::{str::FromStr, sync::Arc};

use anyhow::Result;
use hyper::Uri;
use serde_json::Value;

use crate::{app_state::AppState, config::Config, http, json, tg_api};

pub fn get_main_bot_id(config: &Config) -> &str {
    &config.main_bot_id
}

pub fn get_main_bot_token(config: &Config) -> &str {
    &config.main_bot_token
}

pub fn get_main_bot_owner(config: &Config) -> u64 {
    config.main_bot_owner
}

pub fn get_main_bot_admins(config: &Config) -> &[u64] {
    &config.main_bot_admins
}

pub async fn parse_update(
    update: &json::Update,
    token: &str,
    state: &Arc<AppState>,
) -> Result<Value> {
    let tg_api_url = tg_api::get_tg_api_url(token);
    match &update.data {
        json::UpdateData::Message(message) => {
            if let (Some(text), Some(chat)) = (message.text.as_deref(), message.chat.as_ref()) {
                let chat_id = chat.id;

                let inline_keyboard = serde_json::json!({
                    "inline_keyboard": [
                        [{
                            "text": "Settings",
                            "web_app": {"url": "https://advanced-oddly-herring.ngrok-free.app/stardonationservice/app"}
                        }]
                    ]
                });

                // Use the new send_message method
                tg_api::send_message(token, chat_id, text, Some(inline_keyboard)).await?;
            }
        }
        json::UpdateData::PreCheckoutQuery(pre_checkout_query) => {
            let checkout_id = &pre_checkout_query.id;
            // user who sent the payment
            // let from = &pre_checkout_query.from;
            let stars_amount = pre_checkout_query.total_amount;

            let mut error_message: Option<String> = None;

            // Try to parse bot_id from invoice_payload
            let bot_id = match pre_checkout_query
                .invoice_payload
                .split("paymentFor:")
                .nth(1)
            {
                Some(id) if !id.is_empty() => id.to_string(),
                _ => {
                    error_message = Some("Invalid invoice payload format".to_string());
                    String::new()
                }
            };

            // Try to process payment if no error yet
            if error_message.is_none() {
                if let Err(e) = state
                    .process_payment(bot_id.clone(), stars_amount as i64)
                    .await
                {
                    tracing::error!(bot_id = %bot_id, error = %e, "payment processing failed");
                    error_message = Some("Payment processing failed".to_string());
                }
            }

            // Build URI based on whether there was an error
            let uri = if let Some(error) = error_message {
                let encoded_error = error.replace(" ", "%20");
                Uri::from_str(&format!(
                    "{}answerPreCheckoutQuery?pre_checkout_query_id={}&ok=false&error_message={}",
                    tg_api_url, checkout_id, encoded_error
                ))
                .unwrap()
            } else {
                Uri::from_str(&format!(
                    "{}answerPreCheckoutQuery?pre_checkout_query_id={}&ok=true",
                    tg_api_url, checkout_id
                ))
                .unwrap()
            };

            if let Ok(_res) = http::get(&uri, None).await {
                // example {"ok":true,"result":true}
                // println!("answerPreCheckoutQuery res: {}", _res.to_str().unwrap());
            }
        }
    }
    Ok(Value::Null)
}
