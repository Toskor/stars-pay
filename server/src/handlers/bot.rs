use anyhow::Result;
use axum::{
    body::Body,
    extract::{Json, Multipart, Path, State},
    response::{IntoResponse, Response},
};
use hyper::{header, StatusCode};
use serde_json::Value;
use std::{str::FromStr, sync::Arc};

use super::auth::{self, AuthenticatedUser, BotAccess, BotAccessWithPayload};
use crate::{
    app_state::{AppState, ControlledBots, UserRole},
    db::DBBot,
    http, json, tg_api,
};

use std::time::Instant;
use tokio::task::JoinHandle;

pub async fn add_bot(
    AuthenticatedUser { user }: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<json::AddBotQueryParam>,
) -> impl IntoResponse {
    let res = state.add_bot(&payload.bot_token, user.id).await;
    match res {
        Ok((bot_id, bot_name)) => {
            let bot_data = json::TMABotData {
                id: bot_id,
                numeric_id: tg_api::bot_numeric_id_from_token(&payload.bot_token).unwrap_or(0),
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

pub async fn add_bot_admin(
    State(state): State<Arc<AppState>>,
    BotAccessWithPayload { access, payload }: BotAccessWithPayload<json::AddBotAdminQueryParam>,
) -> impl IntoResponse {
    if access.role != UserRole::Owner {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Only owner can add admin"})),
        );
    }

    let res = state
        .add_bot_admin(access.user.id, &payload.bot_id, payload.admin_id)
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

pub async fn remove_bot_admin(
    State(state): State<Arc<AppState>>,
    BotAccessWithPayload { access, payload }: BotAccessWithPayload<json::RemoveBotAdminQueryParam>,
) -> impl IntoResponse {
    if access.role != UserRole::Owner {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Only owner can remove admin"})),
        );
    }

    let res = state
        .remove_bot_admin(access.user.id, &payload.bot_id, payload.admin_id)
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

pub async fn config_handler(
    auth::BotOwnerOrAdmin { .. }: auth::BotOwnerOrAdmin,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<json::ConfigQueryParam>,
) -> impl IntoResponse {
    match state.get_app_config(payload.target_bot_id).await {
        Ok(config) => match serde_json::from_str::<Value>(&config) {
            Ok(json_config) => (StatusCode::OK, Json(json_config)),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Failed to parse config: {}", e)})),
            ),
        },
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": format!("Failed to get config: {}", e)})),
        ),
    }
}

pub async fn update_config(
    auth::BotOwnerOrAdmin { .. }: auth::BotOwnerOrAdmin,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<json::UpdateConfigQueryParam>,
) -> impl IntoResponse {
    let target_bot_token = match state.get_bot_token(payload.target_bot_id.to_string()).await {
        Ok(token) => token,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("Failed to get bot token: {}", e)})),
            );
        }
    };
    let Ok(mut tma_app_config) = serde_json::from_str::<json::TMAAppConfig>(&payload.app_config)
    else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid app config"})),
        );
    };

    generate_invoice_urls(&mut tma_app_config, &target_bot_token).await;

    let config_str = match serde_json::to_string(&tma_app_config) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Failed to serialize config: {}", e)})),
            );
        }
    };

    let upd_res = state
        .update_bot_config(payload.target_bot_id, config_str)
        .await;

    match upd_res {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": err.to_string()})),
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

pub async fn fetch_user_bots(
    AuthenticatedUser { user: web_app_user }: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.get_controlled_bots(web_app_user.id).await {
        Ok(controlled_bots) => {
            match convert_controlled_bots_to_json_value(
                controlled_bots,
                web_app_user,
                state.config.max_stars_debt,
            )
            .await
            {
                Ok(json_value) => (StatusCode::OK, Json(json_value)),
                Err(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": e.to_string()})),
                ),
            }
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": format!("Failed to get controlled bots: {}", e)})),
        ),
    }
}

async fn convert_controlled_bots_to_json_value(
    controlled_bots: ControlledBots,
    web_app_user: json::WebAppUser, //user that opened mini app
    max_stars_debt: f64,
) -> Result<Value> {
    let mut tasks: Vec<JoinHandle<Option<json::TMABotData>>> = Vec::new();

    // Process owner bots
    process_owner_bots(
        &controlled_bots.owner_bots,
        &web_app_user,
        &mut tasks,
        max_stars_debt,
    );

    // Process admin bots
    process_admin_bots(
        &controlled_bots.admin_bots,
        &web_app_user,
        &mut tasks,
        max_stars_debt,
    );

    let tasks_start = Instant::now();

    let mut bots = Vec::with_capacity(tasks.len());
    for task in tasks {
        match task.await {
            Ok(Some(tma_bot_data)) => {
                bots.push(tma_bot_data);
            }
            Ok(None) => {
                tracing::debug!("task returned None");
            }
            Err(e) => {
                tracing::warn!(error = %e, "task join error");
            }
        }
    }

    tracing::debug!(elapsed = ?tasks_start.elapsed(), "tasks completed");

    let main_page_props = json::MainBotMainPageProps { bots };
    let json_value = serde_json::to_value(&main_page_props)?;

    Ok(json_value)
}

fn process_owner_bots(
    owner_bots: &Vec<DBBot>,
    web_app_user: &json::WebAppUser,
    tasks: &mut Vec<tokio::task::JoinHandle<Option<json::TMABotData>>>,
    max_stars_debt: f64,
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
                        bot_info.result.map(|r| r.first_name)
                    } else {
                        None
                    }
                }
                Ok(Err(e)) => {
                    //todo here need to log error
                    tracing::warn!(bot_id = %bot.id, error = %e, "error fetching bot info");
                    None
                }
                Err(e) => {
                    //task error
                    tracing::warn!(error = %e, "bot_info task error");
                    None
                }
            };

            let suspended = if bot.star_debt > max_stars_debt {
                Some(true)
            } else {
                None
            };
            let blocked = if bot.blocked { Some(true) } else { None };

            Some(json::TMABotData {
                id: bot.id.clone(),
                numeric_id: tg_api::bot_numeric_id_from_token(&bot.token).unwrap_or(0),
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
    max_stars_debt: f64,
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
                        bot_info.result.map(|r| r.first_name)
                    } else {
                        None
                    }
                }
                Ok(Err(e)) => {
                    //todo here need to log error
                    tracing::warn!(bot_id = %bot.id, error = %e, "error fetching bot info");
                    None
                }
                Err(e) => {
                    //task error
                    tracing::warn!(error = %e, "bot_info task error");
                    None
                }
            };

            let owner_info = match owner_info_task.await {
                Ok(Ok(owner_info)) => {
                    if owner_info.ok {
                        match owner_info.result {
                            Some(result) => result,
                            None => return None,
                        }
                    } else {
                        return None;
                    }
                }
                Ok(Err(e)) => {
                    //todo here need to log error
                    tracing::warn!(bot_id = %bot.id, error = %e, "error fetching owner info");
                    return None;
                }
                Err(e) => {
                    //task error
                    tracing::warn!(error = %e, "owner_info task error");
                    return None;
                }
            };

            let suspended = if bot.star_debt > max_stars_debt {
                Some(true)
            } else {
                None
            };
            let blocked = if bot.blocked { Some(true) } else { None };

            Some(json::TMABotData {
                id: bot.id.clone(),
                numeric_id: tg_api::bot_numeric_id_from_token(&bot.token).unwrap_or(0),
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
        let admin_info_future = tokio::task::spawn(async move {
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
                    if let Some(result) = admin.result {
                        admins.push(json::TMAUserData {
                            id: admin_id,
                            name: format!("{} {}", result.first_name, result.last_name),
                            username: result.username,
                            avatar_url: None,
                        });
                    }
                }
            }
        }
    }

    admins
}

