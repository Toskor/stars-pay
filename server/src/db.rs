use std::sync::Arc;

use anyhow::Result;
use async_rusqlite::{rusqlite::named_params, Connection};
use rusqlite::functions::FunctionFlags;

use crate::app_state::UserRole;

#[derive(Debug, Clone)]
pub struct DBBot {
    ///bot username without "bot" end
    pub id: String,

    pub token: String,
    pub secret_token: String,

    // pub app_config: String,
    pub owner: u64,
    // admins id vec in json format
    pub admins: Vec<u64>,
}
impl DBBot {
    pub fn new(
        id: String,
        token: String,
        secret_token: String,
        owner: u64,
        admins: Vec<u64>,
    ) -> Self {
        DBBot {
            id,
            token,
            secret_token,
            owner,
            admins,
        }
    }
}

pub struct DataBase {
    pub conn: Connection,
}

//todo fn update
impl DataBase {
    pub async fn new_sql_lite(path: &str) -> Result<Self> {
        let conn = Connection::open(path).await?;

        conn.call(move |conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS bots (
                id              TEXT PRIMARY KEY NOT NULL,
                token           TEXT NOT NULL,
                secret_token    TEXT NOT NULL,
                app_config      TEXT NOT NULL,
                owner           INTEGER NOT NULL,
                admins          TEXT NOT NULL
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

        //todo improve sql index to use in get_bots_by_admin_id
        conn.call(move |conn| {
            conn.execute(
                "CREATE INDEX IF NOT EXISTS admins_contains ON bots ( admins_contains(admins, 1) )",
                (),
            )
        })
        .await?;

        Ok(Self { conn })
    }

    pub async fn get_bots_by_admin_id(&self, admin_id: u64) -> Result<Vec<DBBot>> {
        let conn = &self.conn;
        let search_pattern = format!("%{}%", admin_id);

        let bots = conn
            .call(move |conn| {
                let mut stmt = conn.prepare_cached(
                    "SELECT id, token, secret_token, owner, admins FROM bots 
                    WHERE admins LIKE :search_pattern",
                )?;
                let query_result =
                    stmt.query_map(named_params! { ":search_pattern": search_pattern }, |row| {
                        let id: String = row.get(0)?;
                        let token: String = row.get(1)?;
                        let secret_token: String = row.get(2)?;
                        let owner: u64 = row.get(3)?;
                        let admins_json = row.get_ref(4)?.as_str()?;
                        let admins: Vec<u64> =
                            serde_json::from_str(admins_json).map_err(map_serde_err)?;

                        if !admins.contains(&owner) {
                            Ok(Some(DBBot {
                                id,
                                token,
                                secret_token,
                                owner,
                                admins,
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

    pub async fn get_bots_by_owner_id(&self, owner_id: u64) -> Result<Vec<DBBot>> {
        let conn = &self.conn;
        let bots = conn
            .call(move |conn| {
                let mut stmt =
                    conn.prepare_cached("SELECT id, token, secret_token, owner, admins FROM bots WHERE owner = :owner_id")?;
                let bots_map = stmt.query_map(named_params! { ":owner_id": owner_id }, |row| {
                    let id: String = row.get(0)?;
                    let token: String = row.get(1)?;
                    let secret_token: String = row.get(2)?;
                    let owner: u64 = row.get(3)?;

                    let admins_json = row.get_ref(4)?.as_str()?;
                    let admins: Vec<u64> =
                        serde_json::from_str(admins_json).map_err(map_serde_err)?;
                    Ok(DBBot { id, token, secret_token, owner, admins })
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

    pub async fn get_bot(&self, bot_id: String) -> Result<DBBot> {
        let conn = &self.conn;

        let bot = conn
            .call(move |conn| {
                let mut stmt = conn.prepare_cached(
                    "SELECT id, token, secret_token, owner, admins FROM bots WHERE id = ?",
                )?;
                let bot = stmt.query_row([bot_id], |row| {
                    let admins_json = row.get_ref(4)?.as_str()?;
                    let admins: Vec<u64> =
                        serde_json::from_str(admins_json).map_err(map_serde_err)?;

                    Ok(DBBot {
                        id: row.get(0)?,
                        token: row.get(1)?,
                        secret_token: row.get(2)?,
                        owner: row.get(3)?,
                        admins: admins,
                    })
                })?;

                Ok::<_, async_rusqlite::Error>(bot)
            })
            .await?;

        Ok(bot)
    }

    pub async fn insert_bot(&self, bot: DBBot, app_config: String) -> Result<()> {
        let conn = &self.conn;

        let admins_json = serde_json::to_string(&bot.admins)?;

        conn.call(move |conn| {
            conn.execute(
                "INSERT INTO bots (id, token, secret_token, app_config, owner, admins) VALUES (:id, :token, :secret_token, :app_config, :owner, :admins)",
                named_params! {
                    ":id": &bot.id,
                    ":token": &bot.token,
                    ":secret_token": &bot.secret_token,
                    ":app_config": &app_config,
                    ":owner": &bot.owner,
                    ":admins": &admins_json,
                },
            )
        })
        .await?;

        Ok(())
    }

    pub async fn update_bot(&self, bot: DBBot) -> Result<()> {
        let conn = &self.conn;

        let admins_json = serde_json::to_string(&bot.admins)?;

        conn.call(move |conn| {
            conn.execute(
                "UPDATE bots SET token = :token, secret_token = :secret_token, owner = :owner, admins = :admins WHERE id = :id",
                named_params! {
                    ":token": &bot.token,
                    ":secret_token": &bot.secret_token,
                    ":owner": &bot.owner,
                    ":admins": &admins_json,
                    ":id": &bot.id,
                },
            )
        })
        .await?;

        Ok(())
    }

    pub async fn update_bot_config(&self, bot_id: String, app_config: String) -> Result<()> {
        let conn = &self.conn;

        conn.call(move |conn| {
            conn.execute(
                "UPDATE bots SET app_config = :app_config WHERE id = :id",
                named_params! {
                    ":app_config": &app_config,
                    ":id": &bot_id,
                },
            )
        })
        .await?;

        Ok(())
    }

    pub async fn contains_bot(&self, bot_id: String) -> Result<bool> {
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

    pub async fn get_bots_config(&self) -> Result<Vec<(String, String)>> {
        let conn = &self.conn;
        let bots_config = conn
            .call(move |conn| {
                let mut stmt = conn.prepare_cached("SELECT id, app_config FROM bots")?;
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

    pub async fn get_bot_config(&self, bot_id: String) -> Result<String> {
        let conn = &self.conn;
        let app_config = conn
            .call(move |conn| {
                conn.query_row(
                    "SELECT app_config FROM bots WHERE id = :id",
                    named_params! { ":id": &bot_id },
                    |row| row.get(0),
                )
            })
            .await?;
        Ok(app_config)
    }

    pub async fn add_bot_admin(&self, bot_id: String, admin_id: u64) -> Result<()> {
        let conn = &self.conn;
        conn.call(move |conn| {
            let mut stmt = conn.prepare_cached("SELECT admins FROM bots WHERE id = :id")?;
            let admins_json: String =
                stmt.query_row(named_params! { ":id": &bot_id }, |row| row.get(0))?;

            let mut admins: Vec<u64> = serde_json::from_str(&admins_json).map_err(map_serde_err)?;
            admins.push(admin_id);
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

    pub async fn remove_bot_admin(&self, bot_id: String, admin_id: u64) -> Result<()> {
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
