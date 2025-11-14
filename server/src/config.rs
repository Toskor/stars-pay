use anyhow::{anyhow, Result};
use std::num::NonZeroUsize;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    // Telegram Bot Configuration
    pub main_bot_id: String,
    pub main_bot_token: String,
    pub main_bot_owner: u64,
    pub main_bot_admins: Vec<u64>,

    // Server Configuration
    pub domain: String,
    pub ws_domain: String,
    pub port: u16,

    // Database
    pub db_path: String,

    // S3 Configuration
    pub s3_bucket_name: String,
    pub s3_website: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    pub s3_region: String,
    pub s3_endpoint_url: String,

    // Application Settings
    pub cache_size: NonZeroUsize,
    pub room_capacity: usize,
    pub max_stars_debt: f64,
    pub days_since_last_payment_for_block: u64,
    pub days_since_last_payment_for_notification: u64,
    pub procent_for_main_bot: f32,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        // Load .env file if it exists
        dotenv::dotenv().ok();

        // Telegram Bot Configuration
        let main_bot_id = std::env::var("MAIN_BOT_ID")?;
        let main_bot_token = std::env::var("MAIN_BOT_TOKEN")
            .map_err(|_| anyhow!("MAIN_BOT_TOKEN environment variable is required"))?;
        let main_bot_owner = std::env::var("MAIN_BOT_OWNER")?
            .parse()
            .map_err(|_| anyhow!("MAIN_BOT_OWNER must be a valid u64"))?;
        let main_bot_admins = std::env::var("MAIN_BOT_ADMINS")?
            .split(',')
            .map(|s| s.trim().parse::<u64>())
            .collect::<Result<Vec<u64>, _>>()
            .map_err(|_| anyhow!("MAIN_BOT_ADMINS must be comma-separated u64 values"))?;

        // Server Configuration
        let domain = std::env::var("DOMAIN")
            .map_err(|_| anyhow!("DOMAIN environment variable is required"))?;
        let ws_domain = std::env::var("WS_DOMAIN")
            .map_err(|_| anyhow!("WS_DOMAIN environment variable is required"))?;
        let port = std::env::var("PORT")?.parse()?;

        // Database
        let db_path =
            std::env::var("DB_PATH").unwrap_or_else(|_| "db/bots_data_base.sqlite".to_string());

        // S3 Configuration
        let s3_bucket_name = std::env::var("S3_BUCKET_NAME")
            .map_err(|_| anyhow!("S3_BUCKET_NAME environment variable is required"))?;
        let s3_website = std::env::var("S3_WEBSITE")
            .map_err(|_| anyhow!("S3_WEBSITE environment variable is required"))?;
        let s3_access_key = std::env::var("S3_ACCESS_KEY")
            .map_err(|_| anyhow!("S3_ACCESS_KEY environment variable is required"))?;
        let s3_secret_key = std::env::var("S3_SECRET_KEY")
            .map_err(|_| anyhow!("S3_SECRET_KEY environment variable is required"))?;
        let s3_region = std::env::var("S3_REGION")
            .map_err(|_| anyhow!("S3_REGION environment variable is required"))?;
        let s3_endpoint_url = std::env::var("S3_ENDPOINT_URL")
            .map_err(|_| anyhow!("S3_ENDPOINT_URL environment variable is required"))?;

        // Application Settings
        let cache_size = std::env::var("CACHE_SIZE")?
            .parse()
            .ok()
            .and_then(NonZeroUsize::new)
            .unwrap_or_else(|| unsafe { NonZeroUsize::new_unchecked(100) });
        let room_capacity = std::env::var("ROOM_CAPACITY")?.parse().unwrap_or(10);
        let max_stars_debt = std::env::var("MAX_STARS_DEBT")
            .map_err(|_| anyhow!("MAX_STARS_DEBT environment variable is required"))?
            .parse()?;
        let days_since_last_payment_for_block = std::env::var("DAYS_SINCE_LAST_PAYMENT_FOR_BLOCK")?
            .parse()
            .unwrap_or(30);
        let days_since_last_payment_for_notification =
            std::env::var("DAYS_SINCE_LAST_PAYMENT_FOR_NOTIFICATION")?
                .parse()
                .unwrap_or(28);
        let procent_for_main_bot = std::env::var("PROCENT_FOR_MAIN_BOT")?.parse()?;

        Ok(Config {
            main_bot_id,
            main_bot_token,
            main_bot_owner,
            main_bot_admins,
            domain,
            ws_domain,
            port,
            db_path,
            s3_bucket_name,
            s3_website,
            s3_access_key,
            s3_secret_key,
            s3_region,
            s3_endpoint_url,
            cache_size,
            room_capacity,
            max_stars_debt,
            days_since_last_payment_for_block,
            days_since_last_payment_for_notification,
            procent_for_main_bot,
        })
    }
}
