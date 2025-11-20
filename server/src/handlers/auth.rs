use axum::{
    async_trait,
    extract::{FromRequest, FromRequestParts, Path, Request},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use hmac::{Hmac, Mac};
use hyper::HeaderMap;
use serde_json::json;
use sha2::Sha256;
use std::{collections::HashMap, sync::Arc};

use crate::{app_state::AppState, json};

/// Check secret token from headers for webhook authentication
pub fn check_secret_token(secret_token: &str, headers: &HeaderMap) -> bool {
    if let Some(header_token) = headers.get("X-Telegram-Bot-Api-Secret-Token") {
        if header_token == secret_token {
            return true;
        } else {
            println!(
                "check_secret_token header_token: {:?} secret_token: {:?}",
                header_token, secret_token
            );
        }
    }

    false
}

/// Extract and validate Telegram Mini App init data from headers
pub fn check_hash_in_headers(headers: &HeaderMap, token: &str) -> Option<json::WebAppUser> {
    if let Some(hash) = headers.get("X-Telegram-InitData") {
        return check_hash(hash.to_str().unwrap(), token);
    }

    None
}

/// Validate Telegram Mini App init data hash
/// Reference: https://core.telegram.org/bots/webapps#validating-data-received-via-the-mini-app
pub fn check_hash(init_data: &str, token: &str) -> Option<json::WebAppUser> {
    let data: HashMap<_, _> = form_urlencoded::parse(init_data.as_bytes())
        .into_owned()
        .collect();

    let mut check_string = data
        .iter()
        .filter(|&(key, _)| key != "hash")
        .map(|(key, value)| format!("{}={}", key, value))
        .collect::<Vec<_>>();
    check_string.sort();
    let check_string = check_string.join("\n");

    let mut mac = Hmac::<Sha256>::new_from_slice(b"WebAppData").unwrap();
    mac.update(token.as_bytes());
    let secret_key = mac.finalize().into_bytes();

    let mut mac = Hmac::<Sha256>::new_from_slice(&secret_key).unwrap();
    mac.update(check_string.as_bytes());
    let signature = mac.finalize().into_bytes();

    if let Some(hash) = data.get("hash") {
        // println!("hash: {}\nsign: {}", hash, hex::encode(signature));

        if hex::encode(signature) == *hash {
            if let Some(user) = data.get("user") {
                let user: Option<json::WebAppUser> = serde_json::from_str(user).ok();
                return user;
            }
        }
    }

    None
}

/// Authenticated user from Telegram Mini App
/// Validates user authentication via main bot token
/// For paths https://domain.com/stardonationservice/*
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user: json::WebAppUser,
}

#[async_trait]
impl FromRequestParts<Arc<AppState>> for AuthenticatedUser {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let app_state = state.clone();

        let headers = parts.headers.clone();

        let user = app_state
            .with_record(&app_state.config.main_bot_id, |bot| {
                check_hash_in_headers(&headers, &bot.token)
            })
            .await
            .map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": e.to_string()})),
                )
                    .into_response()
            })?;

        match user {
            Some(user) => Ok(AuthenticatedUser { user }),
            None => Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "security check failure"})),
            )
                .into_response()),
        }
    }
}

/// Bot access information - user role and bot data
/// Validates user authentication and checks access rights to a specific bot
#[derive(Debug, Clone)]
pub struct BotAccess {
    pub user: json::WebAppUser,
    pub role: crate::app_state::UserRole,
    pub bot: crate::db::DBBot,
}

#[async_trait]
impl FromRequestParts<Arc<AppState>> for BotAccess {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let Path(bot_id) = Path::<String>::from_request_parts(parts, state)
            .await
            .map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "Failed to extract bot_id from path"})),
                )
                    .into_response()
            })?;

        validate_bot_access_from_id(&bot_id, &parts.headers, state).await
    }
}

/// Bot access that requires owner or admin role
#[derive(Debug, Clone)]
pub struct BotOwnerOrAdmin {
    pub user: json::WebAppUser,
    pub role: crate::app_state::UserRole,
    pub bot: crate::db::DBBot,
}

#[async_trait]
impl FromRequestParts<Arc<AppState>> for BotOwnerOrAdmin {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let bot_access = BotAccess::from_request_parts(parts, state).await?;

        match bot_access.role {
            crate::app_state::UserRole::Owner | crate::app_state::UserRole::Admin => {
                Ok(BotOwnerOrAdmin {
                    user: bot_access.user,
                    role: bot_access.role,
                    bot: bot_access.bot,
                })
            }
            crate::app_state::UserRole::User => Err((
                StatusCode::FORBIDDEN,
                Json(json!({"error": "Insufficient permissions. Owner or admin access required."})),
            )
                .into_response()),
        }
    }
}

