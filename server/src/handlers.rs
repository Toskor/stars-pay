use anyhow::Result;

use axum::{
    extract::{Json, Path, Query, Request, State},
    http::HeaderValue,
    response::{Html, IntoResponse},
};
use hmac::{Hmac, Mac};
use hyper::{header, HeaderMap, StatusCode, Uri};
use integrations::http;
use serde_json::Value;
use sha2::Sha256;
use std::{collections::HashMap, str::FromStr, sync::Arc};
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use tower::{Service, ServiceExt};
use tower_http::services::{ServeDir, ServeFile};

use crate::{
    api, json,
    main_bot::{self, MAIN_BOT_ID},
    AppState, HTML_MINI_APP,
};

// pub async fn user_check(
//     headers: HeaderMap,
//     Path(bot_id): Path<String>,
//     State(state): State<Arc<AppState>>,
// ) -> impl IntoResponse {
//     let role = state
//         .with_record(&bot_id, |bot| {
//             if let Some(user) = check_hash_in_headers(&headers, &bot.token) {
//                 let role = if bot.admins.contains(&user.id) {
//                     "admin"
//                 } else {
//                     "user"
//                 };
//                 return Some(role);
//             }
//             None
//         })
//         .await;

//     match role {
//         Ok(Some(role)) => (StatusCode::OK, Json(serde_json::json!({"role": role}))),
//         Ok(None) => (
//             StatusCode::BAD_REQUEST,
//             Json(serde_json::json!({"error": "security check failure"})),
//         ),
//         Err(e) => (
//             StatusCode::BAD_REQUEST,
//             Json(serde_json::json!({"error": e.to_string()})),
//         ),
//     }
// }

pub async fn update_config(
    headers: HeaderMap,
    Path(bot_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<json::UpdateConfigQueryParam>,
) -> impl IntoResponse {
    println!("update_config: {}", payload.app_config);

    let security_check = state
        .with_record(&bot_id, |bot| {
            if let Some(user) = check_hash_in_headers(&headers, &bot.token) {
                let role = if bot.admins.contains(&user.id) {
                    "admin"
                } else {
                    "user"
                };
                return Some(role);
            }
            None
        })
        .await;

    match security_check {
        Ok(Some(role)) => {
            if role != "admin" {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": "Inappropriate user role"})),
                );
            }

            let upd_res = state.update_bot_config(bot_id, payload.app_config).await;
            match upd_res {
                Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))),
                Err(err) => (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": err.to_string()})),
                ),
            }
        }
        Ok(None) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "security check failure"})),
        ),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": err.to_string()})),
        ),
    }
}

// #[axum::debug_handler]
pub async fn create_invoice(
    headers: HeaderMap,
    Path(bot_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<json::CreateInvoiceQueryParam>,
) -> impl IntoResponse {
    let security_check = state
        .with_record(&bot_id, |bot| {
            if let Some(user) = check_hash_in_headers(&headers, &bot.token) {
                let role = if bot.admins.contains(&user.id) {
                    "admin"
                } else {
                    "user"
                };
                return Some((format!("https://api.telegram.org/bot{}/", bot.token), role));
            }
            None
        })
        .await;

    match security_check {
        Ok(Some((tg_api_url, role))) => {
            if role != "admin" {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": "Inappropriate user role"})),
                );
            }

            match api::create_invoice_link(&tg_api_url, &payload).await {
                Ok(url) => {
                    return (
                        StatusCode::OK,
                        Json(serde_json::json!({"invoice_url": url})),
                    )
                }
                Err(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": e.to_string()})),
                ),
            }
        }
        Ok(None) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "security check failure"})),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

pub async fn mini_app(Path(bot_id): Path<String>) -> impl IntoResponse {
    //todo add to app_config error version
    //todo need return error page (mb if payment is expired return special page)

    let path = format!("server/src/mini_app_sources/{}.html", bot_id);
    let file = match File::open(&path).await {
        Ok(file) => Ok(file),
        Err(_) => File::open("server/src/mini_app_sources/404.html").await,
    };

    match file {
        Ok(file) => {
            let stream = ReaderStream::new(file);
            let body = axum::body::Body::from_stream(stream);

            let headers = [(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            )];

            Ok((headers, body))
        }
        Err(err) => return Err((StatusCode::NOT_FOUND, format!("Not found: {}", err))),
    }
}

pub async fn controlled_bots(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let security_check = state
        .with_record(MAIN_BOT_ID, |bot| {
            if let Some(user) = check_hash_in_headers(&headers, &bot.token) {
                return Some(user.id);
            }
            None
        })
        .await;

    match security_check {
        Ok(Some(user_id)) => {
            //todo bad unwrap
            let controlled_bots = state.get_controlled_bots(user_id).await.unwrap();

            let main_page_props = json::MainBotMainPageProps {
                bots: vec![],
                has_suspended_bots: false,
            };

            let json_value = serde_json::to_value(&main_page_props).unwrap();
            (StatusCode::OK, Json(json_value))
        }
        Ok(None) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "security check failure"})),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

