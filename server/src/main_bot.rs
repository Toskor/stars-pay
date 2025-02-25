use std::str::FromStr;

use anyhow::Result;
use hyper::Uri;
use serde_json::Value;

use crate::json;

pub const MAIN_BOT_ID: &str = "stardonationservice";
pub const MAIN_BOT_TOKEN: &str = "***REMOVED***";
pub const MAIN_BOT_SECRET_TOKEN: &str = "secret";
// Torsor now
pub const MAIN_BOT_OWNER: u64 = 348135868;
pub const MAIN_BOT_ADMINS: [u64; 1] = [348135868];

pub async fn parse_update(update: &json::Update, tg_api_url: &str) -> Result<Value> {
    Ok(Value::Null)
}

pub async fn get_full_user(user_id: &str, tg_api_url: &str) -> Result<()> {
    let url = format!("{}/bot{}/getChat?chat_id={}", tg_api_url, MAIN_BOT_TOKEN, user_id);
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
        get_full_user("@torsor", "https://api.telegram.org").await.unwrap();
    }
}
