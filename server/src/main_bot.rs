use std::{str::FromStr, sync::Arc};

use anyhow::Result;
use hyper::{HeaderMap, Uri};
use serde_json::Value;

use crate::{api, app_state::AppState, http, json};

pub const MAIN_BOT_ID: &str = "stardonationservice";
pub const MAIN_BOT_TOKEN: &str = "***REMOVED***";
pub const MAIN_BOT_SECRET_TOKEN: &str = "secret";
// Torsor now
pub const MAIN_BOT_OWNER: u64 = 348135868;
pub const MAIN_BOT_ADMINS: [u64; 1] = [348135868];

pub async fn parse_update(
    update: &json::Update,
    token: &str,
    state: &Arc<AppState>,
) -> Result<Value> {
    let tg_api_url = api::get_tg_api_url(token);
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
                let ans = serde_json::json!({
                    // "method": "sendMessage", // field for webhook ans
                    "text": text,
                    "chat_id": chat_id,
                    "reply_markup": inline_keyboard,
                });

                let uri = Uri::from_str(&format!("{}sendMessage", tg_api_url)).unwrap();
                let headers: HeaderMap = HeaderMap::from_iter([(
                    hyper::header::CONTENT_TYPE,
                    hyper::header::HeaderValue::from_static("application/json"),
                )]);

                let res = http::post(&uri, Some(&headers), ans.to_string()).await?;
                // if res.status == StatusCode::OK {
                //     println!("sendMessage res: {}", res.to_str().unwrap());
                // } else {
                //     println!("sendMessage error: {}", res.to_str().unwrap());
                // }
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
                if let Err(e) = state.process_payment(bot_id.clone(), stars_amount as i64).await {
                    println!("Payment processing for bot {} failed: {}", bot_id, e);
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

            if let Ok(res) = http::get(&uri, None).await {
                // example {"ok":true,"result":true}
                // println!("answerPreCheckoutQuery res: {}", res.to_str().unwrap());
            }
        }
        _ => {
            println!("Unsuported update by MainBot");
            return Err(anyhow::anyhow!("Unsuported update by MainBot"));
        }
    }
    Ok(Value::Null)
}

pub async fn get_full_user(user_id: &str, tg_api_url: &str) -> Result<()> {
    let url = format!(
        "{}/bot{}/getChat?chat_id={}",
        tg_api_url, MAIN_BOT_TOKEN, user_id
    );
    let uri = Uri::from_str(&url).unwrap();

    let body = serde_json::json!({
        "chat_id": user_id
    });

    // let res = integrations::http::post(&uri, None, body.to_string()).await?;
    let res = http::get(&uri, None).await?;

    println!("{}", res.to_str().unwrap());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query() {
        get_full_user("348135868", "https://api.telegram.org")
            .await
            .unwrap();
    }
}
