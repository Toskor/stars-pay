use std::str::FromStr;

use crate::{json, WEBHOOK_ALLOWED_UPDATES};
use anyhow::Result;
use hyper::{header, HeaderMap, StatusCode};

pub fn generate_secret_token() -> String {
    uuid::Uuid::new_v4().to_string()
}
pub fn get_tg_api_url(token: &str) -> String {
    format!("https://api.telegram.org/bot{}/", token)
}

pub fn bot_numeric_id_from_token(token: &str) -> Result<u64> {
    let token_parts = token.split(':').collect::<Vec<&str>>();
    if token_parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid token format"));
    }
    let numeric_id = token_parts[0].parse::<u64>()?;
    Ok(numeric_id)
}

pub async fn set_tg_webhook(token: &str, webhook_url: &str, secret_token: &str) -> Result<()> {
    let tg_api_url = get_tg_api_url(token);
    let url_param = format!(
        "url={webhook_url}&allowed_updates={}&secret_token={secret_token}",
        WEBHOOK_ALLOWED_UPDATES
    );

    let uri = hyper::Uri::from_str(&format!("{}setWebhook?{url_param}", tg_api_url)).unwrap();

    let res = integrations::http::get(&uri, None).await?;

    if res.status == StatusCode::OK {
        // println!("{}", res.to_str().unwrap());
        Ok(())
    } else {
        let err: json::Error = res.to_json()?;
        Err(anyhow::anyhow!("{}", err.description))
    }
}

pub async fn get_webhook_info(tg_api_url: &str) -> Result<()> {
    let uri = hyper::Uri::from_str(&format!("{}getWebhookInfo", tg_api_url)).unwrap();

    let res = integrations::http::get(&uri, None).await?;

    if res.status != StatusCode::OK {
        println!("bad status {}", res.status);
    } else {
        println!("{}", res.to_str().unwrap())
    }

    Ok(())
}

pub async fn set_bot_commands(tg_api_url: &str) -> Result<()> {
    // can add language_code
    let commands = serde_json::json!([
        {
            "command": "start",
            "description": "Запустить бота"
        },
        {
            "command": "help",
            "description": "Получить помощь"
        }
    ]);
    let commands_str = serde_json::to_string(&commands).unwrap();

    let uri = hyper::Uri::from_str(&format!("{}setMyCommands", tg_api_url)).unwrap();

    let res = integrations::http::post(&uri, None, commands_str).await?;

    if res.status != StatusCode::OK {
        println!("bad status {}", res.status);
    } else {
        println!("{}", res.to_str().unwrap())
    }

    Ok(())
}

//https://core.telegram.org/bots/api#createinvoicelink
pub async fn create_invoice_link(
    tg_api_url: &str,
    params: &json::CreateInvoiceQueryParam,
) -> Result<String> {
    //todo add photo_url URL of the product photo for the invoice. Can be a photo of the goods or a marketing image for a service.
    //the picture that user pick to show on stream
    let body = serde_json::json!({
        "title": params.title,
        "description": params.description,
        "payload": params.payload,
        "provider_token": "",
        "currency": "XTR",
        "prices": [
            {
                "label": "Цена",
                "amount": params.amount
            }
        ]
    });
    let body_str = serde_json::to_string(&body).unwrap();
    let headers = HeaderMap::from_iter([(
        header::CONTENT_TYPE,
        hyper::header::HeaderValue::from_bytes("application/json".as_bytes()).unwrap(),
    )]);

    let uri = hyper::Uri::from_str(&format!("{}createInvoiceLink", tg_api_url)).unwrap();

    let res = integrations::http::post(&uri, Some(&headers), body_str).await?;
    // println!("{}", res.to_str().unwrap());

    if res.status != StatusCode::OK {
        let err: json::Error = res.to_json()?;
        println!("cil bad status {} {}", res.status, err.description);

        return Err(anyhow::anyhow!(err.description));
    } else {
        let res_body: json::CreateInvoiceAnswer = res.to_json()?;
        return Ok(res_body.result);
    }
}

pub async fn set_menu_button(token: &str, button_text: &str, button_url: &str) -> Result<()> {
    let tg_api_url = get_tg_api_url(token);
    let uri = hyper::Uri::from_str(&format!("{}setChatMenuButton", tg_api_url)).unwrap();

    let body = serde_json::json!({
        "menu_button": {
            "type": "web_app",
            "text": button_text,
            "web_app": {
                "url": button_url
            }
        }
    });
    let body_str = serde_json::to_string(&body).unwrap();

    let headers = HeaderMap::from_iter([(
        header::CONTENT_TYPE,
        hyper::header::HeaderValue::from_bytes("application/json".as_bytes()).unwrap(),
    )]);

    let res = integrations::http::post(&uri, Some(&headers), body_str).await?;
    // println!("{}", res.to_str().unwrap());
    if res.status == StatusCode::OK {
        Ok(())
    } else {
        let err: json::Error = res.to_json()?;
        Err(anyhow::anyhow!("{}", err.description))
    }
}