/// Trait for extracting bot_id from payload types
pub trait HasBotId {
    fn bot_id(&self) -> &str;
}

// Implement HasBotId for payload types that have bot_id field
impl HasBotId for json::AddBotAdminQueryParam {
    fn bot_id(&self) -> &str {
        &self.bot_id
    }
}

impl HasBotId for json::RemoveBotAdminQueryParam {
    fn bot_id(&self) -> &str {
        &self.bot_id
    }
}

impl HasBotId for json::RemoveBotQueryParam {
    fn bot_id(&self) -> &str {
        &self.bot_id
    }
}

impl HasBotId for json::ChangeBotTokenQueryParam {
    fn bot_id(&self) -> &str {
        &self.bot_id
    }
}

impl HasBotId for json::ConfigQueryParam {
    fn bot_id(&self) -> &str {
        &self.target_bot_id
    }
}

impl HasBotId for json::UpdateConfigQueryParam {
    fn bot_id(&self) -> &str {
        &self.target_bot_id
    }
}

impl HasBotId for json::RefreshLayerTokenQueryParams {
    fn bot_id(&self) -> &str {
        &self.target_bot_id
    }
}

impl HasBotId for json::GetDebtInvoiceURLQueryParam {
    fn bot_id(&self) -> &str {
        &self.target_bot_id
    }
}

impl HasBotId for json::MakeTestDonationQueryParam {
    fn bot_id(&self) -> &str {
        &self.target_bot_id
    }
}

#[derive(Debug, Clone)]
pub struct BotAccessWithPayload<T> {
    pub access: BotAccess,
    pub payload: T,
}

#[async_trait]
impl<T> FromRequest<Arc<AppState>> for BotAccessWithPayload<T>
where
    T: for<'de> serde::Deserialize<'de> + HasBotId + Send,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &Arc<AppState>) -> Result<Self, Self::Rejection> {
        let headers = req.headers().clone();

        let Json(payload) = Json::<T>::from_request(req, state).await.map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": format!("Failed to parse request body: {}", e)})),
            )
                .into_response()
        })?;

        let bot_id = payload.bot_id().to_string();
        let access = validate_bot_access_from_id(&bot_id, &headers, state).await?;
        Ok(BotAccessWithPayload { access, payload })
    }
}

/// Helper function to validate bot access from bot_id string
pub async fn validate_bot_access_from_id(
    bot_id: &str,
    headers: &HeaderMap,
    state: &Arc<AppState>,
) -> Result<BotAccess, Response> {
    let result = state
        .with_record(bot_id, |bot| {
            if let Some(user) = check_hash_in_headers(headers, &bot.token) {
                let role = if user.id == bot.owner {
                    crate::app_state::UserRole::Owner
                } else if bot.admins.contains(&user.id) {
                    crate::app_state::UserRole::Admin
                } else {
                    crate::app_state::UserRole::User
                };
                Some((user, role, bot.clone()))
            } else {
                None
            }
        })
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        })?;

    match result {
        Some((user, role, bot)) => Ok(BotAccess { user, role, bot }),
        None => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "security check failure"})),
        )
            .into_response()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_hash() {
        //second test bot
        let init_data = "query_id=AAG8IcAUAAAAALwhwBTPhTwZ&user=%7B%22id%22%3A348135868%2C%22first_name%22%3A%22%D0%93%D1%80%D0%B8%D0%B3%D0%BE%D1%80%D0%B8%D0%B9%22%2C%22last_name%22%3A%22%D0%91%D0%BE%D1%80%D0%B8%D1%81%D0%BE%D0%B2%22%2C%22username%22%3A%22Torsor%22%2C%22language_code%22%3A%22ru%22%2C%22allows_write_to_pm%22%3Atrue%2C%22photo_url%22%3A%22https%3A%5C%2F%5C%2Ft.me%5C%2Fi%5C%2Fuserpic%5C%2F320%5C%2FwsUOF6a3vdHs4d6GxHTdD5Y7swpuTZO6dz0iWc0e8go.svg%22%7D&auth_date=1734603245&signature=vf8Crn0P3kI1ZE0HvkgzBT3XZxGGjehqpn7vgIHidwQ18GVNdkZ6RgRkRAjmoM2VNihAdBAYOyRaNYICKQPbBQ&hash=42c3594acc55ea181d8d7a62be0e79af134e1a400d681345c88f18ac52844a38";
        let token = "8090667304:AAFDIkQ7htfPHAjm2Vnzrl5JH6oELo4Y1e4";

        let user = check_hash(init_data, token);

        assert!(user.is_some());
        assert_eq!(user.unwrap().id, 348135868);
    }
}
