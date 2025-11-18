use hmac::{Hmac, Mac};
use hyper::HeaderMap;
use sha2::Sha256;
use std::collections::HashMap;

use crate::json;

/// Check secret token from headers for webhook authentication
pub fn check_secret_token(secret_token: &str, headers: &HeaderMap) -> bool {
    if let Some(header_token) = headers.get("X-Telegram-Bot-Api-Secret-Token") {
        if header_token == secret_token {
            return true;
        } else {
            println!(
                "check_secret_token header_token: {:?} secret_token: {:?}",
                header_token, secret_token
            );
        }
    }

    false
}

/// Extract and validate Telegram Mini App init data from headers
pub fn check_hash_in_headers(headers: &HeaderMap, token: &str) -> Option<json::WebAppUser> {
    if let Some(hash) = headers.get("X-Telegram-InitData") {
        return check_hash(hash.to_str().unwrap(), token);
    }

    None
}

/// Validate Telegram Mini App init data hash
/// Reference: https://core.telegram.org/bots/webapps#validating-data-received-via-the-mini-app
pub fn check_hash(init_data: &str, token: &str) -> Option<json::WebAppUser> {
    let data: HashMap<_, _> = form_urlencoded::parse(init_data.as_bytes())
        .into_owned()
        .collect();

    let mut check_string = data
        .iter()
        .filter(|&(key, _)| key != "hash")
        .map(|(key, value)| format!("{}={}", key, value))
        .collect::<Vec<_>>();
    check_string.sort();
    let check_string = check_string.join("\n");

    let mut mac = Hmac::<Sha256>::new_from_slice(b"WebAppData").unwrap();
    mac.update(token.as_bytes());
    let secret_key = mac.finalize().into_bytes();

    let mut mac = Hmac::<Sha256>::new_from_slice(&secret_key).unwrap();
    mac.update(check_string.as_bytes());
    let signature = mac.finalize().into_bytes();

    if let Some(hash) = data.get("hash") {
        // println!("hash: {}\nsign: {}", hash, hex::encode(signature));

        if hex::encode(signature) == *hash {
            if let Some(user) = data.get("user") {
                let user: Option<json::WebAppUser> = serde_json::from_str(user).ok();
                return user;
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_hash() {
        //second test bot
        let init_data = "query_id=AAG8IcAUAAAAALwhwBTPhTwZ&user=%7B%22id%22%3A348135868%2C%22first_name%22%3A%22%D0%93%D1%80%D0%B8%D0%B3%D0%BE%D1%80%D0%B8%D0%B9%22%2C%22last_name%22%3A%22%D0%91%D0%BE%D1%80%D0%B8%D1%81%D0%BE%D0%B2%22%2C%22username%22%3A%22Torsor%22%2C%22language_code%22%3A%22ru%22%2C%22allows_write_to_pm%22%3Atrue%2C%22photo_url%22%3A%22https%3A%5C%2F%5C%2Ft.me%5C%2Fi%5C%2Fuserpic%5C%2F320%5C%2FwsUOF6a3vdHs4d6GxHTdD5Y7swpuTZO6dz0iWc0e8go.svg%22%7D&auth_date=1734603245&signature=vf8Crn0P3kI1ZE0HvkgzBT3XZxGGjehqpn7vgIHidwQ18GVNdkZ6RgRkRAjmoM2VNihAdBAYOyRaNYICKQPbBQ&hash=42c3594acc55ea181d8d7a62be0e79af134e1a400d681345c88f18ac52844a38";
        let token = "8090667304:AAFDIkQ7htfPHAjm2Vnzrl5JH6oELo4Y1e4";

        let user = check_hash(init_data, token);

        assert!(user.is_some());
        assert_eq!(user.unwrap().id, 348135868);
    }
}