pub async fn get_bot_info(token: &str) -> Result<json::BotInfo> {
    let tg_api_url = get_tg_api_url(token);
    let uri = hyper::Uri::from_str(&format!("{}getMe", tg_api_url)).unwrap();

    let res = integrations::http::get(&uri, None).await?;

    let json: json::BotInfo = res.to_json()?;
    Ok(json)
}

pub async fn get_user_info(token: &str, user_id: u64) -> Result<json::UserInfo> {
    //getChat
    let tg_api_url = get_tg_api_url(token);
    let uri = hyper::Uri::from_str(&format!("{}getChat?chat_id={}", tg_api_url, user_id)).unwrap();

    let res = integrations::http::get(&uri, None).await?;
    let user_info: json::UserInfo = res.to_json()?;

    Ok(user_info)
}

async fn get_user_profile_photos(token: &str, user_id: u64) -> Result<json::UserProfilePhotos> {
    let tg_api_url = get_tg_api_url(token);

    let uri = hyper::Uri::from_str(&format!(
        "{}getUserProfilePhotos?user_id={}",
        tg_api_url, user_id
    ))
    .unwrap();

    let res = integrations::http::get(&uri, None).await?;

    if res.status == StatusCode::OK {
        Ok(res.to_json()?)
    } else {
        let err: json::Error = res.to_json()?;
        Err(anyhow::anyhow!("{}", err.description))
    }
}

async fn get_file_url(token: &str, file_id: &str) -> Result<String> {
    let tg_api_url = get_tg_api_url(token);
    let uri = hyper::Uri::from_str(&format!("{}getFile?file_id={}", tg_api_url, file_id)).unwrap();

    let res = integrations::http::get(&uri, None).await?;

    if res.status == StatusCode::OK {
        let file_info: json::FileResponse = res.to_json()?;
        // https://core.telegram.org/bots/api#getfile
        let file_url = format!(
            "https://api.telegram.org/file/bot{}/{}",
            token, file_info.result.file_path
        );
        Ok(file_url)
    } else {
        let err: json::Error = res.to_json()?;
        Err(anyhow::anyhow!("{}", err.description))
    }
}

pub async fn get_avatar_url(token: &str, id: u64) -> Result<Option<String>> {
    let photos = get_user_profile_photos(token, id).await?;
    // println!("photos: {:?}", photos);

    let photos_result = if photos.ok {
        photos.result.unwrap()
    } else {
        return Err(anyhow::anyhow!(
            "{} {}",
            photos.error_code.unwrap(),
            photos.description.unwrap()
        ));
    };

    if photos_result.total_count == 0 || photos_result.photos.is_empty() {
        return Ok(None);
    }

    if let Some(photo_sizes) = photos_result.photos.first() {
        if let Some(largest_photo) = photo_sizes.last() {
            let file_url = get_file_url(token, &largest_photo.file_id).await?;
            return Ok(Some(file_url));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use tokio::time::Instant;

    use crate::main_bot::{MAIN_BOT_OWNER, MAIN_BOT_TOKEN};

    use super::*;

    #[tokio::test]
    async fn test_get_user_info_by_id() {
        let token = MAIN_BOT_TOKEN;
        let user_id = 2135923914;

        let user_info = get_user_info(token, user_id).await.unwrap();
        println!(
            "user_info: {:?}",
            serde_json::to_string(&user_info).unwrap()
        );
    }

    #[tokio::test]
    async fn test_get_me() {
        let token = MAIN_BOT_TOKEN;

        let start = Instant::now();
        let res = get_bot_info(token).await.unwrap();
        let duration = start.elapsed();
        println!("res: {}", res.result.unwrap().username);
        println!("duration out: {:?}", duration);
    }

    #[tokio::test]
    async fn test_new_bot() {
        //just_for_test75w67_bot
        let token = "7662668003:AAHf61QA87mA0nCTSgzWJr7X7vm-dYvvShM";
        let res = get_bot_info(token).await.unwrap();
        println!("res: {}", res.result.unwrap().username);
    }

    #[tokio::test]
    async fn test_get_avatar_url() {
        let token = MAIN_BOT_TOKEN;
        //@yuryk
        let user_id = 487373;

        let user_id = 348135868;

        //bot star donatoin
        let user_id = 7792542554;

        let avatar_url = get_avatar_url(&token, user_id).await.unwrap();
        println!("avatar_url: {}", avatar_url.unwrap());
    }
}
