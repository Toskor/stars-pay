//! PostgreSQL-backed [`BotStore`].
//!
//! Uses a `deadpool-postgres` connection pool over `tokio-postgres`. Telegram
//! ids are `u64` in the domain model but always fit in `i64`, so they're stored
//! as `BIGINT` and cast at the boundary. The `admins` list is a native
//! `BIGINT[]` column (`$1 = ANY(admins)`), avoiding the JSON-in-text scan the
//! SQLite store falls back to.
//!
//! TLS is intentionally omitted (`NoTls`) — this backend targets a local/
//! sidecar Postgres for the demo; production would swap in a rustls connector.

use anyhow::{Context, Result};
use async_trait::async_trait;
use deadpool_postgres::{Config, Pool, Runtime};
use tokio_postgres::{NoTls, Row};

use super::{BotDebtParams, BotStore, DBBot};

pub struct PostgresStore {
    pool: Pool,
}

impl PostgresStore {
    pub async fn connect(url: &str) -> Result<Self> {
        let mut cfg = Config::new();
        cfg.url = Some(url.to_string());
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1), NoTls)
            .map_err(|e| anyhow::anyhow!("failed to build postgres pool: {e}"))?;

        let store = Self { pool };
        store.init_schema().await?;
        Ok(store)
    }

    async fn init_schema(&self) -> Result<()> {
        let client = self.pool.get().await.context("pg pool checkout")?;
        client
            .batch_execute(
                "CREATE TABLE IF NOT EXISTS bots (
                    id                TEXT PRIMARY KEY,
                    token             TEXT NOT NULL,
                    secret_token      TEXT NOT NULL,
                    ws_token          TEXT NOT NULL,
                    owner             BIGINT NOT NULL,
                    admins            BIGINT[] NOT NULL DEFAULT '{}',
                    last_payment_date BIGINT,
                    star_debt         DOUBLE PRECISION NOT NULL DEFAULT 0,
                    blocked           BOOLEAN NOT NULL DEFAULT FALSE
                );
                CREATE TABLE IF NOT EXISTS configs (
                    id          TEXT PRIMARY KEY,
                    app_config  TEXT NOT NULL,
                    goal_config TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS bots_admins_gin ON bots USING GIN (admins);",
            )
            .await
            .context("init postgres schema")?;
        Ok(())
    }

    async fn owner_of(&self, bot_id: &str) -> Result<u64> {
        let client = self.pool.get().await.context("pg pool checkout")?;
        let row = client
            .query_one("SELECT owner FROM bots WHERE id = $1", &[&bot_id])
            .await
            .context("select owner")?;
        Ok(row.get::<_, i64>(0) as u64)
    }
}

const BOT_COLUMNS: &str =
    "id, token, secret_token, ws_token, owner, admins, last_payment_date, star_debt, blocked";

fn row_to_bot(row: &Row) -> DBBot {
    let owner: i64 = row.get("owner");
    let admins: Vec<i64> = row.get("admins");
    let last_payment_date: Option<i64> = row.get("last_payment_date");
    DBBot {
        id: row.get("id"),
        token: row.get("token"),
        secret_token: row.get("secret_token"),
        ws_token: row.get("ws_token"),
        owner: owner as u64,
        admins: admins.into_iter().map(|a| a as u64).collect(),
        last_payment_date: last_payment_date.map(|t| t as u64),
        star_debt: row.get("star_debt"),
        blocked: row.get("blocked"),
    }
}

#[async_trait]
impl BotStore for PostgresStore {
    async fn get_bots_by_admin_id(&self, admin_id: u64) -> Result<Vec<DBBot>> {
        let client = self.pool.get().await?;
        let sql = format!("SELECT {BOT_COLUMNS} FROM bots WHERE $1 = ANY(admins)");
        let rows = client.query(&sql, &[&(admin_id as i64)]).await?;

        // Mirror the SQLite store: skip bots whose admin list also contains the
        // owner, and never surface the owner inside the admins list.
        let bots = rows
            .iter()
            .map(row_to_bot)
            .filter(|bot| !bot.admins.contains(&bot.owner))
            .map(|mut bot| {
                bot.admins.retain(|&id| id != bot.owner);
                bot
            })
            .collect();
        Ok(bots)
    }

