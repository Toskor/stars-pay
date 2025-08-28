use anyhow::Result;

use axum::{
    body::Body,
    extract::{Json, Multipart, Path, Query, State},
    response::{IntoResponse, Response},
};
use fastwebsockets::upgrade;
use hmac::{Hmac, Mac};
use hyper::{header, HeaderMap, StatusCode, Uri};
use serde_json::Value;
use sha2::Sha256;
use std::{collections::HashMap, str::FromStr, sync::Arc};
use tokio::{fs::File, task};
use tokio_util::io::ReaderStream;

use crate::{
    app_state::get_bot_id_from_username,
    http, json,
    main_bot::{self, MAIN_BOT_ID},
    tg_api::{self, bot_numeric_id_from_token},
    ws_server::handle_client,
    AppState, MAX_STARS_DEBT,
};
use crate::{app_state::ControlledBots, db::DBBot};

pub async fn make_test_donation(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<json::MakeTestDonationQueryParam>,
) -> impl IntoResponse {
    let security_check = state
        .with_record(MAIN_BOT_ID, |bot| {
            if let Some(_user) = check_hash_in_headers(&headers, &bot.token) {
                return Some(());
            }
            None
        })
        .await;

    match security_check {
        Ok(Some(_)) => {
            let ws_donation_event = json::WSEvent::Success(json::WSEventSuccess {
                ok: true,
                data: json::WSEventData::Donation {
                    from: "Test User".to_string(),
                    total_amount: payload.amount,
                    invoice_payload: payload.media_source.clone(),
                    message: "Its just test donation".to_string(),
                },
            });

            match state
                .send_event_to_room_members(&payload.target_bot_id, ws_donation_event)
                .await
            {
                Ok(_) => (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))),
                Err(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": e.to_string()})),
                ),
            }
        }
        Ok(None) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "security check failure while making test donation"})),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

pub async fn upload_image(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let security_check = state
        .with_record(MAIN_BOT_ID, |bot| {
            if let Some(_user) = check_hash_in_headers(&headers, &bot.token) {
                return Some(());
            }
            None
        })
        .await;

    match security_check {
        Ok(Some(_)) => {
            let mut image = Option::<Vec<u8>>::None;
            let mut image_type = Option::<String>::None;

            while let Ok(Some(field)) = multipart.next_field().await {
                let Some(name) = field.name() else {
                    continue;
                };

                if name == "image" {
                    let Some(content_type) = field.content_type() else {
                        continue;
                    };
                    if content_type.starts_with("image/") {
                        image_type = Some(content_type.to_string());
                        image = field.bytes().await.ok().map(|b| b.to_vec());
                    }
                }
            }

            if image.is_none() || image_type.is_none() {
                (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": "Invalid upload image params"})),
                )
            } else {
                let url = state
                    .upload_image_to_s3(image.unwrap(), &image_type.unwrap())
                    .await
                    .unwrap();
                println!("upload image url: {}", url);
                (StatusCode::OK, Json(serde_json::json!({"image_url": url})))
            }
        }
        Ok(None) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "security check failure while uploading image"})),
        ),
        Err(e) => {
            println!("upload image error: {:?}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        }
    }
}

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
            let config = state.get_app_config(payload.target_bot_id).await.unwrap();
            let json_config: Value = serde_json::from_str(&config).unwrap();
            (StatusCode::OK, Json(json_config))
        }
        Ok(None) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "security check failure while getting config"})),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

