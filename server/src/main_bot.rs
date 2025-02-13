use anyhow::Result;
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
