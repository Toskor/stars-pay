use anyhow::Result;
use app_state::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use db::{DBBot, DataBase};
use handlers::{
    add_bot, add_bot_admin, avatar_url_handler, change_bot_token, config_handler, create_invoice, fetch_user_bots, mini_app, remove_bot, remove_bot_admin, update_config, webhook_handler
};

use main_bot::{MAIN_BOT_ADMINS, MAIN_BOT_ID, MAIN_BOT_OWNER, MAIN_BOT_TOKEN};
use std::{num::NonZeroUsize, sync::Arc};
use tokio::{self, fs::OpenOptions, io::AsyncWriteExt, sync::Mutex};

#[macro_use]
extern crate dotenv_codegen;

mod api;
pub mod app_state;
pub mod db;
mod handlers;
pub mod json;
pub mod main_bot;

const PATH_TO_DIST: &str = "../../tma-client/dist/src/pages";
const HTML_MINI_APP: &str = include_str!("../../tma-client/dist/src/pages/mini_app.html");
const HTML_MAIN_BOT_MINI_APP: &str =
    include_str!("../../tma-client/dist/src/pages/main_bot_mini_app.html");

const WEBHOOK_ALLOWED_UPDATES: &str = "[%22message%22,%22pre_checkout_query%22]";

const CACHE_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(100) };

#[tokio::main]
async fn main() {
    let arc_app_state = Arc::new(AppState::new().await);
    arc_app_state.prepare().await.unwrap();

    //stardonationservice no need /app route cause /:bot_id/app enough
    let app = Router::new()
        .route("/:bot_id/webhook", post(webhook_handler))
        .route("/:bot_id/app", get(mini_app))
        .route("/:bot_id/createInvoice", post(create_invoice))
        .route("/:bot_id/avatar/:user_id", get(avatar_url_handler))
        .route("/:bot_id/config", post(config_handler))
        .route("/:bot_id/updateConfig", post(update_config))
        //main bot routes
        .route("/stardonationservice/controlledBots", get(fetch_user_bots))
        .route("/stardonationservice/addBot", post(add_bot))
        .route("/stardonationservice/addBotAdmin", post(add_bot_admin))
        .route(
            "/stardonationservice/removeBotAdmin",
            post(remove_bot_admin),
        )
        .route("/stardonationservice/removeBot", post(remove_bot))
        .route("/stardonationservice/changeBotToken", post(change_bot_token))
        
        //app state
        .with_state(arc_app_state);

    let listener = tokio::net::TcpListener::bind("localhost:5001")
        .await
        .unwrap();
    println!("Listening on {:?}", listener);

    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn value_size() {
        let size_of_null = std::mem::size_of::<Value>();
        println!("Size of Value::Null: {} bytes", size_of_null);
    }

    #[tokio::test]
    async fn upd_mini_app_source() {
        let app_state = AppState::new().await;
        app_state
            .update_mini_app_source("star_donation".to_string())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn get_controlled_bots() {
        let app_state = AppState::new().await;
        let controlled_bots = app_state.get_controlled_bots(348135868).await.unwrap();
        println!("{:?}", controlled_bots);
    }
}
