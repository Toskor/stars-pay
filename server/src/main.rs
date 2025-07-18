use app_state::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use std::{num::NonZeroUsize, sync::Arc};
use tokio::{self};

#[macro_use]
extern crate dotenv_codegen;

pub mod app_state;
pub mod db;
mod handlers;
mod http;
pub mod json;
pub mod main_bot;
pub mod s3_api;
mod tg_api;
pub mod ws_server;

const HTML_MINI_APP: &str = include_str!("../../tma-client/dist/src/pages/mini_app.html");
const HTML_MAIN_BOT_MINI_APP: &str =
    include_str!("../../tma-client/dist/src/pages/main_bot_mini_app.html");
const HTML_LAYER: &str = include_str!("../../tma-client/dist/src/pages/layer.html");
const HTML_BLOCKED_APP: &str = include_str!("../../tma-client/dist/src/pages/blocked_app.html");

const WEBHOOK_ALLOWED_UPDATES: &str = "[%22message%22,%22pre_checkout_query%22]";

pub const CACHE_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(10) };
pub const ROOM_CAPACITY: usize = 100;

// Maximum allowed stars debt before bot gets blocked
pub const MAX_STARS_DEBT: f64 = 100.0;
// Maximum days since last payment before bot gets blocked
pub const DAYS_SINCE_LAST_PAYMENT_FOR_BLOCK: u64 = 30;
pub const DAYS_SINCE_LAST_PAYMENT_FOR_NOTIFICATION: u64 = 28;
pub const PROCENT_FOR_MAIN_BOT: f32 = 0.03;

#[tokio::main]
async fn main() {
    let app_state = AppState::new().await;
    let arc_app_state = Arc::new(app_state);
    arc_app_state.prepare().await.unwrap();

    //stardonationservice no need /app route cause /:bot_id/app enough
    let router = Router::new()
        .route("/:bot_id/webhook", post(handlers::webhook_handler))
        // .route("/:bot_id/app", get(handlers::mini_app))
        .route("/:bot_id/createInvoice", post(handlers::create_invoice))
        .route(
            "/:bot_id/avatar/:user_id",
            get(handlers::avatar_url_handler),
        )
        .route("/:bot_id/config", post(handlers::config_handler))
        .route("/:bot_id/layer", get(handlers::layer))
        //only for owner and admins
        .route("/:bot_id/updateConfig", post(handlers::update_config))
        //only for owner and admins
        .route("/:bot_id/ws_token", get(handlers::get_bot_ws_token))
        //main bot routes
        .route(
            "/stardonationservice/controlledBots",
            get(handlers::fetch_user_bots),
        )
        .route("/stardonationservice/addBot", post(handlers::add_bot))
        .route(
            "/stardonationservice/addBotAdmin",
            post(handlers::add_bot_admin),
        )
        .route(
            "/stardonationservice/removeBotAdmin",
            post(handlers::remove_bot_admin),
        )
        .route("/stardonationservice/removeBot", post(handlers::remove_bot))
        .route(
            "/stardonationservice/changeBotToken",
            post(handlers::change_bot_token),
        )
        .route(
            "/stardonationservice/refreshLayerToken",
            post(handlers::refresh_layer_token),
        )
        .route(
            "/stardonationservice/getDebtInvoiceURL",
            post(handlers::get_debt_invoice_url),
        )
        .route(
            "/stardonationservice/makeTestDonation",
            post(handlers::make_test_donation),
        )
        .route("/sound/:sound_name", get(handlers::sound_handler))
        // ws server
        // exmple url: wss://host/ws/bot_username?ws_token=1234567890
        .route("/ws/:bot_username", get(handlers::ws_handler))
        .with_state(arc_app_state.clone())
        .layer(cors_layer());

    let _axum_task = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind("localhost:5001")
            .await
            .unwrap();
        println!("Listening on {:?}", listener);

        axum::serve(listener, router).await.unwrap();
    });

    let _task_for_proccess_bots_debt = tokio::spawn({
        let app_state = arc_app_state.clone();
        async move {
            loop {
                app_state.process_bots_debt_status().await.unwrap();
                tokio::time::sleep(std::time::Duration::from_secs(60 * 60 * 24)).await;
            }
        }
    });

    tokio::signal::ctrl_c().await.unwrap();
}

fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(
            "https://tg-stars.s3-website.nl-ams.scw.cloud"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
        )
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        //todo mb make more strict
        .allow_headers(Any)
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
            .update_mini_app_source("star_donation".to_string(), false)
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
