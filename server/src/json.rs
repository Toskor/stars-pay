use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct Error {
    pub ok: bool, // = false
    pub error_code: u32,
    pub description: String,
}

#[derive(Deserialize, Debug)]
pub struct Update {
    pub update_id: u64,
    #[serde(flatten)]
    pub data: UpdateData,
}

#[derive(Deserialize, Debug)]
pub enum UpdateData {
    #[serde(rename = "pre_checkout_query")]
    PreCheckoutQuery(PreCheckoutQuery),
    #[serde(rename = "message")]
    Message(Message),
}

#[derive(Deserialize, Debug)]
pub struct Message {
    pub message_id: u64,
    pub text: Option<String>,
    pub chat: Option<MessageChat>,
}

#[derive(Deserialize, Debug)]
pub struct MessageChat {
    pub id: u64,
    #[serde(rename = "type")]
    pub chat_type: String,
}

#[derive(Deserialize, Debug)]
pub struct PreCheckoutQuery {
    pub id: String,
    pub from: PreCheckoutQueryFrom,
    pub currency: String,
    pub total_amount: u32,
    pub invoice_payload: String,
}

#[derive(Deserialize, Debug)]
pub struct PreCheckoutQueryFrom {
    pub id: u64,
    pub is_bot: bool,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub language_code: String,
}

//docs https://core.telegram.org/bots/api#createinvoicelink
#[derive(Deserialize, Debug)]
pub struct CreateInvoiceQueryParam {
    pub user_id: u32,
    pub title: String,
    pub description: String,
    pub payload: String,
    pub amount: u32,
    //currency = XTR
    //subscription_period ?
}

//{
// "ok": true,
// "result": "https://t.me/$3U620DljMEpICQAAqZv97cvJbEw"
// }
#[derive(Deserialize, Debug)]
pub struct CreateInvoiceAnswer {
    pub result: String,
}

#[derive(Deserialize)]
pub struct BotInfo {
    pub ok: bool,
    pub result: BotInfoResult,
}

#[derive(Deserialize)]
pub struct BotInfoResult {
    pub id: u64,
    pub username: String,
}

#[derive(Deserialize)]
pub struct UpdateConfigQueryParam {
    pub app_config: String,
}

#[derive(Deserialize)]
pub struct WebAppUser {
    pub id: u64,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub language_code: String,
    pub photo_url: String,
}

#[derive(Deserialize)]
pub struct AddBotQueryParam {
    pub bot_token: String,
}

#[derive(Deserialize)]
pub struct AddBotAdminQueryParam {
    pub bot_id: String,
    pub admin_id: u64,
}

#[derive(Deserialize)]
pub struct RemoveBotAdminQueryParam {
    pub bot_id: String,
    pub admin_id: u64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ControlledBots {
    pub owner_bots: Vec<String>,
    pub admin_bots: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MainBotMainPageProps {
    pub bots: Vec<Bot>,
    pub has_suspended_bots: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Bot {
    pub id: u64,
    pub name: String,
    pub avatar: String,

    pub user_role: String,
    pub owner: User,
    pub admins: Vec<User>,

    pub suspended: bool,
    pub balance: u64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    pub id: u64,
    //ex torsor
    pub username: String,
    //ex Григорий Борисов
    pub name: String,
    pub avatar_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_pre_checkout() {
        let data = r#"{"pre_checkout_query":{"currency":"XTR","from":{"first_name":"Григорий","id":348135868,"is_bot":false,"language_code":"ru","last_name":"Борисов","username":"Torsor"},"id":"1495232168384784946","invoice_payload":"12345","total_amount":1},"update_id":13134160}"#;
        let _: Update = serde_json::from_str(data).unwrap();
    }

    #[test]
    fn parse_message() {
        let data = r#"{"message":{"chat":{"first_name":"Григорий","id":348135868,"last_name":"Борисов","type":"private","username":"Torsor"},"date":1732798848,"from":{"first_name":"Григорий","id":348135868,"is_bot":false,"language_code":"ru","last_name":"Борисов","username":"Torsor"},"message_id":39,"text":"fvd"},"update_id":13134182}"#;
        let _: Update = serde_json::from_str(data).unwrap();
    }
}