    async fn get_bots_by_owner_id(&self, owner_id: u64) -> Result<Vec<DBBot>> {
        let client = self.pool.get().await?;
        let sql = format!("SELECT {BOT_COLUMNS} FROM bots WHERE owner = $1");
        let rows = client.query(&sql, &[&(owner_id as i64)]).await?;
        let bots = rows
            .iter()
            .map(row_to_bot)
            .map(|mut bot| {
                bot.admins.retain(|&id| id != owner_id);
                bot
            })
            .collect();
        Ok(bots)
    }

    async fn get_bot(&self, bot_id: String) -> Result<DBBot> {
        let client = self.pool.get().await?;
        let sql = format!("SELECT {BOT_COLUMNS} FROM bots WHERE id = $1");
        let row = client.query_one(&sql, &[&bot_id]).await?;
        Ok(row_to_bot(&row))
    }

    async fn insert_bot(&self, bot: DBBot, app_config: String, goal_config: String) -> Result<()> {
        let mut client = self.pool.get().await?;
        let tx = client.transaction().await?;

        let admins: Vec<i64> = bot.admins.iter().map(|&a| a as i64).collect();
        let last_payment_date = bot.last_payment_date.map(|t| t as i64);
        tx.execute(
            "INSERT INTO bots (id, token, secret_token, ws_token, owner, admins, last_payment_date, star_debt, blocked)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            &[
                &bot.id,
                &bot.token,
                &bot.secret_token,
                &bot.ws_token,
                &(bot.owner as i64),
                &admins,
                &last_payment_date,
                &bot.star_debt,
                &bot.blocked,
            ],
        )
        .await?;
        tx.execute(
            "INSERT INTO configs (id, app_config, goal_config) VALUES ($1, $2, $3)",
            &[&bot.id, &app_config, &goal_config],
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn update_bot(&self, bot: DBBot) -> Result<()> {
        let client = self.pool.get().await?;
        let admins: Vec<i64> = bot.admins.iter().map(|&a| a as i64).collect();
        let last_payment_date = bot.last_payment_date.map(|t| t as i64);
        client
            .execute(
                "UPDATE bots SET token = $2, secret_token = $3, ws_token = $4, owner = $5,
                 admins = $6, last_payment_date = $7, star_debt = $8, blocked = $9 WHERE id = $1",
                &[
                    &bot.id,
                    &bot.token,
                    &bot.secret_token,
                    &bot.ws_token,
                    &(bot.owner as i64),
                    &admins,
                    &last_payment_date,
                    &bot.star_debt,
                    &bot.blocked,
                ],
            )
            .await?;
        Ok(())
    }

    async fn update_app_config(&self, bot_id: String, app_config: String) -> Result<()> {
        let client = self.pool.get().await?;
        client
            .execute(
                "UPDATE configs SET app_config = $2 WHERE id = $1",
                &[&bot_id, &app_config],
            )
            .await?;
        Ok(())
    }

    async fn update_goal_config(&self, bot_id: String, goal_config: String) -> Result<()> {
        let client = self.pool.get().await?;
        client
            .execute(
                "UPDATE configs SET goal_config = $2 WHERE id = $1",
                &[&bot_id, &goal_config],
            )
            .await?;
        Ok(())
    }

    async fn contains_bot(&self, bot_id: String) -> Result<bool> {
        let client = self.pool.get().await?;
        let row = client
            .query_one(
                "SELECT EXISTS(SELECT 1 FROM bots WHERE id = $1)",
                &[&bot_id],
            )
            .await?;
        Ok(row.get(0))
    }

    async fn get_app_configs(&self) -> Result<Vec<(String, String)>> {
        let client = self.pool.get().await?;
        let rows = client
            .query("SELECT id, app_config FROM configs", &[])
            .await?;
        Ok(rows.iter().map(|r| (r.get(0), r.get(1))).collect())
    }

    async fn get_app_config(&self, bot_id: String) -> Result<String> {
        let client = self.pool.get().await?;
        let row = client
            .query_one("SELECT app_config FROM configs WHERE id = $1", &[&bot_id])
            .await?;
        Ok(row.get(0))
    }

    async fn get_goal_config(&self, bot_id: String) -> Result<String> {
        let client = self.pool.get().await?;
        let row = client
            .query_one("SELECT goal_config FROM configs WHERE id = $1", &[&bot_id])
            .await?;
        Ok(row.get(0))
    }

    async fn get_bot_token(&self, bot_id: String) -> Result<String> {
        let client = self.pool.get().await?;
        let row = client
            .query_one("SELECT token FROM bots WHERE id = $1", &[&bot_id])
            .await?;
        Ok(row.get(0))
    }

    async fn get_bot_ws_token(&self, bot_id: String) -> Result<String> {
        let client = self.pool.get().await?;
        let row = client
            .query_one("SELECT ws_token FROM bots WHERE id = $1", &[&bot_id])
            .await?;
        Ok(row.get(0))
    }

    async fn add_bot_admin(&self, bot_id: String, admin_id: u64) -> Result<()> {
        let client = self.pool.get().await?;
        client
            .execute(
                "UPDATE bots SET admins = array_append(admins, $2) \
                 WHERE id = $1 AND NOT ($2 = ANY(admins))",
                &[&bot_id, &(admin_id as i64)],
            )
            .await?;
        Ok(())
    }

    async fn remove_bot_admin(&self, bot_id: String, admin_id: u64) -> Result<()> {
        let client = self.pool.get().await?;
        client
            .execute(
                "UPDATE bots SET admins = array_remove(admins, $2) WHERE id = $1",
                &[&bot_id, &(admin_id as i64)],
            )
            .await?;
        Ok(())
    }

    async fn remove_bot(&self, user_id: u64, bot_id: String) -> Result<()> {
        if self.owner_of(&bot_id).await? != user_id {
            return Err(anyhow::anyhow!("Only owner can delete bot"));
        }
        let mut client = self.pool.get().await?;
        let tx = client.transaction().await?;
        tx.execute("DELETE FROM bots WHERE id = $1", &[&bot_id])
            .await?;
        tx.execute("DELETE FROM configs WHERE id = $1", &[&bot_id])
            .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn change_bot_token(
        &self,
        user_id: u64,
        bot_id: String,
        new_token: String,
    ) -> Result<()> {
        if self.owner_of(&bot_id).await? != user_id {
            return Err(anyhow::anyhow!("Only owner can change bot token"));
        }
        let client = self.pool.get().await?;
        client
            .execute(
                "UPDATE bots SET token = $2 WHERE id = $1",
                &[&bot_id, &new_token],
            )
            .await?;
        Ok(())
    }

    async fn update_bot_layer_token(&self, bot_id: String, layer_token: String) -> Result<()> {
        let client = self.pool.get().await?;
        client
            .execute(
                "UPDATE bots SET ws_token = $2 WHERE id = $1",
                &[&bot_id, &layer_token],
            )
            .await?;
        Ok(())
    }

    async fn increase_stars_debt(&self, bot_id: String, stars_amount: f32) -> Result<()> {
        let client = self.pool.get().await?;
        client
            .execute(
                "UPDATE bots SET star_debt = star_debt + $2 WHERE id = $1",
                &[&bot_id, &(stars_amount as f64)],
            )
            .await?;
        Ok(())
    }

    async fn decrease_debt(&self, bot_id: String, stars_amount: i64) -> Result<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let client = self.pool.get().await?;
        client
            .execute(
                "UPDATE bots SET last_payment_date = $2, star_debt = star_debt - $3 WHERE id = $1",
                &[&bot_id, &timestamp, &stars_amount],
            )
            .await?;
        Ok(())
    }

    async fn set_bot_blocked(&self, bot_id: String, blocked: bool) -> Result<()> {
        let client = self.pool.get().await?;
        client
            .execute(
                "UPDATE bots SET blocked = $2 WHERE id = $1",
                &[&bot_id, &blocked],
            )
            .await?;
        Ok(())
    }

    async fn debt_params(&self, bot_id: String) -> Result<(Option<u64>, f64, bool)> {
        let client = self.pool.get().await?;
        let row = client
            .query_one(
                "SELECT last_payment_date, star_debt, blocked FROM bots WHERE id = $1",
                &[&bot_id],
            )
            .await?;
        let last_payment_date: Option<i64> = row.get(0);
        Ok((last_payment_date.map(|t| t as u64), row.get(1), row.get(2)))
    }

    async fn get_all_bots_debt_params(&self, main_bot_id: &str) -> Result<Vec<BotDebtParams>> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT id, last_payment_date, star_debt, blocked FROM bots WHERE id <> $1",
                &[&main_bot_id],
            )
            .await?;
        let params = rows
            .iter()
            .map(|row| {
                let last_payment_date: Option<i64> = row.get(1);
                BotDebtParams {
                    id: row.get(0),
                    last_payment_date: last_payment_date.map(|t| t as u64),
                    star_debt: row.get(2),
                    blocked: row.get(3),
                }
            })
            .collect();
        Ok(params)
    }
}
