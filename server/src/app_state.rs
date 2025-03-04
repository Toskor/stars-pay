use anyhow::Result;
use lru::LruCache;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use tokio::sync::Mutex;

use crate::{api, db::{DBBot, DataBase}, main_bot::{MAIN_BOT_ADMINS, MAIN_BOT_ID, MAIN_BOT_OWNER, MAIN_BOT_TOKEN}, CACHE_SIZE, HTML_MAIN_BOT_MINI_APP, HTML_MINI_APP};

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

//for AppState
#[derive(Debug)]
pub struct ControlledBots {
    pub owner_bots: Vec<DBBot>,
    pub admin_bots: Vec<DBBot>,
}

//todo sometimes long delay in webhook (mb when change webhook url)
pub struct AppState {
    pub cache: Mutex<LruCache<String, DBBot>>,
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
        // match self
        //     .add_bot("8090667304:AAFDIkQ7htfPHAjm2Vnzrl5JH6oELo4Y1e4", 348135868)
        //     .await
        // {
        //     Ok(_) => println!("Bot added"),
        //     Err(e) => println!("Error: {}", e),
        // }

        //star_donation
        // match self
        //     .add_bot("7792542554:AAEVkmVbOKN3ouDPJORrfNZIX2j4uMlEZHs", 348135868)
        //     .await
        // {
        //     Ok(_) => println!("Bot added"),
        //     Err(e) => println!("Error: {}", e),
        // }

        self.update_mini_app_source(MAIN_BOT_ID.to_string())
            .await
            .unwrap();

        //without main bot
        self.update_mini_app_sources().await.unwrap();
        Ok(())
    }

    //todo add new bot:  set cmd,
    pub async fn add_bot(&self, token: &str, owner: u64) -> Result<(u64, String)> {
        let bot_info = api::get_bot_info(token).await?;
        let bot_info = if bot_info.ok {
            bot_info.result.unwrap()
        } else {
            return Err(anyhow::anyhow!(
                "{} {}",
                bot_info.error_code.unwrap(),
                bot_info.description.unwrap()
            ));
        };

        let name = bot_info.username.to_lowercase();
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
        api::set_menu_button(token, button_text, &button_url).await?;

        let webhook_url = format!("{api_url}webhook");
        let secret_token = api::generate_secret_token();
        api::set_tg_webhook(token, &webhook_url, &secret_token).await?;

        let bot: DBBot = DBBot::new(
            bot_id.to_string(),
            token.to_string(),
            secret_token,
            owner,
            admins,
        );

        self.db.insert_bot(bot, app_config).await?;

        self.update_mini_app_source(bot_id.to_string()).await?;

        Ok((bot_info.id, bot_info.username))
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

        let bot: DBBot = DBBot::new(
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
    pub async fn update_bot(&self, bot: DBBot) -> Result<()> {
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
            HTML_MAIN_BOT_MINI_APP.to_string()
        } else {
            let config = self.db.get_bot_config(bot_id).await?;
            HTML_MINI_APP.replace(r#"{"json_to_replace":""}"#, &config)
        };

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&path)
            .await?;

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
            let html = HTML_MINI_APP.replace(r#"{"json_to_replace":""}"#, &app_config);

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

        let html = HTML_MINI_APP.replace(r#"{"json_to_replace":""}"#, &app_config);

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

    pub async fn with_record<T>(&self, bot_id: &str, f: impl FnOnce(&DBBot) -> T) -> Result<T> {
        let mut cache = self.cache.lock().await;

        if let Some(cache_bot) = cache.get(bot_id) {
            return Ok(f(cache_bot));
        }

        let bot = self.db.get_bot(bot_id.to_string()).await?;

        cache.push(bot_id.to_string(), bot.clone());

        Ok(f(&bot))
    }

    pub async fn get_controlled_bots(&self, user_id: u64) -> Result<ControlledBots> {
        let owner_bots = self.db.get_bots_by_owner_id(user_id).await?;
        let admin_bots = self.db.get_bots_by_admin_id(user_id).await?;

        Ok(ControlledBots {
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