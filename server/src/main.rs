use anyhow::Result;
use app_state::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use handlers::{
    add_bot, add_bot_admin, avatar_url_handler, change_bot_token, config_handler, create_invoice,
    fetch_user_bots, get_bot_ws_token, layer, mini_app, refresh_layer_token, remove_bot,
    remove_bot_admin, update_config, webhook_handler, ws_handler,
};

use main_bot::{MAIN_BOT_ADMINS, MAIN_BOT_ID, MAIN_BOT_OWNER, MAIN_BOT_TOKEN};
use std::{num::NonZeroUsize, sync::Arc};
use tokio::{
    self,
    fs::OpenOptions,
    io::AsyncWriteExt,
    sync::{broadcast, Mutex},
};

use crate::handlers::get_debt_invoice_url;

#[macro_use]
extern crate dotenv_codegen;

mod api;
pub mod app_state;
pub mod db;
mod handlers;
mod http;
pub mod json;
pub mod main_bot;
pub mod ws_server;

const PATH_TO_DIST: &str = "../../tma-client/dist/src/pages";
const HTML_MINI_APP: &str = include_str!("../../tma-client/dist/src/pages/mini_app.html");
const HTML_MAIN_BOT_MINI_APP: &str =
    include_str!("../../tma-client/dist/src/pages/main_bot_mini_app.html");
const HTML_LAYER: &str = include_str!("../../tma-client/dist/src/pages/layer.html");
const HTML_BLOCKED_APP: &str = include_str!("../../tma-client/dist/src/pages/blocked_app.html");

const WEBHOOK_ALLOWED_UPDATES: &str = "[%22message%22,%22pre_checkout_query%22]";

pub const CACHE_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(10) };
pub const ROOM_CAPACITY: usize = 100;

// Maximum allowed stars debt before bot gets blocked
pub const MAX_STARS_DEBT: i64 = 100;
// Maximum days since last payment before bot gets blocked
pub const MAX_DAYS_SINCE_LAST_PAYMENT: u64 = 30;
pub const PROCENT_FOR_MAIN_BOT: f32 = 0.03;

#[tokio::main]
async fn main() {
    // let (event_tx, _) = broadcast::channel::<json::WSDonationEvent>(100);

    let app_state = AppState::new().await;
    let arc_app_state = Arc::new(app_state);
    arc_app_state.prepare().await.unwrap();

    //stardonationservice no need /app route cause /:bot_id/app enough
    let app = Router::new()
        .route("/:bot_id/webhook", post(webhook_handler))
        .route("/:bot_id/app", get(mini_app))
        .route("/:bot_id/createInvoice", post(create_invoice))
        .route("/:bot_id/avatar/:user_id", get(avatar_url_handler))
        .route("/:bot_id/config", post(config_handler))
        .route("/:bot_id/layer", get(layer))
        //only for owner and admins
        .route("/:bot_id/updateConfig", post(update_config))
        //only for owner and admins
        .route("/:bot_id/ws_token", get(get_bot_ws_token))
        //main bot routes
        .route("/stardonationservice/controlledBots", get(fetch_user_bots))
        .route("/stardonationservice/addBot", post(add_bot))
        .route("/stardonationservice/addBotAdmin", post(add_bot_admin))
        .route(
            "/stardonationservice/removeBotAdmin",
            post(remove_bot_admin),
        )
        .route("/stardonationservice/removeBot", post(remove_bot))
        .route(
            "/stardonationservice/changeBotToken",
            post(change_bot_token),
        )
        .route(
            "/stardonationservice/refreshLayerToken",
            post(refresh_layer_token),
        )
        .route(
            "/stardonationservice/getDebtInvoiceURL",
            post(get_debt_invoice_url),
        )
        //test cdn
        .route("/sound/:sound_name", get(handlers::sound_handler))
        // ws server
        // exmple url: wss://host/ws/bot_username?ws_token=1234567890
        .route("/ws/:bot_username", get(ws_handler))
        //app state
        .with_state(arc_app_state.clone());

    let _axum_task = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind("localhost:5001")
            .await
            .unwrap();
        println!("Listening on {:?}", listener);

        axum::serve(listener, app).await.unwrap();
    });

    let _test_layer_task = tokio::spawn({
        let arc_app_state = arc_app_state.clone();
        async move {
            loop {
                let msg = json::WSDonationEvent {
                    ok: true,
                    from: "username".to_string(),
                    total_amount: 100,
                    invoice_payload: "https://i.giphy.com/media/3oEjI6SIIHBdRx6PBI/giphy.gif"
                        .to_string(),
                    message: "test message".to_string(),
                };

                arc_app_state
                    .send_donation_to_room_members(
                        "star_donation".to_string(),
                        serde_json::to_vec(&msg).unwrap(),
                    )
                    .await;
                tokio::time::sleep(std::time::Duration::from_secs(7)).await;
            }
        }
    });

    _test_layer_task.abort();

    // let _test_task_2 = tokio::spawn({
    //     let arc_app_state = arc_app_state.clone();
    //     async move {
    //         tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    //         arc_app_state.refresh_layer_token("star_donation".to_string()).await.unwrap();
    //     }
    // });

    // let _channel_keeper = event_tx.subscribe();
    tokio::signal::ctrl_c().await.unwrap();
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
