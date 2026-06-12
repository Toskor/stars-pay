//! SQLite-backed [`BotStore`]. Bots and per-bot config live in two tables;
//! `admins` is a JSON array column (good enough at this scale — see the
//! roadmap for the normalized-table alternative).

use anyhow::Result;
use async_rusqlite::{rusqlite::named_params, Connection};
use async_trait::async_trait;
use rusqlite::functions::FunctionFlags;

use super::{BotDebtParams, BotStore, DBBot};

pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    pub async fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path).await?;

        conn.call(move |conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS bots (
                id                  TEXT PRIMARY KEY NOT NULL,
                token               TEXT NOT NULL,
                secret_token        TEXT NOT NULL,
                ws_token            TEXT NOT NULL,
                owner               INTEGER NOT NULL,
                admins              TEXT NOT NULL,
                last_payment_date   INTEGER,
                star_debt           REAL NOT NULL DEFAULT 0,
                blocked             BOOLEAN NOT NULL DEFAULT 0
            )",
                (),
            )
        })
        .await?;

        conn.call(move |conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS configs (
                id                  TEXT PRIMARY KEY NOT NULL,
                app_config          TEXT NOT NULL,
                goal_config         TEXT NOT NULL
            )",
                (),
            )
        })
        .await?;

        conn.call(move |conn| {
            conn.create_scalar_function(
                "admins_contains",
                2,
                FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
                |ctx| {
                    let json_text: String = ctx.get(0)?;
                    let key: u64 = ctx.get(1)?;
                    let value: Vec<u64> = serde_json::from_str(&json_text).unwrap_or(vec![]);
                    Ok(value.contains(&key))
                },
            )
        })
        .await?;

        conn.call(move |conn| {
            conn.execute(
                "CREATE INDEX IF NOT EXISTS admins_contains ON bots ( admins_contains(admins, 1) )",
                (),
            )
        })
        .await?;

        Ok(Self { conn })
    }
}

#[async_trait]
impl BotStore for SqliteStore {
    async fn get_bots_by_admin_id(&self, admin_id: u64) -> Result<Vec<DBBot>> {
        let conn = &self.conn;
        let search_pattern = format!("%{}%", admin_id);

        let bots = conn
            .call(move |conn| {
                let mut stmt = conn.prepare_cached(
                    "SELECT id, token, secret_token, ws_token, owner, admins, last_payment_date, star_debt, blocked FROM bots
                    WHERE admins LIKE :search_pattern",
                )?;
                let query_result =
                    stmt.query_map(named_params! { ":search_pattern": search_pattern }, |row| {
                        let id: String = row.get(0)?;
                        let token: String = row.get(1)?;
                        let secret_token: String = row.get(2)?;
                        let ws_token: String = row.get(3)?;
                        let owner: u64 = row.get(4)?;
                        let admins_json = row.get_ref(5)?.as_str()?;
                        let admins: Vec<u64> =
                            serde_json::from_str(admins_json).map_err(map_serde_err)?;
                        let last_payment_date: Option<u64> = row.get(6)?;
                        let star_debt: f64 = row.get(7)?;
                        let blocked: bool = row.get(8)?;

                        if !admins.contains(&owner) {
                            Ok(Some(DBBot {
                                id,
                                token,
                                secret_token,
                                ws_token,
                                owner,
                                admins: admins.into_iter().filter(|&id| id != owner).collect(),
                                last_payment_date,
                                star_debt,
                                blocked,
                            }))
                        } else {
                            Ok(None)
                        }
                    })?;

                let mut bots: Vec<DBBot> = vec![];
                for bot_option in query_result.into_iter() {
                    if let Some(bot) = bot_option? {
                        bots.push(bot);
                    }
                }
                Ok::<Vec<DBBot>, async_rusqlite::Error>(bots)
            })
            .await?;
        Ok(bots)
    }