pub async fn update_goal_config(
    headers: HeaderMap,
    Path(bot_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<json::GoalPropsQueryParam>,
) -> impl IntoResponse {
    println!("start update_goal_config");
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
            let Ok(ws_token) = state.get_bot_ws_token(bot_id).await else {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(
                        serde_json::json!({"error": "security check failure while getting bot ws token"}),
                    ),
                );
            };
            match state
                .update_goal_config(payload.target_bot_id, &ws_token, payload.goal_config)
                .await
            {
                Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))),
                Err(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": e.to_string()})),
                ),
            }
        }
        Ok(None) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "security check failure while getting goal config"})),
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
            let Ok(mut tma_app_config) =
                serde_json::from_str::<json::TMAAppConfig>(&payload.app_config)
            else {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": "Invalid app config"})),
                );
            };
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
            Json(serde_json::json!({"error": "security check failure while updating config"})),
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
                tg_api::create_invoice_link(
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

            match tg_api::create_invoice_link(&token, &payload).await {
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
            Json(serde_json::json!({"error": "security check failure while creating invoice"})),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
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
            Json(serde_json::json!({"error": "security check failure while fetching user bots"})),
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
    use tokio::task::JoinHandle;

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
            let Ok(_numeric_id) = tg_api::bot_numeric_id_from_token(&bot.token) else {
                //unreachable, token format is incorrect, but token was verified before the bot was added to the db
                return None;
            };

            let bot_info_task = tokio::spawn({
                let token = bot.token.clone();
                async move { tg_api::get_bot_info(&token).await }
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

            let suspended = if bot.star_debt > MAX_STARS_DEBT {
                Some(true)
            } else {
                None
            };
            let blocked = if bot.blocked { Some(true) } else { None };

            Some(json::TMABotData {
                id: bot.id.clone(),
                numeric_id: bot_numeric_id_from_token(&bot.token).unwrap_or(0),
                name: bot_info_first_name.unwrap_or(bot.id),
                avatar: None,
                user_role: "owner".to_string(),
                owner: json::TMAUserData {
                    id: owner.id,
                    username: owner.username.clone().unwrap_or_default(),
                    name: format!("{} {}", owner.first_name, owner.last_name),
                    avatar_url: Some(owner.photo_url),
                },
                admins,
                suspended,
                debt: Some(bot.star_debt.floor() as i64),
                blocked,
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
            let Ok(_numeric_id) = tg_api::bot_numeric_id_from_token(&bot.token) else {
                //unreachable, token format is incorrect, token was verified before the bot was added to the db
                return None;
            };

            let bot_info_task = tokio::spawn({
                let token = bot.token.clone();
                async move { tg_api::get_bot_info(&token).await }
            });

            let owner_info_task = tokio::spawn({
                let token = bot.token.clone();
                let owner_id = bot.owner;
                async move { tg_api::get_user_info(&token, owner_id).await }
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

            let suspended = if bot.star_debt > MAX_STARS_DEBT {
                Some(true)
            } else {
                None
            };
            let blocked = if bot.blocked { Some(true) } else { None };

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
                suspended,
                debt: Some(bot.star_debt.floor() as i64),
                blocked,
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
            let admin_info = tg_api::get_user_info(&token, admin_id).await;
            (admin_id, admin_info)
        });
        admin_futures.push(admin_info_future);
    }

    let mut admins = Vec::with_capacity(admin_ids.len());

    if let Some(user) = mini_app_user {
        if admin_ids.contains(&user.id) {
            admins.push(json::TMAUserData {
                id: user.id,
                username: user.username.clone().unwrap_or_default(),
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
                            username: user.username.unwrap_or_default(),
                            name: format!("{} {}", user.first_name, user.last_name),
                            avatar_url: Some(user.photo_url),
                        },
                        admins: vec![],
                        suspended: None,
                        debt: None,
                        blocked: None,
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
            Json(serde_json::json!({"error": "security check failure while adding bot"})),
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
            Json(serde_json::json!({"error": "security check failure while adding bot admin"})),
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
            Json(serde_json::json!({"error": "security check failure while removing bot admin"})),
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
            if let Some(_user) = check_hash_in_headers(&headers, &bot.token) {
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

    let avatar_url_result = tg_api::get_avatar_url(&token, user_id_parsed).await;

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
            match http::get(&uri, None).await {
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
            Json(serde_json::json!({"error": "security check failure while removing bot"})),
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
            Json(serde_json::json!({"error": "security check failure while changing bot token"})),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

pub async fn refresh_layer_token(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<json::RefreshLayerTokenQueryParams>,
) -> impl IntoResponse {
    let security_check = state
        .with_record(MAIN_BOT_ID, |bot| {
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
            let res = state
                .refresh_layer_token(payload.target_bot_id.clone())
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
            Json(
                serde_json::json!({"error": "security check failure while refreshing layer token"}),
            ),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

#[axum::debug_handler]
pub async fn get_debt_invoice_url(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<json::GetDebtInvoiceURLQueryParam>,
) -> impl IntoResponse {
    let security_check = state
        .with_record(MAIN_BOT_ID, |bot| {
            if let Some(_user) = check_hash_in_headers(&headers, &bot.token) {
                Some(())
            } else {
                None
            }
        })
        .await;

    match security_check {
        Ok(Some(_)) => {
            let invoice_url = match state.generate_debt_invoice_url(payload.target_bot_id).await {
                Ok(url) => url,
                Err(e) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": e.to_string()})),
                    );
                }
            };
            (
                StatusCode::OK,
                Json(serde_json::json!({"invoice_url": invoice_url})),
            )
        }
        Ok(None) => (
            StatusCode::BAD_REQUEST,
            Json(
                serde_json::json!({"error": "security check failure while getting debt invoice url"}),
            ),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

pub async fn get_bot_ws_token(
    headers: HeaderMap,
    Path(bot_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let security_check = state
        .with_record(&bot_id, |bot| {
            if let Some(user) = check_hash_in_headers(&headers, &bot.token) {
                if user.id == bot.owner || bot.admins.contains(&user.id) {
                    return Some(bot.ws_token.clone());
                }
            }
            None
        })
        .await;

    match security_check {
        Ok(Some(ws_token)) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "success", "ws_token": ws_token})),
        ),
        Ok(None) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "security check failure while getting bot ws token"})),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

#[axum::debug_handler]
pub async fn ws_handler(
    ws: upgrade::IncomingUpgrade,
    Path(bot_username): Path<String>,
    Query(params): Query<json::WSConnectionParams>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let bot_id = get_bot_id_from_username(&bot_username);
    let ws_token = params.ws_token;

    let security_check = state
        .with_record(&bot_id, |bot| {
            if bot.ws_token == ws_token {
                Some(())
            } else {
                None
            }
        })
        .await;

    match security_check {
        Ok(Some(())) => {
            let (response, fut) = ws.upgrade().unwrap();

            tokio::task::spawn(async move {
                if let Err(e) = handle_client(fut, state.clone(), bot_id).await {
                    eprintln!("Error in websocket connection: {}", e);
                }
            });

            response.into_response()
        }
        Ok(None) => {
            println!("Ws token mismatch");
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("Ws token mismatch"))
                .unwrap();
        }
        Err(e) => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(e.to_string()))
                .unwrap();
        }
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
            println!("Failed security check for {}", bot_id);
            return (StatusCode::UNAUTHORIZED, Json(Value::Null));
        }

        //todo redo
        if bot_id == MAIN_BOT_ID {
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
        let command_text = message
            .text
            .as_ref()
            .unwrap()
            .chars()
            .skip(command.offset as usize)
            .take(command.length as usize)
            .collect::<String>();

        match command_text.as_str() {
            "/start" => Ok("start".to_string()),
            "/help" => Ok("help".to_string()),
            "/donate" => Ok("donate".to_string()),
            "/layer" => {
                let layer_url = state.generate_layer_url(bot_id).await.unwrap();
                Ok(layer_url)
            }
            _ => Err(anyhow::anyhow!("Unknown command")),
        }
    } else {
        Err(anyhow::anyhow!("No command found"))
    }
}

async fn parse_message(
    message: &json::Message,
    state: &Arc<AppState>,
    bot_id: &str,
) -> Result<String> {
    let text = message
        .text
        .clone()
        .ok_or(anyhow::anyhow!("no text in message"))?;
    Ok(text)
}

fn check_secret_token(secret_token: &str, headers: &HeaderMap) -> bool {
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

//handle test cdn
pub async fn sound_handler(Path(sound_name): Path<String>) -> impl IntoResponse {
    let sound_path = format!("server/src/sounds/{}", sound_name);

    let file = File::open(sound_path).await.unwrap();
    let stream = ReaderStream::new(file);
    let body = axum::body::Body::from_stream(stream);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "audio/mpeg")
        .body(body)
        .unwrap()
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
