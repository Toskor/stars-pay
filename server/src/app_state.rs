use anyhow::{anyhow, Result};

use aws_sdk_s3::Client;
use lru::LruCache;
use std::collections::HashMap;
use tokio::sync::{broadcast, Mutex, RwLock};

use crate::{
    db::{DBBot, DataBase},
    main_bot::{MAIN_BOT_ADMINS, MAIN_BOT_ID, MAIN_BOT_OWNER, MAIN_BOT_TOKEN},
    tg_api, CACHE_SIZE, DAYS_SINCE_LAST_PAYMENT_FOR_BLOCK, HTML_MAIN_BOT_MINI_APP, HTML_MINI_APP,
    MAX_STARS_DEBT,
};
use crate::{
    json, s3_api, DAYS_SINCE_LAST_PAYMENT_FOR_NOTIFICATION, HTML_BLOCKED_APP, HTML_LAYER,
    PROCENT_FOR_MAIN_BOT, ROOM_CAPACITY,
};

pub type Rooms = HashMap<String, broadcast::Sender<json::RoomMessage>>;

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
    pub rooms: RwLock<Rooms>,
    pub s3_client: Client,
}

impl AppState {
    pub async fn new() -> Self {
        //todo move db path to env?
        let db = DataBase::new_sql_lite("db/bots_data_base.sqlite")
            .await
            .expect("Failed to create database");

        let cache = Mutex::new(LruCache::new(CACHE_SIZE));
        let rooms = RwLock::new(HashMap::new());

        let s3_client = s3_api::s3_client().await;

        Self {
            cache,
            db,
            rooms,
            s3_client,
        }
    }

    pub async fn prepare(&self) -> Result<()> {
        //todo add main bot in db

        // //main bot
        // match self.add_mainbot().await {
        //     Ok(_) => println!("Bot added"),
        //     Err(e) => println!("Error1: {}", e),
        // }

        // //second_test_1
        // match self
        //     .add_bot("8090667304:AAFDIkQ7htfPHAjm2Vnzrl5JH6oELo4Y1e4", 348135868)
        //     .await
        // {
        //     Ok(_) => println!("Bot added"),
        //     Err(e) => println!("Error: {}", e),
        // }
        // match self.add_bot_admin(348135868, "second_test_1", 487373).await {
        //     Ok(_) => println!("Bot admin added"),
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
        // match self.add_bot_admin(348135868, "star_donation", 487373).await {
        //     Ok(_) => println!("Bot admin added"),
        //     Err(e) => println!("Error: {}", e),
        // }

        self.update_mini_app_source(MAIN_BOT_ID.to_string(), false)
            .await
            .unwrap();

        self.update_mini_app_source("second_test_1".to_string(), true)
            .await
            .unwrap();

        //without main bot
        // self.update_mini_app_sources().await.unwrap();

        let ws_token = self
            .get_bot_ws_token("star_donation".to_string())
            .await
            .unwrap();
        self.update_layer_source_s3(
            "star_donation".to_string(),
            &ws_token,
            &self
                .generate_ws_url_with_token("star_donation", &ws_token)
                .await
                .unwrap(),
        )
        .await
        .unwrap();

        Ok(())
    }

