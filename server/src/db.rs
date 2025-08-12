use anyhow::Result;
use async_rusqlite::{rusqlite::named_params, Connection};
use rusqlite::functions::FunctionFlags;

#[derive(Debug, Clone)]
pub struct DBBot {
    ///bot username without "bot" end
    pub id: String,

    pub token: String,
    pub secret_token: String,
    pub ws_token: String,

    // pub app_config: String,
    pub owner: u64,
    // admins id vec in json format
    pub admins: Vec<u64>,
    // Last payment date
    pub last_payment_date: Option<u64>,
    // Stars debt
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

#[derive(Debug, Clone)]
pub struct BotDebtParams {
    pub id: String,
    pub last_payment_date: Option<u64>,
    pub star_debt: f64,
    pub blocked: bool,
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
                id                  TEXT PRIMARY KEY NOT NULL,
                token               TEXT NOT NULL,
                secret_token        TEXT NOT NULL,
                ws_token            TEXT NOT NULL,
                app_config          TEXT NOT NULL,
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

    pub async fn get_bots_by_owner_id(&self, owner_id: u64) -> Result<Vec<DBBot>> {
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

    pub async fn get_bot(&self, bot_id: String) -> Result<DBBot> {
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

    pub async fn insert_bot(&self, bot: DBBot, app_config: String) -> Result<()> {
        let conn = &self.conn;

        let admins_json = serde_json::to_string(&bot.admins)?;

        conn.call(move |conn| {
            conn.execute(
                "INSERT INTO bots (id, token, secret_token, ws_token, app_config, owner, admins, last_payment_date, star_debt, blocked) VALUES (:id, :token, :secret_token, :ws_token, :app_config, :owner, :admins, :last_payment_date, :star_debt, :blocked)",
                named_params! {
                    ":id": &bot.id,
                    ":token": &bot.token,
                    ":secret_token": &bot.secret_token,
                    ":ws_token": &bot.ws_token,
                    ":app_config": &app_config,
                    ":owner": &bot.owner,
                    ":admins": &admins_json,
                    ":last_payment_date": &bot.last_payment_date,
                    ":star_debt": &bot.star_debt,
                    ":blocked": &bot.blocked,
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

    pub async fn get_bot_token(&self, bot_id: String) -> Result<String> {
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

    pub async fn get_bot_ws_token(&self, bot_id: String) -> Result<String> {
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

    pub async fn add_bot_admin(&self, bot_id: String, admin_id: u64) -> Result<()> {
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

    pub async fn remove_bot(&self, user_id: u64, bot_id: String) -> Result<()> {
        let conn = &self.conn;

        // First, check if the user is the owner of the bot
        let bot_id_ = bot_id.clone();
        let owner = conn
            .call(move |conn| {
                let mut stmt = conn.prepare_cached("SELECT owner FROM bots WHERE id = :id")?;
                let owner: u64 =
                    stmt.query_row(named_params! { ":id": &bot_id_ }, |row| row.get(0))?;
                Ok::<_, async_rusqlite::Error>(owner)
            })
            .await?;

        // If the user is not the owner, return an error
        if owner != user_id {
            return Err(anyhow::anyhow!("Only owner can delete bot"));
        }

        // If the user is the owner, proceed with deletion
        conn.call(move |conn| {
            conn.execute(
                "DELETE FROM bots WHERE id = :id",
                named_params! { ":id": &bot_id },
            )?;
            Ok::<(), async_rusqlite::Error>(())
        })
        .await?;
        Ok(())
    }

    pub async fn change_bot_token(
        &self,
        user_id: u64,
        bot_id: String,
        new_token: String,
    ) -> Result<()> {
        let conn = &self.conn;

        // First, check if the user is the owner of the bot
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

    pub async fn update_bot_layer_token(&self, bot_id: String, layer_token: String) -> Result<()> {
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

    pub async fn update_last_payment_date(&self, bot_id: String, timestamp: i64) -> Result<()> {
        let conn = &self.conn;
        conn.call(move |conn| {
            conn.execute(
                "UPDATE bots SET last_payment_date = :timestamp WHERE id = :id",
                named_params! { ":timestamp": &timestamp, ":id": &bot_id },
            )?;
            Ok::<(), async_rusqlite::Error>(())
        })
        .await?;
        Ok(())
    }

    pub async fn update_stars_balance(&self, bot_id: String, stars: i64) -> Result<()> {
        let conn = &self.conn;
        conn.call(move |conn| {
            conn.execute(
                "UPDATE bots SET star_debt = :stars WHERE id = :id",
                named_params! { ":stars": &stars, ":id": &bot_id },
            )?;
            Ok::<(), async_rusqlite::Error>(())
        })
        .await?;
        Ok(())
    }

    pub async fn increase_stars_debt(&self, bot_id: String, stars_amount: f32) -> Result<()> {
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

    pub async fn decrease_debt(&self, bot_id: String, stars_amount: i64) -> Result<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

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

    pub async fn set_bot_blocked(&self, bot_id: String, blocked: bool) -> Result<()> {
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

    pub async fn debt_params(&self, bot_id: String) -> Result<(Option<u64>, f64, bool)> {
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

    /// Get debt parameters for all bots except the main bot in one query
    pub async fn get_all_bots_debt_params(&self) -> Result<Vec<BotDebtParams>> {
        let conn = &self.conn;
        let main_bot_id = dotenv!("MAIN_BOT_ID");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn change_bot() {
        let db = DataBase::new_sql_lite(
            "/Users/grigory/Documents/GitHub/tg-stars/db/bots_data_base.sqlite",
        )
        .await
        .expect("Failed to create database");

        let mut bot = db.get_bot("second_test_1".to_string()).await.unwrap();
        bot.blocked = false;
        bot.last_payment_date = Some(1742187221);

        db.update_bot(bot).await.unwrap();
    }

    #[tokio::test]
    async fn test_add_admin() {
        let bot_id = "star_donation";
        //yury
        let admin_id_1 = 487373;
        //matvey
        let admin_id_2 = 2135923914;

        let db = DataBase::new_sql_lite("db/bots_data_base.sqlite")
            .await
            .expect("Failed to create database");

        db.add_bot_admin(bot_id.to_string(), admin_id_1)
            .await
            .unwrap();
        db.add_bot_admin(bot_id.to_string(), admin_id_2)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn set_new_db_app_config() {
        let preview_default_config = r#"{"title":"Donate to me","donation_buttons":[{"name":"Donate 1","description":"Description 1","amount":100,"source_url":"https://i.imgur.com/892vhef.jpeg","invoice_url":"https://t.me/$clgGxe0mwEq9CwAAvJUBv--iitU"},{"name":"Donate 2","description":"Description 2","amount":200,"source_url":"https://avatars.mds.yandex.net/i?id=afb182659773be12c48e7a49d7e8212c_l-5858046-images-thumbs&n=13","invoice_url":"https://t.me/$pHcweO0mwEq-CwAATVxs8DbroT0"},{"name":"Donate 3","description":"Description 3","amount":300,"source_url":"https://avatars.mds.yandex.net/i?id=3ef58cad5f77fcebe674582d17765372_l-4032453-images-thumbs&n=13","invoice_url":"https://t.me/$HYySbu0mwErACwAA5Pxvbym2xzw"},{"name":"Donate 4","description":"Description 4","amount":400,"source_url":"https://avatars.mds.yandex.net/i?id=cc49c0be94d8640bd74a7a7a4ba48dfd_l-2396749-images-thumbs&n=13","invoice_url":"https://t.me/$RJw1Ye0mwErBCwAA2omxPGrT2II"}]}"#;
        let db = DataBase::new_sql_lite("../db/bots_data_base.sqlite")
            .await
            .expect("Failed to create database");

        db.update_bot_config(
            "second_test_1".to_string(),
            preview_default_config.to_string(),
        )
        .await
        .unwrap();

        db.update_bot_config(
            "star_donation".to_string(),
            preview_default_config.to_string(),
        )
        .await
        .unwrap();

        db.update_bot_config(
            "just_for_test75w67".to_string(),
            preview_default_config.to_string(),
        )
        .await
        .unwrap();
    }
}
