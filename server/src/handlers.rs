use anyhow::Result;

use axum::{
    body::Body,
    extract::{Json, Path, Query, Request, State},
    http::HeaderValue,
    response::{IntoResponse, Response},
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
    api::{self, bot_numeric_id_from_token},
    json,
    main_bot::{self, MAIN_BOT_ID, MAIN_BOT_TOKEN},
    AppState, HTML_MINI_APP,
};
use crate::{app_state::ControlledBots, db::DBBot};

pub async fn config_handler(
    headers: HeaderMap,
    Path(bot_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<json::ConfigQueryParam>,
) -> impl IntoResponse {
    let security_check = state
        .with_record(&bot_id, |bot| {
            if let Some(user) = check_hash_in_headers(&headers, &bot.token) {
                if user.id == bot.owner || bot.admins.contains(&user.id) {
                    return Some(());
                }
            }
            None
        })
        .await;

    match security_check {
        Ok(Some(_)) => {
            let config = state.get_bot_config(payload.target_bot_id).await.unwrap();
            let json_config: Value = serde_json::from_str(&config).unwrap();
            (StatusCode::OK, Json(json_config))
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

pub async fn update_config(
    headers: HeaderMap,
    Path(bot_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<json::UpdateConfigQueryParam>,
) -> impl IntoResponse {
    let security_check = state
        .with_record(&bot_id, |bot| {
            if let Some(user) = check_hash_in_headers(&headers, &bot.token) {
                if user.id == bot.owner || bot.admins.contains(&user.id) {
                    return Some(());
                }
            }
            None
        })
        .await;

    match security_check {
        Ok(Some(_)) => {
            let target_bot_token = state
                .get_bot_token(payload.target_bot_id.to_string())
                .await
                .unwrap();
            let mut tma_app_config: json::TMAAppConfig =
                serde_json::from_str(&payload.app_config).unwrap();
            generate_invoice_urls(&mut tma_app_config, &target_bot_token).await;

            let upd_res = state
                .update_bot_config(
                    payload.target_bot_id,
                    serde_json::to_string(&tma_app_config).unwrap(),
                )
                .await;
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
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

async fn generate_invoice_urls(tma_app_config: &mut json::TMAAppConfig, token: &str) {
    use tokio::task::{self, JoinHandle};

    let mut tasks: Vec<(usize, JoinHandle<Result<String>>)> = Vec::new();

    for (index, button) in tma_app_config.donation_buttons.iter().enumerate() {
        if button.invoice_url.is_empty() {
            let token = token.to_string();
            let name = button.name.clone();
            let description = button.description.clone();
            let amount = button.amount;

            let task = task::spawn(async move {
                api::create_invoice_link(
                    &token,
                    &json::CreateInvoiceQueryParam {
                        title: name.clone(),
                        description,
                        payload: name,
                        amount,
                    },
                )
                .await
            });

            tasks.push((index, task));
        }
    }

    // Обрабатываем результаты задач
    for (index, task) in tasks {
        if let Ok(Ok(url)) = task.await {
            tma_app_config.donation_buttons[index].invoice_url = url;
        }
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
                return Some((bot.token.clone(), role));
            }
            None
        })
        .await;

    match security_check {
        Ok(Some((token, role))) => {
            if role != "admin" {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": "Inappropriate user role"})),
                );
            }

            match api::create_invoice_link(&token, &payload).await {
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

pub async fn fetch_user_bots(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let security_check = state
        .with_record(MAIN_BOT_ID, |bot| {
            if let Some(user) = check_hash_in_headers(&headers, &bot.token) {
                return Some(user);
            }
            None
        })
        .await;

    match security_check {
        Ok(Some(web_app_user)) => {
            //todo bad unwrap
            let controlled_bots = state.get_controlled_bots(web_app_user.id).await.unwrap();
            match convert_controlled_bots_to_json_value(controlled_bots, web_app_user).await {
                Ok(json_value) => (StatusCode::OK, Json(json_value)),
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

async fn convert_controlled_bots_to_json_value(
    controlled_bots: ControlledBots,
    web_app_user: json::WebAppUser, //user that opened mini app
) -> Result<Value> {
    //todo remove time
    use std::time::Instant;
    use tokio::task::{self, JoinHandle};

    let mut tasks: Vec<JoinHandle<Option<json::TMABotData>>> = Vec::new();

    // Process owner bots
    process_owner_bots(&controlled_bots.owner_bots, &web_app_user, &mut tasks);

    // Process admin bots
    process_admin_bots(&controlled_bots.admin_bots, &web_app_user, &mut tasks);

    let tasks_start = Instant::now();

    let mut bots = Vec::with_capacity(tasks.len());
    for task in tasks {
        match task.await {
            Ok(Some(tma_bot_data)) => {
                bots.push(tma_bot_data);
            }
            Ok(None) => {
                println!("Task returned None");
            }
            Err(e) => {
                println!("Task join error: {}", e);
            }
        }
    }

    println!("Tasks completed in: {:?}", tasks_start.elapsed());

    let main_page_props = json::MainBotMainPageProps { bots };
    let json_value = serde_json::to_value(&main_page_props)?;

    Ok(json_value)
}

fn process_owner_bots(
    owner_bots: &Vec<DBBot>,
    web_app_user: &json::WebAppUser,
    tasks: &mut Vec<tokio::task::JoinHandle<Option<json::TMABotData>>>,
) {
    use tokio::task::{self, JoinHandle};

    for bot in owner_bots {
        let bot = bot.clone();
        let owner = web_app_user.clone();
        let task: JoinHandle<Option<json::TMABotData>> = task::spawn(async move {
            let Ok(numeric_id) = api::bot_numeric_id_from_token(&bot.token) else {
                //unreachable, token format is incorrect, but token was verified before the bot was added to the db
                return None;
            };

            let bot_info_task = tokio::spawn({
                let token = bot.token.clone();
                async move { api::get_bot_info(&token).await }
            });

            // Process admin information
            let admins = process_admin_info(&bot.token, &bot.admins, None).await;

            let bot_info_first_name = match bot_info_task.await {
                Ok(Ok(bot_info)) => {
                    if bot_info.ok {
                        Some(bot_info.result.unwrap().first_name)
                    } else {
                        None
                    }
                }
                Ok(Err(e)) => {
                    //todo here need to log error
                    println!("Error fetching bot info: {} error {}", bot.id, e);
                    None
                }
                Err(e) => {
                    //task error
                    println!("bot_info task error: {}", e);
                    None
                }
            };

            Some(json::TMABotData {
                id: bot.id.clone(),
                numeric_id: bot_numeric_id_from_token(&bot.token).unwrap_or(0),
                name: bot_info_first_name.unwrap_or(bot.id),
                avatar: None,
                user_role: "owner".to_string(),
                owner: json::TMAUserData {
                    id: owner.id,
                    username: owner.username,
                    name: format!("{} {}", owner.first_name, owner.last_name),
                    avatar_url: Some(owner.photo_url),
                },
                admins,
                suspended: None,
                debt: None,
            })
        });
        tasks.push(task);
    }
}

fn process_admin_bots(
    admin_bots: &Vec<DBBot>,
    web_app_user: &json::WebAppUser,
    tasks: &mut Vec<tokio::task::JoinHandle<Option<json::TMABotData>>>,
) {
    use tokio::task::{self, JoinHandle};

    for bot in admin_bots {
        let bot = bot.clone();
        let mini_app_user = web_app_user.clone();
        let task: JoinHandle<Option<json::TMABotData>> = task::spawn(async move {
            let Ok(numeric_id) = api::bot_numeric_id_from_token(&bot.token) else {
                //unreachable, token format is incorrect, token was verified before the bot was added to the db
                return None;
            };

            let bot_info_task = tokio::spawn({
                let token = bot.token.clone();
                async move { api::get_bot_info(&token).await }
            });

            let owner_info_task = tokio::spawn({
                let token = bot.token.clone();
                let owner_id = bot.owner;
                async move { api::get_user_info(&token, owner_id).await }
            });

            // Process admin information
            let admins = process_admin_info(&bot.token, &bot.admins, Some(&mini_app_user)).await;

            let bot_info_first_name = match bot_info_task.await {
                Ok(Ok(bot_info)) => {
                    if bot_info.ok {
                        Some(bot_info.result.unwrap().first_name)
                    } else {
                        None
                    }
                }
                Ok(Err(e)) => {
                    //todo here need to log error
                    println!("Error fetching bot info: {} error {}", bot.id, e);
                    None
                }
                Err(e) => {
                    //task error
                    println!("bot_info task error: {}", e);
                    None
                }
            };

            let owner_info = match owner_info_task.await {
                Ok(Ok(owner_info)) => {
                    if owner_info.ok {
                        owner_info.result.unwrap()
                    } else {
                        return None;
                    }
                }
                Ok(Err(e)) => {
                    //todo here need to log error
                    println!("Error fetching owner info for bot: {} error {}", bot.id, e);
                    return None;
                }
                Err(e) => {
                    //task error
                    println!("owner_info task error: {}", e);
                    return None;
                }
            };

            Some(json::TMABotData {
                id: bot.id.clone(),
                numeric_id: bot_numeric_id_from_token(&bot.token).unwrap_or(0),
                name: bot_info_first_name.unwrap_or(bot.id),
                avatar: None,
                user_role: "admin".to_string(),
                owner: json::TMAUserData {
                    id: owner_info.id,
                    username: owner_info.username,
                    name: format!("{} {}", owner_info.first_name, owner_info.last_name),
                    avatar_url: None,
                },
                admins,
                suspended: None,
                debt: None,
            })
        });
        tasks.push(task);
    }
}

async fn process_admin_info(
    token: &str,
    admin_ids: &Vec<u64>,
    mini_app_user: Option<&json::WebAppUser>,
) -> Vec<json::TMAUserData> {
    use tokio::task::{self, JoinHandle};

    let mut admin_futures = Vec::new();
    for admin_id in admin_ids {
        if let Some(user) = mini_app_user {
            if *admin_id == user.id {
                continue;
            }
        }

        let token = token.to_string();
        let admin_id = *admin_id;
        let admin_info_future = task::spawn(async move {
            let admin_info = api::get_user_info(&token, admin_id).await;
            (admin_id, admin_info)
        });
        admin_futures.push(admin_info_future);
    }

    let mut admins = Vec::with_capacity(admin_ids.len());

    if let Some(user) = mini_app_user {
        if admin_ids.contains(&user.id) {
            admins.push(json::TMAUserData {
                id: user.id,
                username: user.username.clone(),
                name: format!("{} {}", user.first_name, user.last_name),
                avatar_url: Some(user.photo_url.clone()),
            });
        }
    }

    for admin_future in admin_futures {
        if let Ok((admin_id, admin_info)) = admin_future.await {
            if let Ok(admin) = admin_info {
                if admin.ok {
                    admins.push(json::TMAUserData {
                        id: admin_id,
                        name: format!(
                            "{} {}",
                            admin.result.as_ref().unwrap().first_name,
                            admin.result.as_ref().unwrap().last_name
                        ),
                        username: admin.result.unwrap().username,
                        avatar_url: None,
                    });
                }
            }
        }
    }

    admins
}

pub async fn add_bot(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<json::AddBotQueryParam>,
) -> impl IntoResponse {
    let security_check = state
        .with_record(MAIN_BOT_ID, |bot| {
            if let Some(user) = check_hash_in_headers(&headers, &bot.token) {
                return Some(user);
            }
            None
        })
        .await;

    match security_check {
        Ok(Some(user)) => {
            let res = state.add_bot(&payload.bot_token, user.id).await;
            match res {
                Ok((bot_id, bot_name)) => {
                    let bot_data = json::TMABotData {
                        id: bot_id,
                        numeric_id: bot_numeric_id_from_token(&payload.bot_token).unwrap_or(0),
                        name: bot_name,
                        avatar: None,
                        user_role: "owner".to_string(),
                        owner: json::TMAUserData {
                            id: user.id,
                            username: user.username,
                            name: format!("{} {}", user.first_name, user.last_name),
                            avatar_url: Some(user.photo_url),
                        },
                        admins: vec![],
                        suspended: None,
                        debt: None,
                    };
                    (
                        StatusCode::OK,
                        Json(serde_json::json!({"status": "success", "bot_data": bot_data})),
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
                Ok(admin_info) => (
                    StatusCode::OK,
                    Json(serde_json::json!({"status": "success", "admin_info": admin_info})),
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

pub async fn avatar_url_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Path((bot_id, user_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let security_check = state
        .with_record(&bot_id, |bot| {
            if let Some(user) = check_hash_in_headers(&headers, &bot.token) {
                return Some(bot.token.clone());
            }
            None
        })
        .await;

    let token = match security_check {
        Ok(Some(token)) => token,
        Ok(None) => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(
                    serde_json::json!({"error": "security check failure"}).to_string(),
                ))
                .unwrap();
        }
        Err(e) => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(
                    serde_json::json!({"error": e.to_string()}).to_string(),
                ))
                .unwrap();
        }
    };

    let user_id_parsed = match user_id.parse::<u64>() {
        Ok(id) => id,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(
                    serde_json::json!({"error": "Invalid user ID"}).to_string(),
                ))
                .unwrap();
        }
    };

    let avatar_url_result = api::get_avatar_url(&token, user_id_parsed).await;

    match avatar_url_result {
        Ok(Some(avatar_url)) => {
            let uri = match hyper::Uri::from_str(&avatar_url) {
                Ok(uri) => uri,
                Err(_) => {
                    return Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Body::from(
                            serde_json::json!({"error": "Invalid avatar URL format"}).to_string(),
                        ))
                        .unwrap();
                }
            };

            // Download the image
            match integrations::http::get(&uri, None).await {
                Ok(response) => {
                    if response.status != StatusCode::OK {
                        return Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Body::from(
                                serde_json::json!({"error": "Failed to download avatar image"})
                                    .to_string(),
                            ))
                            .unwrap();
                    }

                    // For Telegram avatar images, the content type is always image/jpeg
                    let content_type = "image/jpeg";
                    let image_data = response.to_bytes().to_vec();

                    return Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, content_type)
                        .body(Body::from(image_data))
                        .unwrap();
                }
                Err(e) => {
                    return Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from(format!("Failed to download image: {}", e)))
                        .unwrap();
                }
            }
        }
        Ok(None) => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(
                    serde_json::json!({"error": "No avatar found for this user"}).to_string(),
                ))
                .unwrap();
        }
        Err(e) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("Failed to download image: {}", e)))
                .unwrap();
        }
    }
}

pub async fn remove_bot(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<json::RemoveBotQueryParam>,
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
            let res = state.remove_bot(user_id, payload.bot_id).await;
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

pub async fn change_bot_token(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<json::ChangeBotTokenQueryParam>,
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
                .change_bot_token(user_id, payload.bot_id, payload.new_token)
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

            let token = bot.token.to_string();
            (true, token)
        })
        .await;

    let mut error: anyhow::Error = anyhow::anyhow!("Some error");

    if let Ok((security_check_result, token)) = res {
        //safety check
        if !security_check_result {
            println!("Failed security check");
            return (StatusCode::UNAUTHORIZED, Json(Value::Null));
        }

        //todo redo
        if bot_id == MAIN_BOT_ID {
            println!("Main bot webhook");
            match main_bot::parse_update(&payload, &token).await {
                Ok(json) => return (StatusCode::OK, Json(json)),
                Err(e) => error = e,
            }
        } else {
            match parse_update(&payload, &token).await {
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

pub async fn parse_update(update: &json::Update, token: &str) -> Result<Value> {
    let tg_api_url = api::get_tg_api_url(token);
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
