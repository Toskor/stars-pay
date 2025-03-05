use std::str::FromStr;

use anyhow::Result;
use hyper::{HeaderMap, Uri};
use integrations::http;
use serde_json::Value;

use crate::{api, json};

pub const MAIN_BOT_ID: &str = "stardonationservice";
pub const MAIN_BOT_TOKEN: &str = "***REMOVED***";
pub const MAIN_BOT_SECRET_TOKEN: &str = "secret";
// Torsor now
pub const MAIN_BOT_OWNER: u64 = 348135868;
pub const MAIN_BOT_ADMINS: [u64; 1] = [348135868];

pub async fn parse_update(update: &json::Update, token: &str) -> Result<Value> {
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
    let res = integrations::http::get(&uri, None).await?;

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