    async fn get_bots_by_owner_id(&self, owner_id: u64) -> Result<Vec<DBBot>> {
        let conn = &self.conn;
        let bots = conn
            .call(move |conn| {
                let mut stmt =
                    conn.prepare_cached("SELECT id, token, secret_token, ws_token, owner, admins, last_payment_date, star_debt, blocked FROM bots WHERE owner = :owner_id")?;
                let bots_map = stmt.query_map(named_params! { ":owner_id": owner_id }, |row| {
                    let id: String = row.get(0)?;
                    let token: String = row.get(1)?;
                    let secret_token: String = row.get(2)?;
                    let ws_token: String = row.get(3)?;
                    let owner: u64 = row.get(4)?;

                    let admins_json = row.get_ref(5)?.as_str()?;
                    let admins: Vec<u64> =
                        serde_json::from_str(admins_json).map_err(map_serde_err)?;
                    let admins = admins.into_iter().filter(|&id| id != owner_id).collect();

                    let last_payment_date: Option<u64> = row.get(6)?;
                    let star_debt: f64 = row.get(7)?;
                    let blocked: bool = row.get(8)?;

                    Ok(DBBot { id, token, secret_token, ws_token, owner, admins, last_payment_date, star_debt, blocked })
                })?;
                let mut bots: Vec<DBBot> = vec![];
                for bot in bots_map.into_iter() {
                    bots.push(bot?);
                }
                Ok::<Vec<DBBot>, async_rusqlite::Error>(bots)
            })
            .await?;
        Ok(bots)
    }

    async fn get_bot(&self, bot_id: String) -> Result<DBBot> {
        let conn = &self.conn;

        let bot = conn
            .call(move |conn| {
                let mut stmt = conn.prepare_cached(
                    "SELECT id, token, secret_token, ws_token, owner, admins, last_payment_date, star_debt, blocked FROM bots WHERE id = ?",
                )?;
                let bot = stmt.query_row([bot_id], |row| {
                    let id: String = row.get(0)?;
                    let token: String = row.get(1)?;
                    let secret_token: String = row.get(2)?;
                    let ws_token: String = row.get(3)?;
                    let owner: u64 = row.get(4)?;
                    let admins_json = row.get_ref(5)?.as_str()?;
                    let admins: Vec<u64> =
                        serde_json::from_str(admins_json).map_err(map_serde_err)?;
                    let last_payment_date: Option<u64> = row.get(6)?;
                    let star_debt: f64 = row.get(7)?;
                    let blocked: bool = row.get(8)?;

                    Ok(DBBot {
                        id,
                        token,
                        secret_token,
                        ws_token,
                        owner,
                        admins,
                        last_payment_date,
                        star_debt,
                        blocked,
                    })
                })?;

                Ok::<_, async_rusqlite::Error>(bot)
            })
            .await?;

        Ok(bot)
    }

    async fn insert_bot(&self, bot: DBBot, app_config: String, goal_config: String) -> Result<()> {
        let conn = &self.conn;

        let admins_json = serde_json::to_string(&bot.admins)?;

        conn.call(move |conn| {
            let tx = conn.transaction()?;
            tx.execute(
                "INSERT INTO bots (id, token, secret_token, ws_token, owner, admins, last_payment_date, star_debt, blocked) VALUES (:id, :token, :secret_token, :ws_token, :owner, :admins, :last_payment_date, :star_debt, :blocked)",
                named_params! {
                    ":id": &bot.id,
                    ":token": &bot.token,
                    ":secret_token": &bot.secret_token,
                    ":ws_token": &bot.ws_token,
                    ":owner": &bot.owner,
                    ":admins": &admins_json,
                    ":last_payment_date": &bot.last_payment_date,
                    ":star_debt": &bot.star_debt,
                    ":blocked": &bot.blocked,
                },
            )?;

            tx.execute(
                "INSERT INTO configs (id, app_config, goal_config) VALUES (:id, :app_config, :goal_config)",
                named_params! {
                    ":id": &bot.id,
                    ":app_config": &app_config,
                    ":goal_config": &goal_config,
                },
            )?;
            tx.commit()
        })
        .await?;

        Ok(())
    }

