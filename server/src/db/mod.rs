//! Persistence layer.
//!
//! [`BotStore`] is the storage abstraction used by the rest of the app. Two
//! implementations exist:
//!
//! - [`sqlite::SqliteStore`] — embedded, the default, always compiled.
//! - [`postgres::PostgresStore`] — `deadpool-postgres` pool + `tokio-postgres`,
//!   behind the `postgres` feature.
//!
//! [`connect`] picks one at startup based on config, returning a trait object
//! so handlers depend only on `BotStore`, never on a concrete backend.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use crate::config::Config;

pub mod sqlite;

#[cfg(feature = "postgres")]
pub mod postgres;

/// Persisted bot record.
#[derive(Debug, Clone)]
pub struct DBBot {
    /// Bot username without the trailing "bot".
    pub id: String,
    pub token: String,
    pub secret_token: String,
    pub ws_token: String,
    pub owner: u64,
    pub admins: Vec<u64>,
    pub last_payment_date: Option<u64>,
    pub star_debt: f64,
    pub blocked: bool,
}

impl DBBot {
    pub fn new(
        id: String,
        token: String,
        secret_token: String,
        ws_token: String,
        owner: u64,
        admins: Vec<u64>,
        blocked: bool,
    ) -> Self {
        DBBot {
            id,
            token,
            secret_token,
            ws_token,
            owner,
            admins,
            last_payment_date: None,
            star_debt: 0.0,
            blocked,
        }
    }
}

/// Subset of a bot's fields needed by the daily debt-status sweep.
#[derive(Debug, Clone)]
pub struct BotDebtParams {
    pub id: String,
    pub last_payment_date: Option<u64>,
    pub star_debt: f64,
    pub blocked: bool,
}

/// Storage backend for bots and their per-bot config.
///
/// Implementations must be cheap to share behind an `Arc` and safe to call
/// concurrently from many tasks (`Send + Sync`).
#[async_trait]
pub trait BotStore: Send + Sync {
    async fn get_bots_by_admin_id(&self, admin_id: u64) -> Result<Vec<DBBot>>;
    async fn get_bots_by_owner_id(&self, owner_id: u64) -> Result<Vec<DBBot>>;
    async fn get_bot(&self, bot_id: String) -> Result<DBBot>;
    async fn insert_bot(&self, bot: DBBot, app_config: String, goal_config: String) -> Result<()>;
    async fn update_bot(&self, bot: DBBot) -> Result<()>;
    async fn update_app_config(&self, bot_id: String, app_config: String) -> Result<()>;
    async fn update_goal_config(&self, bot_id: String, goal_config: String) -> Result<()>;
    async fn contains_bot(&self, bot_id: String) -> Result<bool>;
    async fn get_app_configs(&self) -> Result<Vec<(String, String)>>;
    async fn get_app_config(&self, bot_id: String) -> Result<String>;
    async fn get_goal_config(&self, bot_id: String) -> Result<String>;
    async fn get_bot_token(&self, bot_id: String) -> Result<String>;
    async fn get_bot_ws_token(&self, bot_id: String) -> Result<String>;
    async fn add_bot_admin(&self, bot_id: String, admin_id: u64) -> Result<()>;
    async fn remove_bot_admin(&self, bot_id: String, admin_id: u64) -> Result<()>;
    async fn remove_bot(&self, user_id: u64, bot_id: String) -> Result<()>;
    async fn change_bot_token(&self, user_id: u64, bot_id: String, new_token: String)
        -> Result<()>;
    async fn update_bot_layer_token(&self, bot_id: String, layer_token: String) -> Result<()>;
    async fn increase_stars_debt(&self, bot_id: String, stars_amount: f32) -> Result<()>;
    async fn decrease_debt(&self, bot_id: String, stars_amount: i64) -> Result<()>;
    async fn set_bot_blocked(&self, bot_id: String, blocked: bool) -> Result<()>;
    async fn debt_params(&self, bot_id: String) -> Result<(Option<u64>, f64, bool)>;
    async fn get_all_bots_debt_params(&self, main_bot_id: &str) -> Result<Vec<BotDebtParams>>;
}

/// Open the configured store. Uses Postgres when `DATABASE_URL` is set and the
/// `postgres` feature is built; otherwise falls back to SQLite.
pub async fn connect(config: &Config) -> Result<Arc<dyn BotStore>> {
    #[cfg(feature = "postgres")]
    if let Some(url) = config.database_url.as_deref() {
        tracing::info!("using PostgreSQL store");
        let store = postgres::PostgresStore::connect(url).await?;
        return Ok(Arc::new(store));
    }

    #[cfg(not(feature = "postgres"))]
    if config.database_url.is_some() {
        tracing::warn!("DATABASE_URL is set but the `postgres` feature is not built; using SQLite");
    }

    tracing::info!(path = %config.db_path, "using SQLite store");
    let store = sqlite::SqliteStore::open(&config.db_path).await?;
    Ok(Arc::new(store))
}
