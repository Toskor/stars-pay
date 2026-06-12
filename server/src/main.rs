//! tg-stars server: Axum HTTP API + WebSocket fan-out for the Telegram
//! Stars donation service. See `README.md` for the high-level architecture.

#![forbid(unsafe_code)]

use app_state::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use std::{net::SocketAddr, sync::Arc, time::Duration};

pub mod app_state;
pub mod config;
pub mod db;
pub mod error;
mod handlers;
mod http;
pub mod json;
pub mod main_bot;
pub mod proto;
pub mod ratelimit;
pub mod s3_api;
mod tg_api;
pub mod ws_server;

// Prebuilt frontend assets. The Svelte sources live on the `frontend` branch;
// see README for how to rebuild and refresh these files.
const HTML_MINI_APP: &str = include_str!("../static/mini_app.html");
const HTML_MAIN_BOT_MINI_APP: &str = include_str!("../static/main_bot_mini_app.html");
const HTML_LAYER: &str = include_str!("../static/layer.html");
const HTML_BLOCKED_APP: &str = include_str!("../static/blocked_app.html");
const HTML_GOAL_APP: &str = include_str!("../static/goal_app.html");

const WEBHOOK_ALLOWED_UPDATES: &str = "[%22message%22,%22pre_checkout_query%22]";
const DEBT_CHECK_INTERVAL: Duration = Duration::from_secs(60 * 60 * 24);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,tg_stars=debug")),
        )
        .init();

    let config = config::Config::from_env().map_err(|e| {
        tracing::error!(error = %e, "failed to load configuration; check .env or env vars");
        e
    })?;

    let port = config.port;
    let cors = build_cors_layer(&config.cors_origin)?;

    let app_state = Arc::new(AppState::new(config).await?);
    app_state.prepare().await?;

    let router = build_router(app_state.clone()).layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
        tracing::error!(%addr, error = %e, "failed to bind tcp listener");
        e
    })?;
    tracing::info!(%addr, "axum server listening");

    // Background loop: walk every bot, suspend or notify on debt thresholds.
    let debt_task = tokio::spawn({
        let app_state = app_state.clone();
        async move {
            let mut ticker = tokio::time::interval(DEBT_CHECK_INTERVAL);
            // Avoid a burst of catch-up ticks if we're behind schedule.
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                ticker.tick().await;
                if let Err(e) = app_state.process_bots_debt_status().await {
                    tracing::error!(error = %e, "error processing bots debt status");
                }
            }
        }
    });

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("server shut down, aborting background tasks");
    debt_task.abort();

    Ok(())
}

/// Wait for SIGINT (Ctrl+C) or SIGTERM (systemd `stop`, container stop).
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("Ctrl+C received"),
        _ = terminate => tracing::info!("SIGTERM received"),
    }
}

fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/:bot_id/webhook", post(handlers::webhook::webhook_handler))
        .route(
            "/:bot_id/avatar/:user_id",
            get(handlers::bot::avatar_url_handler),
        )
        .route("/:bot_id/config", post(handlers::bot::config_handler))
        .route("/:bot_id/updateConfig", post(handlers::bot::update_config))
        .route(
            "/:bot_id/updateGoalConfig",
            post(handlers::update_goal_config),
        )
        .route("/:bot_id/ws_token", get(handlers::get_bot_ws_token))
        // main control bot routes
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
        .route(
            "/stardonationservice/removeBot",
            post(handlers::bot::remove_bot),
        )
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
        // ws example: wss://host/ws/<bot_username>?ws_token=...
        .route("/ws/:bot_username", get(handlers::ws_handler))
        .with_state(state)
}

fn build_cors_layer(origin: &str) -> anyhow::Result<CorsLayer> {
    let origin = origin
        .parse::<axum::http::HeaderValue>()
        .map_err(|e| anyhow::anyhow!("invalid CORS_ORIGIN {origin:?}: {e}"))?;
    Ok(CorsLayer::new()
        .allow_origin(origin)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers(Any)
        .allow_credentials(false))
}
