use anyhow::Result;
use axum::{
    body::Body,
    extract::{Json, Multipart, Path, State},
    response::Response,
};
use hyper::{header, StatusCode};
use serde_json::Value;
use std::{str::FromStr, sync::Arc};

use super::auth::{self, AuthenticatedUser, BotAccess, BotAccessWithPayload};
use crate::{
    app_state::{AppState, ControlledBots, UserRole},
    db::DBBot,
    error::{AppError, AppResult},
    http, json, tg_api,
};

use std::time::Instant;
use tokio::task::JoinHandle;

pub async fn add_bot(
    AuthenticatedUser { user }: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<json::AddBotQueryParam>,
) -> AppResult<Json<Value>> {
    let (bot_id, bot_name) = state.add_bot(&payload.bot_token, user.id).await?;

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

    Ok(Json(
        serde_json::json!({"status": "success", "bot_data": bot_data}),
    ))
}

pub async fn add_bot_admin(
    State(state): State<Arc<AppState>>,
    BotAccessWithPayload { access, payload }: BotAccessWithPayload<json::AddBotAdminQueryParam>,
) -> AppResult<Json<Value>> {
    if access.role != UserRole::Owner {
        return Err(AppError::Forbidden("only owner can add admin".into()));
    }

    let admin_info = state
        .add_bot_admin(access.user.id, &payload.bot_id, payload.admin_id)
        .await?;

    Ok(Json(
        serde_json::json!({"status": "success", "admin_info": admin_info}),
    ))
}

pub async fn remove_bot_admin(
    State(state): State<Arc<AppState>>,
    BotAccessWithPayload { access, payload }: BotAccessWithPayload<json::RemoveBotAdminQueryParam>,
) -> AppResult<Json<Value>> {
    if access.role != UserRole::Owner {
        return Err(AppError::Forbidden("only owner can remove admin".into()));
    }

    state
        .remove_bot_admin(access.user.id, &payload.bot_id, payload.admin_id)
        .await?;

    Ok(Json(serde_json::json!({"status": "success"})))
}

pub async fn config_handler(
    auth::BotOwnerOrAdmin { .. }: auth::BotOwnerOrAdmin,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<json::ConfigQueryParam>,
) -> AppResult<Json<Value>> {
    let config = state.get_app_config(payload.target_bot_id).await?;
    let json_config: Value = serde_json::from_str(&config)
        .map_err(|e| AppError::Internal(format!("failed to parse stored config: {}", e)))?;
    Ok(Json(json_config))
}

pub async fn update_config(
    auth::BotOwnerOrAdmin { .. }: auth::BotOwnerOrAdmin,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<json::UpdateConfigQueryParam>,
) -> AppResult<Json<Value>> {
    let target_bot_token = state
        .get_bot_token(payload.target_bot_id.to_string())
        .await?;

    let mut tma_app_config: json::TMAAppConfig = serde_json::from_str(&payload.app_config)
        .map_err(|_| AppError::BadRequest("invalid app config".into()))?;

    generate_invoice_urls(&mut tma_app_config, &target_bot_token).await;

    let config_str = serde_json::to_string(&tma_app_config)
        .map_err(|e| AppError::Internal(format!("failed to serialize config: {}", e)))?;

    state
        .update_bot_config(payload.target_bot_id, config_str)
        .await?;

    Ok(Json(serde_json::json!({"status": "ok"})))
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
) -> AppResult<Json<Value>> {
    let controlled_bots = state.get_controlled_bots(web_app_user.id).await?;
    let json_value = convert_controlled_bots_to_json_value(
        controlled_bots,
        web_app_user,
        state.config.max_stars_debt,
    )
    .await?;
    Ok(Json(json_value))
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
) -> AppResult<Json<Value>> {
    if access.role != UserRole::Owner {
        return Err(AppError::Forbidden("only owner can remove bot".into()));
    }

    state.remove_bot(access.user.id, payload.bot_id).await?;
    Ok(Json(serde_json::json!({"status": "success"})))
}

pub async fn change_bot_token(
    State(state): State<Arc<AppState>>,
    BotAccessWithPayload { access, payload }: BotAccessWithPayload<json::ChangeBotTokenQueryParam>,
) -> AppResult<Json<Value>> {
    if access.role != UserRole::Owner && access.role != UserRole::Admin {
        return Err(AppError::Forbidden(
            "only owner or admin can change bot token".into(),
        ));
    }

    state
        .change_bot_token(access.user.id, payload.bot_id, payload.new_token)
        .await?;
    Ok(Json(serde_json::json!({"status": "success"})))
}

pub async fn get_debt_invoice_url(
    State(state): State<Arc<AppState>>,
    BotAccessWithPayload { access, payload }: BotAccessWithPayload<json::GetDebtInvoiceURLQueryParam>,
) -> AppResult<Json<Value>> {
    if access.role != UserRole::Owner && access.role != UserRole::Admin {
        return Err(AppError::Forbidden(
            "only owner or admin can get debt invoice URL".into(),
        ));
    }

    let invoice_url = state.generate_debt_invoice_url(payload.target_bot_id).await?;
    Ok(Json(serde_json::json!({"invoice_url": invoice_url})))
}

/// Proxies a Telegram avatar so the bot token stays on the server.
pub async fn avatar_url_handler(
    BotAccess { bot, .. }: BotAccess,
    Path((_bot_id, user_id)): Path<(String, String)>,
    State(_state): State<Arc<AppState>>,
) -> AppResult<Response> {
    let user_id_parsed: u64 = user_id
        .parse()
        .map_err(|_| AppError::BadRequest("invalid user id".into()))?;

    let avatar_url = tg_api::get_avatar_url(&bot.token, user_id_parsed)
        .await
        .map_err(|e| AppError::Internal(format!("failed to get avatar url: {}", e)))?
        .ok_or_else(|| AppError::NotFound("no avatar found for this user".into()))?;

    let uri = hyper::Uri::from_str(&avatar_url)
        .map_err(|_| AppError::BadRequest("invalid avatar URL format".into()))?;

    let response = http::get(&uri, None)
        .await
        .map_err(|e| AppError::Internal(format!("failed to download image: {}", e)))?;

    if response.status != StatusCode::OK {
        return Err(AppError::BadRequest(
            "failed to download avatar image".into(),
        ));
    }

    let image_data = response.to_bytes().to_vec();

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/jpeg")
        .body(Body::from(image_data))
        .map_err(|e| {
            tracing::error!(error = %e, "failed to build avatar response");
            AppError::Internal("failed to build response".into())
        })
}

pub async fn upload_image(
    AuthenticatedUser { .. }: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> AppResult<Json<Value>> {
    let mut image: Option<Vec<u8>> = None;
    let mut image_type: Option<String> = None;

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

    let (img, img_type) = image
        .zip(image_type)
        .ok_or_else(|| AppError::BadRequest("invalid upload image params".into()))?;

    let url = state
        .upload_image_to_s3(img, &img_type)
        .await
        .map_err(|e| AppError::Internal(format!("failed to upload image: {}", e)))?;

    tracing::debug!(url = %url, "uploaded image");
    Ok(Json(serde_json::json!({"image_url": url})))
}