pub async fn remove_bot(
    State(state): State<Arc<AppState>>,
    BotAccessWithPayload { access, payload }: BotAccessWithPayload<json::RemoveBotQueryParam>,
) -> impl IntoResponse {
    if access.role != UserRole::Owner {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Only owner can remove bot"})),
        );
    }

    let res = state.remove_bot(access.user.id, payload.bot_id).await;
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

pub async fn change_bot_token(
    State(state): State<Arc<AppState>>,
    BotAccessWithPayload { access, payload }: BotAccessWithPayload<json::ChangeBotTokenQueryParam>,
) -> impl IntoResponse {
    if access.role != UserRole::Owner && access.role != UserRole::Admin {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Only owner or admin can change bot token"})),
        );
    }

    let res = state
        .change_bot_token(access.user.id, payload.bot_id, payload.new_token)
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

// #[axum::debug_handler]
pub async fn get_debt_invoice_url(
    State(state): State<Arc<AppState>>,
    BotAccessWithPayload { access, payload }: BotAccessWithPayload<json::GetDebtInvoiceURLQueryParam>,
) -> impl IntoResponse {
    if access.role != UserRole::Owner && access.role != UserRole::Admin {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Only owner or admin can get debt invoice URL"})),
        );
    }

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

// #[axum::debug_handler]
pub async fn create_invoice(
    auth::BotOwnerOrAdmin { bot, .. }: auth::BotOwnerOrAdmin,
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<json::CreateInvoiceQueryParam>,
) -> impl IntoResponse {
    match tg_api::create_invoice_link(&bot.token, &payload).await {
        Ok(url) => (
            StatusCode::OK,
            Json(serde_json::json!({"invoice_url": url})),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

pub async fn avatar_url_handler(
    auth::BotAccess { bot, .. }: auth::BotAccess,
    Path((_bot_id, user_id)): Path<(String, String)>,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let token = bot.token.clone();

    let user_id_parsed = match user_id.parse::<u64>() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Invalid user ID"})),
            )
                .into_response();
        }
    };

    let avatar_url_result = tg_api::get_avatar_url(&token, user_id_parsed).await;

    match avatar_url_result {
        Ok(Some(avatar_url)) => {
            let uri = match hyper::Uri::from_str(&avatar_url) {
                Ok(uri) => uri,
                Err(_) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "Invalid avatar URL format"})),
                    )
                        .into_response();
                }
            };

            // Download the image
            match http::get(&uri, None).await {
                Ok(response) => {
                    if response.status != StatusCode::OK {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(serde_json::json!({"error": "Failed to download avatar image"})),
                        )
                            .into_response();
                    }

                    // For Telegram avatar images, the content type is always image/jpeg
                    let content_type = "image/jpeg";
                    let image_data = response.to_bytes().to_vec();

                    return Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, content_type)
                        .body(Body::from(image_data))
                        .map_err(|e| {
                            tracing::error!(error = %e, "failed to build response");
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(serde_json::json!({"error": "Failed to build response"})),
                            )
                                .into_response()
                        })
                        .unwrap_or_else(|resp| resp);
                }
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": format!("Failed to download image: {}", e)})),
                    )
                        .into_response();
                }
            }
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "No avatar found for this user"})),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Failed to get avatar URL: {}", e)})),
            )
                .into_response();
        }
    }
}

pub async fn upload_image(
    AuthenticatedUser { .. }: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
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

    match (image, image_type) {
        (Some(img), Some(img_type)) => match state.upload_image_to_s3(img, &img_type).await {
            Ok(url) => {
                tracing::debug!(url = %url, "uploaded image");
                (StatusCode::OK, Json(serde_json::json!({"image_url": url})))
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Failed to upload image: {}", e)})),
            ),
        },
        _ => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid upload image params"})),
        ),
    }
}