    async fn update_bot(&self, bot: DBBot) -> Result<()> {
        let conn = &self.conn;

        let admins_json = serde_json::to_string(&bot.admins)?;

        conn.call(move |conn| {
            conn.execute(
                "UPDATE bots SET token = :token, secret_token = :secret_token, ws_token = :ws_token, owner = :owner, admins = :admins, last_payment_date = :last_payment_date, star_debt = :star_debt, blocked = :blocked WHERE id = :id",
                named_params! {
                    ":token": &bot.token,
                    ":secret_token": &bot.secret_token,
                    ":ws_token": &bot.ws_token,
                    ":owner": &bot.owner,
                    ":admins": &admins_json,
                    ":last_payment_date": &bot.last_payment_date,
                    ":star_debt": &bot.star_debt,
                    ":blocked": &bot.blocked,
                    ":id": &bot.id,
                },
            )
        })
        .await?;

        Ok(())
    }

    async fn update_app_config(&self, bot_id: String, app_config: String) -> Result<()> {
        let conn = &self.conn;

        conn.call(move |conn| {
            conn.execute(
                "UPDATE configs SET app_config = :app_config WHERE id = :id",
                named_params! {
                    ":app_config": &app_config,
                    ":id": &bot_id,
                },
            )
        })
        .await?;

        Ok(())
    }

    async fn update_goal_config(&self, bot_id: String, goal_config: String) -> Result<()> {
        let conn = &self.conn;

        conn.call(move |conn| {
            conn.execute(
                "UPDATE configs SET goal_config = :goal_config WHERE id = :id",
                named_params! {
                    ":goal_config": &goal_config,
                    ":id": &bot_id,
                },
            )
        })
        .await?;

        Ok(())
    }

