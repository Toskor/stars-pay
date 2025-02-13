use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use db::{Bot, DataBase};
use handlers::{
    add_bot, add_bot_admin, controlled_bots, create_invoice, handler_print_2, mini_app,
    remove_bot_admin, update_config, webhook_handler,
};
use lru::LruCache;

use main_bot::{MAIN_BOT_ADMINS, MAIN_BOT_ID, MAIN_BOT_OWNER, MAIN_BOT_TOKEN};
use std::{num::NonZeroUsize, sync::Arc};
use tokio::{self, fs::OpenOptions, io::AsyncWriteExt, sync::Mutex};

#[macro_use]
extern crate dotenv_codegen;

mod api;
pub mod db;
mod handlers;
pub mod json;
pub mod main_bot;

const PATH_TO_DIST: &str = "../../tma-client/dist";
const HTML_APP: &str = "";// include_str!("../../web-widgets/dist/mini_app.html");
const HTML_MAIN_APP: &str = "";// include_str!("../../web-widgets/dist/main_bot_app.html");

const WEBHOOK_ALLOWED_UPDATES: &str = "[%22message%22,%22pre_checkout_query%22]";

const CACHE_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(100) };

#[derive(Debug, Clone, Copy)]
pub enum UserRole {
    Owner,
    Admin,
    User,
}

impl UserRole {
    pub fn from_str(role: &str) -> Result<Self> {
        match role {
            "owner" => Ok(UserRole::Owner),
            "admin" => Ok(UserRole::Admin),
            "user" => Ok(UserRole::User),
            _ => Err(anyhow::anyhow!("invalid user role")),
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            UserRole::Owner => "owner",
            UserRole::Admin => "admin",
            UserRole::User => "user",
        }
    }
}

//todo sometimes long delay in webhook (mb when change webhook url)
pub struct AppState {
    pub cache: Mutex<LruCache<String, Bot>>,
    pub db: DataBase,
}

impl AppState {
    pub async fn new() -> Self {
        //todo move db path to env?
        let db = DataBase::new_sql_lite("db/bots_data_base.sqlite")
            .await
            .expect("Failed to create database");

        let cache = Mutex::new(LruCache::new(CACHE_SIZE));
        Self { cache, db }
    }

    pub async fn prepare(&self) -> Result<()> {
        //todo create 404.html in mini_app_sources
        //todo add main bot in db

        //main bot
        match self.add_mainbot().await {
            Ok(_) => println!("Bot added"),
            Err(e) => println!("Error: {}", e),
        }

        //second_test_1
        match self
            .add_bot("8090667304:AAFDIkQ7htfPHAjm2Vnzrl5JH6oELo4Y1e4", 348135868)
            .await
        {
            Ok(_) => println!("Bot added"),
            Err(e) => println!("Error: {}", e),
        }

        //star_donation
        match self
            .add_bot("7792542554:AAEVkmVbOKN3ouDPJORrfNZIX2j4uMlEZHs", 348135868)
            .await
        {
            Ok(_) => println!("Bot added"),
            Err(e) => println!("Error: {}", e),
        }

        self.update_mini_app_source(MAIN_BOT_ID.to_string())
            .await
            .unwrap();

        self.update_mini_app_sources().await.unwrap();
        Ok(())
    }

