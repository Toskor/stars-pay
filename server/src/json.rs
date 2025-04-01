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
    // pub user_id: u32,
    pub title: String,
    pub description: String,
    pub payload: String,
    pub amount: u32,
    //currency = XTR
    //subscription_period ?
}

#[derive(Deserialize, Debug)]
pub struct CreateInvoiceAnswer {
    pub result: String,
}

#[derive(Deserialize, Debug)]
pub struct BotInfo {
    pub ok: bool,
    pub result: Option<BotInfoResult>,
    pub error_code: Option<u32>,
    pub description: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BotInfoResult {
    pub id: u64,
    pub username: String,
    pub first_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UserInfo {
    pub ok: bool,
    pub result: Option<UserInfoResult>,
    pub error_code: Option<u32>,
    pub description: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UserInfoResult {
    pub id: u64,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
}

#[derive(Deserialize)]
pub struct UpdateConfigQueryParam {
    pub app_config: String,
    pub target_bot_id: String,
}

#[derive(Deserialize, Clone)]
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

#[derive(Deserialize)]
pub struct RemoveBotQueryParam {
    pub bot_id: String,
}

#[derive(Deserialize)]
pub struct ChangeBotTokenQueryParam {
    pub bot_id: String,
    pub new_token: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MainBotMainPageProps {
    pub bots: Vec<TMABotData>,
}

#[derive(Deserialize, Serialize, Debug)]
/// Telegram Mini App Bot Data for Main Bot pages
pub struct TMABotData {
    pub id: String,
    pub numeric_id: u64,
    pub name: String,
    pub avatar: Option<String>,

    #[serde(rename = "userRole")]
    pub user_role: String,
    pub owner: TMAUserData,
    pub admins: Vec<TMAUserData>,

    pub suspended: Option<bool>,
    pub debt: Option<u64>,
}

#[derive(Deserialize, Serialize, Debug)]
/// Telegram Mini App User Data for Main Bot pages
pub struct TMAUserData {
    pub id: u64,
    //ex torsor
    pub username: String,
    //ex Григорий Борисов
    pub name: String,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct UserProfilePhotos {
    pub ok: bool,
    pub result: Option<UserProfilePhotosResult>,
    pub error_code: Option<u32>,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct UserProfilePhotosResult {
    pub total_count: u32,
    pub photos: Vec<Vec<PhotoSize>>,
}

#[derive(Deserialize, Debug)]
pub struct PhotoSize {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: u32,
    pub height: u32,
    pub file_size: Option<u32>,
}

#[derive(serde::Deserialize)]
pub struct FileResponse {
    pub ok: bool,
    pub result: FileResult,
}

#[derive(serde::Deserialize)]
pub struct FileResult {
    pub file_id: String,
    pub file_unique_id: String,
    pub file_size: Option<u32>,
    pub file_path: String,
}

#[derive(Deserialize)]
pub struct ConfigQueryParam {
    pub target_bot_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TMAAppConfig {
    pub donation_buttons: Vec<DonationButton>,
    pub title: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DonationButton {
    pub name: String,
    pub description: String,
    pub amount: u32,
    pub source_id: u32,
    pub invoice_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WSDonationEvent {
    ///@username
    pub from: String,
    //currency: always TgStars,
    pub total_amount: u32,
    pub invoice_payload: String,
}

#[derive(Debug, Deserialize)]
pub struct WSConnectionParams {
    pub ws_token: String,
}

#[derive(Deserialize)]
pub struct RefreshLayerTokenQueryParams {
    pub target_bot_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoomMessage {
    Text(Vec<u8>),
    CloseConnection(usize),
    CloseRoom(String),
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
