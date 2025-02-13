use std::str::FromStr;

use crate::{json, WEBHOOK_ALLOWED_UPDATES};
use anyhow::Result;
use hyper::{header, HeaderMap, StatusCode};

pub fn generate_secret_token() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub async fn set_tg_webhook(tg_api_url: &str, webhook_url: &str, secret_token: &str) -> Result<()> {
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

pub async fn get_bot_info(tg_api_url: &str) -> Result<json::BotInfo> {
    let uri = hyper::Uri::from_str(&format!("{}getMe", tg_api_url)).unwrap();

    let res = integrations::http::get(&uri, None).await?;

    Ok(res.to_json()?)
}

pub async fn set_menu_button(tg_api_url: &str, button_text: &str, button_url: &str) -> Result<()> {
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

#[cfg(test)]
mod tests {}