    async fn contains_bot(&self, bot_id: String) -> Result<bool> {
        let conn = &self.conn;
        let count: i64 = conn
            .call(move |conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM bots WHERE id = :id",
                    named_params! {
                        ":id": &bot_id,
                    },
                    |row| row.get(0),
                )
            })
            .await?;
        Ok(count > 0)
    }

    async fn get_app_configs(&self) -> Result<Vec<(String, String)>> {
        let conn = &self.conn;
        let bots_config = conn
            .call(move |conn| {
                let mut stmt = conn.prepare_cached("SELECT id, app_config FROM configs")?;
                let bots_map = stmt.query_map([], |row| {
                    let id: String = row.get(0)?;
                    let app_config: String = row.get(1)?;
                    Ok((id, app_config))
                })?;
                let mut bots_config: Vec<(String, String)> = vec![];

                for bot in bots_map.into_iter() {
                    let bot_config = bot?;
                    bots_config.push(bot_config);
                }
                Ok::<Vec<(String, String)>, async_rusqlite::Error>(bots_config)
            })
            .await?;

        Ok(bots_config)
    }

    async fn get_app_config(&self, bot_id: String) -> Result<String> {
        let conn = &self.conn;
        let app_config = conn
            .call(move |conn| {
                conn.query_row(
                    "SELECT app_config FROM configs WHERE id = :id",
                    named_params! { ":id": &bot_id },
                    |row| row.get(0),
                )
            })
            .await?;
        Ok(app_config)
    }

    async fn get_goal_config(&self, bot_id: String) -> Result<String> {
        let conn = &self.conn;
        let goal_config = conn
            .call(move |conn| {
                conn.query_row(
                    "SELECT goal_config FROM configs WHERE id = :id",
                    named_params! { ":id": &bot_id },
                    |row| row.get(0),
                )
            })
            .await?;
        Ok(goal_config)
    }

    async fn get_bot_token(&self, bot_id: String) -> Result<String> {
        let conn = &self.conn;
        let token = conn
            .call(move |conn| {
                conn.query_row(
                    "SELECT token FROM bots WHERE id = :id",
                    named_params! { ":id": &bot_id },
                    |row| row.get(0),
                )
            })
            .await?;
        Ok(token)
    }

    async fn get_bot_ws_token(&self, bot_id: String) -> Result<String> {
        let conn = &self.conn;
        let token = conn
            .call(move |conn| {
                conn.query_row(
                    "SELECT ws_token FROM bots WHERE id = :id",
                    named_params! { ":id": &bot_id },
                    |row| row.get(0),
                )
            })
            .await?;
        Ok(token)
    }

    async fn add_bot_admin(&self, bot_id: String, admin_id: u64) -> Result<()> {
        let conn = &self.conn;
        conn.call(move |conn| {
            let mut stmt = conn.prepare_cached("SELECT admins FROM bots WHERE id = :id")?;
            let admins_json: String =
                stmt.query_row(named_params! { ":id": &bot_id }, |row| row.get(0))?;

            let mut admins: Vec<u64> = serde_json::from_str(&admins_json).map_err(map_serde_err)?;

            if !admins.contains(&admin_id) {
                admins.push(admin_id);
            }

            let admins_json = serde_json::to_string(&admins).map_err(map_serde_err)?;

            conn.execute(
                "UPDATE bots SET admins = :admins WHERE id = :id",
                named_params! { ":admins": &admins_json, ":id": &bot_id },
            )?;
            Ok::<(), async_rusqlite::Error>(())
        })
        .await?;
        Ok(())
    }

    async fn remove_bot_admin(&self, bot_id: String, admin_id: u64) -> Result<()> {
        let conn = &self.conn;
        conn.call(move |conn| {
            let mut stmt = conn.prepare_cached("SELECT admins FROM bots WHERE id = :id")?;
            let admins_json: String =
                stmt.query_row(named_params! { ":id": &bot_id }, |row| row.get(0))?;

            let mut admins: Vec<u64> = serde_json::from_str(&admins_json).map_err(map_serde_err)?;
            admins.retain(|&id| id != admin_id);
            let admins_json = serde_json::to_string(&admins).map_err(map_serde_err)?;

            conn.execute(
                "UPDATE bots SET admins = :admins WHERE id = :id",
                named_params! { ":admins": &admins_json, ":id": &bot_id },
            )?;
            Ok::<(), async_rusqlite::Error>(())
        })
        .await?;
        Ok(())
    }

    async fn remove_bot(&self, user_id: u64, bot_id: String) -> Result<()> {
        let conn = &self.conn;

        let bot_id_ = bot_id.clone();
        let owner = conn
            .call(move |conn| {
                let mut stmt = conn.prepare_cached("SELECT owner FROM bots WHERE id = :id")?;
                let owner: u64 =
                    stmt.query_row(named_params! { ":id": &bot_id_ }, |row| row.get(0))?;
                Ok::<_, async_rusqlite::Error>(owner)
            })
            .await?;

        if owner != user_id {
            return Err(anyhow::anyhow!("Only owner can delete bot"));
        }

        conn.call(move |conn| {
            let tx = conn.transaction()?;
            tx.execute(
                "DELETE FROM bots WHERE id = :id",
                named_params! { ":id": &bot_id },
            )?;
            tx.execute(
                "DELETE FROM configs WHERE id = :id",
                named_params! { ":id": &bot_id },
            )?;
            tx.commit()
        })
        .await?;
        Ok(())
    }

    async fn change_bot_token(
        &self,
        user_id: u64,
        bot_id: String,
        new_token: String,
    ) -> Result<()> {
        let conn = &self.conn;

        let bot_id_ = bot_id.clone();
        let owner = conn
            .call(move |conn| {
                let mut stmt = conn.prepare_cached("SELECT owner FROM bots WHERE id = :id")?;
                let owner: u64 =
                    stmt.query_row(named_params! { ":id": &bot_id_ }, |row| row.get(0))?;
                Ok::<_, async_rusqlite::Error>(owner)
            })
            .await?;

        if owner != user_id {
            return Err(anyhow::anyhow!("Only owner can change bot token"));
        }
        conn.call(move |conn| {
            conn.execute(
                "UPDATE bots SET token = :token WHERE id = :id",
                named_params! { ":token": &new_token, ":id": &bot_id },
            )?;
            Ok::<(), async_rusqlite::Error>(())
        })
        .await?;
        Ok(())
    }

    async fn update_bot_layer_token(&self, bot_id: String, layer_token: String) -> Result<()> {
        let conn = &self.conn;
        conn.call(move |conn| {
            conn.execute(
                "UPDATE bots SET ws_token = :ws_token WHERE id = :id",
                named_params! { ":ws_token": &layer_token, ":id": &bot_id },
            )?;
            Ok::<(), async_rusqlite::Error>(())
        })
        .await?;
        Ok(())
    }

    async fn increase_stars_debt(&self, bot_id: String, stars_amount: f32) -> Result<()> {
        let conn = &self.conn;
        conn.call(move |conn| {
            conn.execute(
                "UPDATE bots SET star_debt = star_debt + :stars WHERE id = :id",
                named_params! { ":stars": &stars_amount, ":id": &bot_id },
            )?;
            Ok::<(), async_rusqlite::Error>(())
        })
        .await?;
        Ok(())
    }

    async fn decrease_debt(&self, bot_id: String, stars_amount: i64) -> Result<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let conn = &self.conn;
        conn.call(move |conn| {
            conn.execute(
                "UPDATE bots SET last_payment_date = :timestamp, star_debt = star_debt - :stars WHERE id = :id",
                named_params! {
                    ":timestamp": &timestamp,
                    ":stars": &stars_amount,
                    ":id": &bot_id
                },
            )?;
            Ok::<(), async_rusqlite::Error>(())
        })
        .await?;
        Ok(())
    }

    async fn set_bot_blocked(&self, bot_id: String, blocked: bool) -> Result<()> {
        let conn = &self.conn;
        conn.call(move |conn| {
            conn.execute(
                "UPDATE bots SET blocked = :blocked WHERE id = :id",
                named_params! { ":blocked": &blocked, ":id": &bot_id },
            )?;
            Ok::<(), async_rusqlite::Error>(())
        })
        .await?;
        Ok(())
    }

    async fn debt_params(&self, bot_id: String) -> Result<(Option<u64>, f64, bool)> {
        let conn = &self.conn;
        let params = conn
            .call(move |conn| {
                conn.query_row(
                    "SELECT last_payment_date, star_debt, blocked FROM bots WHERE id = :id",
                    named_params! { ":id": &bot_id },
                    |row| {
                        let last_payment_date: Option<u64> = row.get(0)?;
                        let star_debt: f64 = row.get(1)?;
                        let blocked: bool = row.get(2)?;

                        Ok((last_payment_date, star_debt, blocked))
                    },
                )
            })
            .await?;
        Ok(params)
    }

    async fn get_all_bots_debt_params(&self, main_bot_id: &str) -> Result<Vec<BotDebtParams>> {
        let conn = &self.conn;
        let main_bot_id = main_bot_id.to_string();
        let params = conn
            .call(move |conn| {
                let mut stmt = conn.prepare_cached(
                    "SELECT id, last_payment_date, star_debt, blocked FROM bots WHERE id != :main_bot_id"
                )?;
                let bots_map = stmt.query_map(named_params! { ":main_bot_id": main_bot_id }, |row| {
                    let id: String = row.get(0)?;
                    let last_payment_date: Option<u64> = row.get(1)?;
                    let star_debt: f64 = row.get(2)?;
                    let blocked: bool = row.get(3)?;
                    Ok(BotDebtParams {
                        id,
                        last_payment_date,
                        star_debt,
                        blocked,
                    })
                })?;
                let mut results: Vec<BotDebtParams> = vec![];
                for bot_params in bots_map.into_iter() {
                    results.push(bot_params?);
                }
                Ok::<Vec<BotDebtParams>, async_rusqlite::Error>(results)
            })
            .await?;
        Ok(params)
    }
}

fn map_serde_err(e: serde_json::error::Error) -> rusqlite::Error {
    rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error {
            code: rusqlite::ffi::ErrorCode::TypeMismatch,
            extended_code: rusqlite::ffi::SQLITE_MISMATCH,
        },
        Some(format!("Failed to parse admins json: {}", e)),
    )
}