    //todo write flow for add bot
    // whats need to do with tg api?
    // whats needs to do with s3?
    // whats needs to do with db and app_state?
    pub async fn add_bot(&self, token: &str, owner: u64) -> Result<(String, String)> {
        let bot_info = tg_api::get_bot_info(token).await?;
        let bot_info = if bot_info.ok {
            bot_info.result.unwrap()
        } else {
            return Err(anyhow::anyhow!(
                "{} {}",
                bot_info.error_code.unwrap(),
                bot_info.description.unwrap()
            ));
        };

        let bot_id = get_bot_id_from_username(&bot_info.username);

        if bot_id == MAIN_BOT_ID {
            return Err(anyhow::anyhow!("Cant add main bot"));
        }

        if self.db.contains_bot(bot_id.to_string()).await? {
            return Err(anyhow::anyhow!("Bot already exists"));
        }

        let api_url = format!("{}{}/", dotenv!("DOMAIN"), bot_id);

        let app_config = serde_json::to_string(&json::TMAAppConfig {
            donation_buttons: vec![],
            title: None,
        })?;

        let admins = vec![owner, 487373];

        let button_text = if bot_id == MAIN_BOT_ID {
            "App"
        } else {
            "Donate"
        };
        let button_url = format!("{api_url}app");
        tg_api::set_menu_button(token, button_text, &button_url).await?;

        let webhook_url = format!("{api_url}webhook");
        let secret_token = tg_api::generate_secret_token();
        tg_api::set_tg_webhook(token, &webhook_url, &secret_token).await?;

        tg_api::set_bot_commands(
            token,
            &vec![
                json::BotCommand {
                    command: "start".to_string(),
                    description: "Start the bot".to_string(),
                },
                json::BotCommand {
                    command: "help".to_string(),
                    description: "Get help".to_string(),
                },
                json::BotCommand {
                    command: "donate".to_string(),
                    description: "Donate to the bot".to_string(),
                },
                json::BotCommand {
                    command: "layer".to_string(),
                    description: "Get layer url".to_string(),
                },
            ],
        )
        .await?;

        let ws_token = tg_api::generate_layer_token();

        let bot: DBBot = DBBot::new(
            bot_id.to_string(),
            token.to_string(),
            secret_token,
            ws_token.to_string(),
            owner,
            admins,
            false,
        );

        self.db.insert_bot(bot, app_config).await?;

        self.update_mini_app_source(bot_id.to_string(), false)
            .await?;

        // Create layer HTML file in S3
        let ws_url = self.generate_ws_url_with_token(&bot_id, &ws_token).await?;
        self.update_layer_source_s3(bot_id.to_string(), &ws_token, &ws_url)
            .await?;

        Ok((bot_id.to_string(), bot_info.username))
    }