    //todo add new bot:  set cmd,
    pub async fn add_bot(&self, token: &str, owner: u64) -> Result<()> {
        let tg_api_url = format!("https://api.telegram.org/bot{}/", token);
        let bot_info = api::get_bot_info(&tg_api_url).await?;

        let name = bot_info.result.username.to_lowercase();
        let bot_id = name.trim_end_matches("bot").trim_end_matches("_");

        if bot_id == MAIN_BOT_ID {
            return Err(anyhow::anyhow!("Cant add main bot"));
        }

        if self.db.contains_bot(bot_id.to_string()).await? {
            return Err(anyhow::anyhow!("Bot already exists"));
        }

        let api_url = format!("{}{}/", dotenv!("DOMAIN"), bot_id);

        let app_config = format!(
            r#"{{"header_text":"Yoml | Best stream app","buttons":[],"api_url":"{api_url}","page_description": "Here you can make star donation for Streamer","owner":{owner},"admins":[{owner}]}}"#
        );
        let admins = vec![owner];

        let button_text = if bot_id == MAIN_BOT_ID {
            "App"
        } else {
            "Donate"
        };
        let button_url = format!("{api_url}app");
        api::set_menu_button(&tg_api_url, button_text, &button_url).await?;

        let webhook_url = format!("{api_url}webhook");
        let secret_token = api::generate_secret_token();
        api::set_tg_webhook(&tg_api_url, &webhook_url, &secret_token).await?;

        let bot: Bot = Bot::new(
            bot_id.to_string(),
            token.to_string(),
            secret_token,
            owner,
            admins,
        );

        self.db.insert_bot(bot, app_config).await?;

        self.update_mini_app_source(bot_id.to_string()).await?;

        Ok(())
    }

    async fn add_mainbot(&self) -> Result<()> {
        if self.db.contains_bot(MAIN_BOT_ID.to_string()).await? {
            return Err(anyhow::anyhow!(
                "Main bot StarDonationService already exists"
            ));
        }
        let api_url = format!("{}{}/", dotenv!("DOMAIN"), MAIN_BOT_ID);
        let tg_api_url = format!("https://api.telegram.org/bot{}/", MAIN_BOT_TOKEN);

        let webhook_url = format!("{api_url}webhook");
        let secret_token = api::generate_secret_token();
        api::set_tg_webhook(&tg_api_url, &webhook_url, &secret_token).await?;

        let bot: Bot = Bot::new(
            MAIN_BOT_ID.to_string(),
            MAIN_BOT_TOKEN.to_string(),
            secret_token,
            MAIN_BOT_OWNER,
            MAIN_BOT_ADMINS.to_vec(),
        );

        self.db.insert_bot(bot, "".to_string()).await?;
        Ok(())
    }

    ///without app_config
    pub async fn update_bot(&self, bot: Bot) -> Result<()> {
        self.db.update_bot(bot.clone()).await?;

        let mut cache = self.cache.lock().await;
        if let Some(cache_bot) = cache.get_mut(&bot.id) {
            *cache_bot = bot;
        }

        Ok(())
    }

    pub async fn update_bot_config(&self, bot_id: String, app_config: String) -> Result<()> {
        self.update_mini_app_source_with_config(&bot_id, &app_config)
            .await?;

        self.db
            .update_bot_config(bot_id.to_string(), app_config.to_string())
            .await?;

        Ok(())
    }

    //todo rename
    //todo move path to env?
    pub async fn update_mini_app_source(&self, bot_id: String) -> Result<()> {
        let path = format!("server/src/mini_app_sources/{}.html", bot_id);
        println!("path: {}", path);

        let html = if bot_id == MAIN_BOT_ID {
            HTML_MAIN_APP.to_string()
        } else {
            let config = self.db.get_bot_config(bot_id).await?;
            HTML_APP.replace(r#"{"json_to_replace":""}"#, &config)
        };

        

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&path)
            .await?;

        println!("html");

        file.write_all(html.as_bytes()).await?;

        Ok(())
    }

