use app_state::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use std::sync::Arc;
use tokio::{self};

pub mod app_state;
pub mod config;
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
const HTML_GOAL_APP: &str = include_str!("../../tma-client/dist/src/pages/goal_app.html");

const WEBHOOK_ALLOWED_UPDATES: &str = "[%22message%22,%22pre_checkout_query%22]";

#[tokio::main]
async fn main() {
    let config = match config::Config::from_env() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            eprintln!("Please check your .env file or environment variables");
            std::process::exit(1);
        }
    };
    let port = config.port.clone();
    let app_state = AppState::new(config).await;
    let arc_app_state = Arc::new(app_state);
    if let Err(e) = arc_app_state.prepare().await {
        eprintln!("Failed to prepare application state: {}", e);
        std::process::exit(1);
    }

    //stardonationservice no need /app route cause /:bot_id/app enough
    let router = Router::new()
        .route(
            "/:bot_id/webhook",
            post(handlers::webhook::webhook_handler),
        )
        // .route("/:bot_id/app", get(handlers::mini_app))
        .route("/:bot_id/createInvoice", post(handlers::bot::create_invoice))
        .route(
            "/:bot_id/avatar/:user_id",
            get(handlers::bot::avatar_url_handler),
        )
        .route("/:bot_id/config", post(handlers::bot::config_handler))
        .route("/:bot_id/updateConfig", post(handlers::bot::update_config))
        // goal routes
        .route(
            "/:bot_id/updateGoalConfig",
            post(handlers::update_goal_config),
        )
        .route("/:bot_id/ws_token", get(handlers::get_bot_ws_token))
        //main bot routes
        .route(
            "/stardonationservice/controlledBots",
            get(handlers::bot::fetch_user_bots),
        )
        .route("/stardonationservice/addBot", post(handlers::bot::add_bot))
        .route(
            "/stardonationservice/addBotAdmin",
            post(handlers::bot::add_bot_admin),
        )
        .route(
            "/stardonationservice/removeBotAdmin",
            post(handlers::bot::remove_bot_admin),
        )
        .route("/stardonationservice/removeBot", post(handlers::bot::remove_bot))
        .route(
            "/stardonationservice/changeBotToken",
            post(handlers::bot::change_bot_token),
        )
        .route(
            "/stardonationservice/refreshLayerToken",
            post(handlers::refresh_layer_token),
        )
        .route(
            "/stardonationservice/getDebtInvoiceURL",
            post(handlers::bot::get_debt_invoice_url),
        )
        .route(
            "/stardonationservice/makeTestDonation",
            post(handlers::make_test_donation),
        )
        .route(
            "/stardonationservice/uploadImage",
            post(handlers::bot::upload_image),
        )
        .route("/sound/:sound_name", get(handlers::sound_handler))
        // ws server
        // exmple url: wss://host/ws/bot_username?ws_token=1234567890
        .route("/ws/:bot_username", get(handlers::ws_handler))
        .with_state(arc_app_state.clone())
        .layer(cors_layer());

    let _axum_task = tokio::spawn(async move {
        let listener = match tokio::net::TcpListener::bind(format!("localhost:{}", port)).await {
            Ok(listener) => listener,
            Err(e) => {
                eprintln!("Failed to bind to port {}: {}", port, e);
                return;
            }
        };
        println!("Listening on {:?}", listener);

        if let Err(e) = axum::serve(listener, router).await {
            eprintln!("Server error: {}", e);
        }
    });

    let _task_for_proccess_bots_debt = tokio::spawn({
        let app_state = arc_app_state.clone();
        async move {
            loop {
                if let Err(e) = app_state.process_bots_debt_status().await {
                    eprintln!("Error processing bots debt status: {}", e);
                }
                tokio::time::sleep(std::time::Duration::from_secs(60 * 60 * 24)).await;
            }
        }
    });

    tokio::signal::ctrl_c().await.unwrap();
}

fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        // .allow_origin(Any)
        .allow_origin(
            "https://tg-stars.s3-website.nl-ams.scw.cloud"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
        )
        // .allow_methods(Any)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::OPTIONS,
        ])
        //todo mb make more strict
        .allow_headers(Any)
        .allow_credentials(false)
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
        let config = config::Config::from_env().unwrap();
        let app_state = AppState::new(config).await;
        app_state
            .update_mini_app_source("star_donation".to_string(), false)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn get_controlled_bots() {
        let config = config::Config::from_env().unwrap();
        let app_state = AppState::new(config).await;
        let controlled_bots = app_state.get_controlled_bots(348135868).await.unwrap();
        println!("{:?}", controlled_bots);
    }
}