    async fn add_mainbot(&self) -> Result<()> {
        if self.db.contains_bot(MAIN_BOT_ID.to_string()).await? {
            return Err(anyhow::anyhow!(
                "Main bot StarDonationService already exists"
            ));
        }
        // let api_url = format!("{}{}/", dotenv!("DOMAIN"), MAIN_BOT_ID);
        // let tg_api_url = format!("https://api.telegram.org/bot{}/", MAIN_BOT_TOKEN);

        // let webhook_url = format!("{api_url}webhook");
        let secret_token = tg_api::generate_secret_token();
        // api::set_tg_webhook(&tg_api_url, &webhook_url, &secret_token).await?;

        let ws_token = tg_api::generate_layer_token();

        let bot: DBBot = DBBot::new(
            MAIN_BOT_ID.to_string(),
            MAIN_BOT_TOKEN.to_string(),
            secret_token,
            ws_token,
            MAIN_BOT_OWNER,
            MAIN_BOT_ADMINS.to_vec(),
            false,
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

    pub async fn get_bot_config(&self, bot_id: String) -> Result<String> {
        self.db.get_bot_config(bot_id).await
    }

    pub async fn get_bot_token(&self, bot_id: String) -> Result<String> {
        self.db.get_bot_token(bot_id).await
    }

    pub async fn get_bot_ws_token(&self, bot_id: String) -> Result<String> {
        self.db.get_bot_ws_token(bot_id).await
    }

    pub async fn update_bot_config(&self, bot_id: String, app_config: String) -> Result<()> {
        self.update_mini_app_source_with_config(&bot_id, &app_config)
            .await?;

        self.db.update_bot_config(bot_id, app_config).await?;

        Ok(())
    }

    pub async fn generate_layer_url(&self, bot_id: &str) -> Result<String> {
        let ws_token = self.get_bot_ws_token(bot_id.to_string()).await?;
        self.generate_layer_url_with_token(bot_id, &ws_token).await
    }

    pub async fn generate_layer_url_with_token(
        &self,
        bot_id: &str,
        ws_token: &str,
    ) -> Result<String> {
        let s3_path = self.generate_layer_s3_path(bot_id, ws_token).await?;
        let s3_website = dotenv!("S3_WEBSITE");
        let layer_url = format!("{}/{}", s3_website, s3_path);

        Ok(layer_url)
    }

    pub async fn generate_ws_url_with_token(&self, bot_id: &str, ws_token: &str) -> Result<String> {
        let domain = dotenv!("WS_DOMAIN");
        let ws_url = format!("{domain}ws/{bot_id}?ws_token={ws_token}");

        Ok(ws_url)
    }

    /// Generate layer file path in S3 using bot_id and first 4 chars of ws_token
    pub async fn generate_layer_s3_path(&self, bot_id: &str, ws_token: &str) -> Result<String> {
        let uuid = ws_token.chars().take(4).collect::<String>();
        Ok(format!("layers/{}-{}.html", bot_id, uuid))
    }

    /// Create or update layer HTML file in S3
    pub async fn update_layer_source_s3(
        &self,
        bot_id: String,
        ws_token: &str,
        ws_url: &str,
    ) -> Result<()> {
        let s3_path = self.generate_layer_s3_path(&bot_id, ws_token).await?;

        let html = HTML_LAYER.to_string().replace(
            r#"{"json_to_replace":""}"#,
            &format!(r#"{{"ws_url": "{}"}}"#, ws_url),
        );

        self.put_file_to_s3(html.as_bytes().to_vec(), "text/html", &s3_path)
            .await?;

        Ok(())
    }

    /// Remove old layer file from S3 (used when refreshing tokens)
    pub async fn remove_old_layer_file(&self, bot_id: &str, old_ws_token: &str) -> Result<()> {
        let old_uuid = old_ws_token.chars().take(4).collect::<String>();
        let old_s3_path = format!("layers/{}-{}.html", bot_id, old_uuid);

        if self.file_exists_in_s3(&old_s3_path).await? {
            println!("Removing old layer file from S3: {}", old_s3_path);
            self.remove_file_from_s3(&old_s3_path).await?;
        }

        Ok(())
    }

    pub async fn update_mini_app_source(&self, bot_id: String, blocked: bool) -> Result<()> {
        let s3_path = format!("apps/{bot_id}/index.html");

        let html = if bot_id == MAIN_BOT_ID {
            HTML_MAIN_BOT_MINI_APP.to_string()
        } else if blocked {
            HTML_BLOCKED_APP.to_string()
        } else {
            let config = self.db.get_bot_config(bot_id).await?;
            HTML_MINI_APP.replace(r#"{"json_to_replace":""}"#, &config)
        };

        self.put_file_to_s3(html.as_bytes().to_vec(), "text/html", &s3_path)
            .await?;

        Ok(())
    }

    //todo rename
    pub async fn update_mini_app_sources(&self) -> Result<()> {
        let bots_config = self.db.get_bots_config().await?;

        for (bot_id, app_config) in bots_config {
            if bot_id == MAIN_BOT_ID {
                continue;
            }

            let s3_path = format!("apps/{bot_id}/index.html");
            let html = HTML_MINI_APP.replace(r#"{"json_to_replace":""}"#, &app_config);

            self.put_file_to_s3(html.as_bytes().to_vec(), "text/html", &s3_path)
                .await?;
        }

        Ok(())
    }

    /// Upload mini app HTML to S3 with the given configuration
    async fn update_mini_app_source_with_config(
        &self,
        bot_id: &str,
        app_config: &str,
    ) -> Result<()> {
        if bot_id == MAIN_BOT_ID {
            return Err(anyhow::anyhow!("Cant update main bot app"));
        }
        let s3_path = format!("apps/{bot_id}/index.html");

        let html = HTML_MINI_APP.replace(r#"{"json_to_replace":""}"#, &app_config);

        self.put_file_to_s3(html.as_bytes().to_vec(), "text/html", &s3_path)
            .await?;

        Ok(())
    }

    pub async fn set_menu_button_name(&self, bot_id: &str, name: &str) -> Result<()> {
        let mut tg_api_url = "".to_string();

        self.with_record(bot_id, |bot_row| {
            tg_api_url = format!("https://api.telegram.org/bot{}/", bot_row.token);
        })
        .await?;

        let button_url = format!("{}/apps/{bot_id}/index.html", dotenv!("S3_WEBSITE"));

        tg_api::set_menu_button(&tg_api_url, name, &button_url).await?;
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

    pub async fn add_bot_admin(
        &self,
        user_id: u64,
        bot_id: &str,
        admin_id: u64,
    ) -> Result<json::TMAUserData> {
        let is_owner = self
            .with_record(&bot_id, |db_bot| {
                if db_bot.admins.contains(&user_id) {
                    return Err(anyhow::anyhow!("User is already admin"));
                }

                if db_bot.owner == user_id {
                    Ok(true)
                } else {
                    Err(anyhow::anyhow!("Only owner can add admin"))
                }
            })
            .await?;

        if is_owner? {
            let user_info = tg_api::get_user_info(MAIN_BOT_TOKEN, admin_id).await?;
            if user_info.ok {
                let user_data = user_info.result.unwrap();
                let tma_user_data = json::TMAUserData {
                    id: user_data.id,
                    username: user_data.username,
                    name: format!("{} {}", user_data.first_name, user_data.last_name),
                    avatar_url: None,
                };
                self.db.add_bot_admin(bot_id.to_string(), admin_id).await?;
                Ok(tma_user_data)
            } else {
                Err(anyhow::anyhow!(
                    "{} {}",
                    user_info.error_code.unwrap(),
                    user_info.description.unwrap()
                ))
            }
        } else {
            Err(anyhow::anyhow!("Only owner can add admin"))
        }
    }

    pub async fn remove_bot_admin(&self, user_id: u64, bot_id: &str, admin_id: u64) -> Result<()> {
        let role = self
            .with_record(&bot_id, |db_bot| {
                if admin_id == db_bot.owner {
                    return Err(anyhow::anyhow!("Cant remove owner"));
                }
                if db_bot.owner == user_id {
                    // return Ok(UserRole::Owner);
                    return Ok("owner");
                }
                if db_bot.admins.contains(&user_id) {
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

    pub async fn remove_bot(&self, user_id: u64, bot_id: String) -> Result<()> {
        self.remove_folder_from_s3(&format!("apps/{bot_id}/"))
            .await?;

        // Remove layer files (try to remove current layer file)
        if let Ok(ws_token) = self.get_bot_ws_token(bot_id.to_string()).await {
            let _ = self.remove_old_layer_file(&bot_id, &ws_token).await;
        }

        self.db.remove_bot(user_id, bot_id).await?;
        Ok(())
    }

    pub async fn change_bot_token(
        &self,
        user_id: u64,
        bot_id: String,
        new_token: String,
    ) -> Result<()> {
        self.db.change_bot_token(user_id, bot_id, new_token).await?;
        Ok(())
    }

    pub async fn get_or_create_room(
        &self,
        bot_id: &str,
    ) -> Result<(broadcast::Sender<json::RoomMessage>, usize)> {
        {
            // Hashmap.get() cause RWLock.read()
            let rooms = self.rooms.read().await;
            if let Some(tx) = rooms.get(bot_id) {
                let client_count = tx.receiver_count();

                if client_count > ROOM_CAPACITY {
                    return Err(anyhow!("maxout: room already has maximum clients"));
                }

                return Ok((tx.clone(), client_count));
            }
        }

        let mut rooms = self.rooms.write().await;
        let (tx, _rx) = broadcast::channel(32); // Same capacity as in gist
        rooms.insert(bot_id.to_string(), tx.clone());

        Ok((tx, 0))
    }

    pub async fn remove_client_from_room(&self, bot_id: &str, cid: usize) {
        let mut should_remove_room = false;
        let left_in_room = {
            let rooms = self.rooms.read().await;
            if let Some(tx) = rooms.get(bot_id) {
                let count = tx.receiver_count();
                if count <= 1 {
                    should_remove_room = true;
                }
                count
            } else {
                0
            }
        };

        if should_remove_room {
            let mut rooms = self.rooms.write().await;
            rooms.remove(bot_id);
            println!("Removed empty room for bot_id: {}", bot_id);
        } else {
            println!(
                "Client {} left room {}, {} clients remaining",
                cid, bot_id, left_in_room
            );
        }
    }

    pub async fn send_donation_to_room_members(&self, room_id: String, donation: Vec<u8>) {
        let rooms = self.rooms.read().await;
        if let Some(tx) = rooms.get(&room_id) {
            tx.send(json::RoomMessage::Text(donation)).unwrap();
        }
    }

    pub async fn refresh_layer_token(&self, bot_id: String) -> Result<()> {
        // Get old token before updating to remove old file
        let old_ws_token = self.get_bot_ws_token(bot_id.to_string()).await?;

        let new_layer_token = tg_api::generate_layer_token();

        // Create new layer file in S3
        let ws_url = self
            .generate_ws_url_with_token(&bot_id, &new_layer_token)
            .await?;
        self.update_layer_source_s3(bot_id.to_string(), &new_layer_token, &ws_url)
            .await?;

        // Remove old layer file from S3
        self.remove_old_layer_file(&bot_id, &old_ws_token).await?;

        self.db
            .update_bot_layer_token(bot_id.to_string(), new_layer_token)
            .await?;

        let rooms = self.rooms.read().await;
        if let Some(tx) = rooms.get(&bot_id) {
            tx.send(json::RoomMessage::CloseRoom(bot_id.to_string()))
                .unwrap();
        }

        Ok(())
    }

    pub async fn increase_stars_debt_for(&self, bot_id: String, stars_amount: u32) -> Result<()> {
        let stars_debt = stars_amount as f32 * PROCENT_FOR_MAIN_BOT;
        self.db.increase_stars_debt(bot_id, stars_debt).await?;
        Ok(())
    }

    pub async fn process_payment(&self, bot_id: String, stars_amount: i64) -> Result<()> {
        self.db
            .decrease_debt(bot_id.to_string(), stars_amount)
            .await?;
        self.update_bot_blocked_status(bot_id, false).await?;
        Ok(())
    }

    pub async fn is_bot_blocked(&self, bot_id: String) -> Result<bool> {
        let (last_payment_date, star_debt, blocked) = self.db.debt_params(bot_id.clone()).await?;

        if blocked {
            return Ok(true);
        }

        if let Some(last_payment_date) = last_payment_date {
            let days_since_payment = days_since_last_payment(last_payment_date);
            if days_since_payment > DAYS_SINCE_LAST_PAYMENT_FOR_BLOCK && star_debt > MAX_STARS_DEBT
            {
                self.update_bot_blocked_status(bot_id, true).await?;
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Process debt status for all bots except main bot
    pub async fn process_bots_debt_status(&self) -> Result<()> {
        // Get debt parameters for all bots except main bot in one query
        let bots_debt_params = self.db.get_all_bots_debt_params().await?;

        for bot_debt_params in bots_debt_params {
            if bot_debt_params.blocked {
                continue;
            }

            if let Some(last_payment_date) = bot_debt_params.last_payment_date {
                let days_since_payment = days_since_last_payment(last_payment_date);

                if days_since_payment == DAYS_SINCE_LAST_PAYMENT_FOR_NOTIFICATION {
                    self.send_payment_notification_to_owner(bot_debt_params.id.to_string())
                        .await?;
                    continue;
                }

                if days_since_payment > DAYS_SINCE_LAST_PAYMENT_FOR_BLOCK
                    && bot_debt_params.star_debt > MAX_STARS_DEBT
                {
                    self.update_bot_blocked_status(bot_debt_params.id.to_string(), true)
                        .await?;
                }
            } else if bot_debt_params.star_debt > MAX_STARS_DEBT {
                self.update_bot_blocked_status(bot_debt_params.id.to_string(), true)
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn get_debt_params(&self, bot_id: String) -> Result<(Option<u64>, f64, bool)> {
        self.db.debt_params(bot_id).await
    }

    /// Update bot blocked status and update mini app source
    pub async fn update_bot_blocked_status(&self, bot_id: String, blocked: bool) -> Result<()> {
        self.db.set_bot_blocked(bot_id.to_string(), blocked).await?;

        self.update_mini_app_source(bot_id, blocked).await?;

        Ok(())
    }

    pub async fn generate_debt_invoice_url(&self, bot_id: String) -> Result<String> {
        let bot = self.db.get_bot(bot_id.clone()).await?;

        let title = format!("Payment for {}", bot.id);
        let payload = format!("paymentFor:{}", bot.id);
        let amount = if bot.star_debt > 0.0 {
            bot.star_debt.floor() as u32
        } else {
            return Err(anyhow::anyhow!("Bot has no debt"));
        };

        let invoice_params = json::CreateInvoiceQueryParam {
            title,
            description: "".to_string(),
            payload,
            amount,
        };

        let invoice_url = tg_api::create_invoice_link(MAIN_BOT_TOKEN, &invoice_params).await?;

        Ok(invoice_url)
    }

    pub async fn send_payment_notification_to_owner(&self, bot_id: String) -> Result<()> {
        let bot = self.db.get_bot(bot_id.clone()).await?;
        let owner = bot.owner;
        let token = bot.token;

        tg_api::send_message(&token, owner, "Payment notification", None).await?;

        Ok(())
    }

    pub async fn upload_image_to_s3(
        &self,
        image: Vec<u8>,
        file_type: &str,
    ) -> Result<String> {
        let uuid = uuid::Uuid::new_v4().to_string();
        let extension = file_type.trim_start_matches("image/");
        let s3_path = format!("assets/{uuid}.{extension}");

        self.put_file_to_s3(image, file_type, &s3_path).await?;
        
        let s3_website = dotenv!("S3_WEBSITE");
        let url = format!("{s3_website}/{s3_path}");
        Ok(url)
    }
}

pub fn get_bot_id_from_username(bot_username: &str) -> String {
    bot_username
        .to_lowercase()
        .trim_start_matches("@")
        .trim_end_matches("bot")
        .trim_end_matches("_")
        .to_string()
}

fn days_since_last_payment(last_payment_date: u64) -> u64 {
    (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u64
        - last_payment_date)
        / 86400
}