    //todo rename
    pub async fn update_mini_app_sources(&self) -> Result<()> {
        let bots_config = self.db.get_bots_config().await?;

        for (bot_id, app_config) in bots_config {
            if bot_id == MAIN_BOT_ID {
                continue;
            }

            let path = format!("server/src/mini_app_sources/{}.html", bot_id);
            let html = HTML_APP.replace(r#"{"json_to_replace":""}"#, &app_config);

            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(&path)
                .await?;

            file.write_all(html.as_bytes()).await?;
        }

        Ok(())
    }

    //todo must be private, now can use in handlers
    async fn update_mini_app_source_with_config(
        &self,
        bot_id: &str,
        app_config: &str,
    ) -> Result<()> {
        if bot_id == MAIN_BOT_ID {
            return Err(anyhow::anyhow!("Cant update main bot app"));
        }

        let path = format!("server/src/mini_app_sources/{}.html", bot_id);

        let html = HTML_APP.replace(r#"{"json_to_replace":""}"#, &app_config);

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&path)
            .await?;
        file.write_all(html.as_bytes()).await?;

        Ok(())
    }

    pub async fn set_menu_button_name(&self, bot_id: &str, name: &str) -> Result<()> {
        let mut tg_api_url = "".to_string();
        let mut button_url = "".to_string();

        self.with_record(bot_id, |bot_row| {
            tg_api_url = format!("https://api.telegram.org/bot{}/", bot_row.token);
            button_url = format!("{}{}/app", dotenv!("DOMAIN"), bot_row.id);
        })
        .await?;

        api::set_menu_button(&tg_api_url, name, &button_url).await?;
        Ok(())
    }

    pub async fn with_record<T>(&self, bot_id: &str, f: impl FnOnce(&Bot) -> T) -> Result<T> {
        let mut cache = self.cache.lock().await;

        if let Some(cache_bot) = cache.get(bot_id) {
            return Ok(f(cache_bot));
        }

        let bot = self.db.get_bot(bot_id.to_string()).await?;

        cache.push(bot_id.to_string(), bot.clone());

        Ok(f(&bot))
    }

    pub async fn get_controlled_bots(&self, user_id: u64) -> Result<json::ControlledBots> {
        let owner_bots = self.db.get_bots_by_owner_id(user_id).await?;
        let admin_bots = self.db.get_bots_by_admin_id(user_id).await?;

        Ok(json::ControlledBots {
            owner_bots,
            admin_bots,
        })
    }
    pub async fn add_bot_admin(&self, user_id: u64, bot_id: &str, admin_id: u64) -> Result<()> {
        let role = self
            .with_record(&bot_id, |bot_row| {
                if bot_row.owner == user_id {
                    // return Ok(UserRole::Owner);
                    return "owner";
                }
                if bot_row.admins.contains(&user_id) {
                    // return Ok(UserRole::Admin);
                    return "admin";
                }
                // return Ok(UserRole::User);
                return "user";
            })
            .await?;

        if role == "owner" || role == "admin" {
            self.db.add_bot_admin(bot_id.to_string(), admin_id).await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Not enough rights"))
        }
    }

    pub async fn remove_bot_admin(&self, user_id: u64, bot_id: &str, admin_id: u64) -> Result<()> {
        let role = self
            .with_record(&bot_id, |bot_row| {
                if admin_id == bot_row.owner {
                    return Err(anyhow::anyhow!("Cant remove owner"));
                }
                if bot_row.owner == user_id {
                    // return Ok(UserRole::Owner);
                    return Ok("owner");
                }
                if bot_row.admins.contains(&user_id) {
                    // return Ok(UserRole::Admin);
                    return Ok("admin");
                }
                // return Ok(UserRole::User);
                return Ok("user");
            })
            .await??;

        if role == "owner" {
            self.db
                .remove_bot_admin(bot_id.to_string(), admin_id)
                .await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Not enough rights"))
        }
    }
}

#[tokio::main]
async fn main() {
    let arc_app_state = Arc::new(AppState::new().await);
    arc_app_state.prepare().await.unwrap();

    //stardonationservice no need /app route cause /:bot_id/app enough
    let app = Router::new()
        .route("/:bot_id/webhook", post(webhook_handler))
        .route("/:bot_id/app", get(mini_app))
        .route("/:bot_id/createInvoice", post(create_invoice))
        .route("/:bot_id/updateConfig", post(update_config))
        //main bot routes
        .route("/stardonationservice/controlledBots", get(controlled_bots))
        .route("/stardonationservice/addBot", post(add_bot))
        .route("/stardonationservice/addBotAdmin", post(add_bot_admin))
        .route(
            "/stardonationservice/removeBotAdmin",
            post(remove_bot_admin),
        )
        // .route("/stardonationservice/removeBot", post(remove_bot))
        //tests
        // .route("/:bot_id/print", get(handler_print_1))
        .route("/stardonationservice/print", get(handler_print_2))
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