pub async fn add_bot(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<json::AddBotQueryParam>,
) -> impl IntoResponse {
    let security_check = state
        .with_record(MAIN_BOT_ID, |bot| {
            if let Some(user) = check_hash_in_headers(&headers, &bot.token) {
                return Some(user.id);
            }
            None
        })
        .await;

    match security_check {
        Ok(Some(user_id)) => {
            let res = state.add_bot(&payload.bot_token, user_id).await;
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
            Json(serde_json::json!({"error": "security check failure"})),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

pub async fn add_bot_admin(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<json::AddBotAdminQueryParam>,
) -> impl IntoResponse {
    let security_check = state
        .with_record(MAIN_BOT_ID, |bot| {
            if let Some(user) = check_hash_in_headers(&headers, &bot.token) {
                return Some(user.id);
            }
            None
        })
        .await;

    match security_check {
        Ok(Some(user_id)) => {
            let res = state
                .add_bot_admin(user_id, &payload.bot_id, payload.admin_id)
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
            Json(serde_json::json!({"error": "security check failure"})),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

pub async fn remove_bot_admin(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<json::RemoveBotAdminQueryParam>,
) -> impl IntoResponse {
    let security_check = state
        .with_record(MAIN_BOT_ID, |bot| {
            if let Some(user) = check_hash_in_headers(&headers, &bot.token) {
                return Some(user.id);
            }
            None
        })
        .await;

    match security_check {
        Ok(Some(user_id)) => {
            let res = state
                .remove_bot_admin(user_id, &payload.bot_id, payload.admin_id)
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
            Json(serde_json::json!({"error": "security check failure"})),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

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
            if !check_secret_token(&bot.secret_token, &headers) {
                return (false, "".to_string());
            }

            let tg_api_url = format!("https://api.telegram.org/bot{}/", bot.token);
            (true, tg_api_url)
        })
        .await;

    let mut error: anyhow::Error = anyhow::anyhow!("Some error");

    if let Ok((security_check_result, tg_api_url)) = res {
        //safety check
        if !security_check_result {
            println!("Failed security check");
            return (StatusCode::UNAUTHORIZED, Json(Value::Null));
        }

        //todo redo
        if bot_id == MAIN_BOT_ID {
            println!("Main bot webhook");
            match main_bot::parse_update(&payload, &tg_api_url).await {
                Ok(json) => return (StatusCode::OK, Json(json)),
                Err(e) => error = e,
            }
        } else {
            match parse_update(&payload, &tg_api_url).await {
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

pub async fn parse_update(update: &json::Update, tg_api_url: &str) -> Result<Value> {
    match &update.data {
        json::UpdateData::PreCheckoutQuery(pre_checkout_query) => {
            let checkout_id = &pre_checkout_query.id;

            let uri = Uri::from_str(&format!(
                "{}answerPreCheckoutQuery?pre_checkout_query_id={}&ok=true",
                tg_api_url, checkout_id
            ))
            .unwrap();

            if let Ok(res) = http::get(&uri, None).await {
                println!("answerPreCheckoutQuery res: {}", res.to_str().unwrap());
                //here send my donation event via websocket
            }
        }
        json::UpdateData::Message(message) => {
            if let (Some(text), Some(chat)) = (message.text.as_deref(), message.chat.as_ref()) {
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
        _ => {}
    }

    Ok(Value::Null)
}

fn check_secret_token(secret_token: &str, headers: &HeaderMap) -> bool {
    if let Some(header_token) = headers.get("X-Telegram-Bot-Api-Secret-Token") {
        // println!("header_token: {:?} secret_token: {:?}", header_token, secret_token);
        if header_token == secret_token {
            return true;
        }
    }

    false
}

fn check_hash_in_headers(headers: &HeaderMap, token: &str) -> Option<json::WebAppUser> {
    if let Some(hash) = headers.get("X-Telegram-InitData") {
        return check_hash(hash.to_str().unwrap(), token);
    }

    None
}

//https://core.telegram.org/bots/webapps#validating-data-received-via-the-mini-app
fn check_hash(init_data: &str, token: &str) -> Option<json::WebAppUser> {
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

pub async fn handler_print_1() -> impl IntoResponse {
    println!("handler_print_1");
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}

pub async fn handler_print_2() -> impl IntoResponse {
    println!("handler_print_2");
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
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
        println!("tt: {:?}", user.unwrap().id);
    }
}
