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
    pub entities: Option<Vec<MessageEntity>>,
}

#[derive(Deserialize, Debug)]
pub struct MessageEntity {
    #[serde(rename = "type")]
    pub entity_type: String,
    pub offset: u64,
    pub length: u64,
    ///For “text_link” only, URL that will be opened after user taps on the text
    pub url: Option<String>,
    ///For “text_mention” only, the mentioned user
    pub user: Option<UserInfoResult>,
    ///For “pre” only, the programming language of the entity text
    pub language: Option<String>,
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
pub struct GetDebtInvoiceURLQueryParam {
    pub target_bot_id: String,
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

#[derive(Deserialize, Clone, Debug)]
pub struct WebAppUser {
    pub id: u64,
    pub first_name: String,
    pub last_name: String,
    pub username: Option<String>,
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
    pub debt: Option<i64>,
    pub blocked: Option<bool>,
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
    pub title: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DonationButton {
    pub name: String,
    pub description: String,
    pub amount: u32,
    pub source_url: String,
    pub invoice_url: String,
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum WSEvent {
    Success(Box<WSEventSuccess>),
    Error { ok: bool, error: String },
}

#[derive(Deserialize, Serialize)]
pub struct WSEventSuccess {
    pub ok: bool, // true
    #[serde(flatten)]
    pub data: WSEventData,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum WSEventData {
    Donation {
        from: String,
        total_amount: u32,
        invoice_payload: String,
        message: String,
    },
    GoalProps {
        props: Box<GoalProps>,
    },
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
    /// Encoded protobuf `ServerMessage`, delivered as a binary WS frame.
    Binary(Vec<u8>),
    CloseConnection(usize),
    CloseRoom(String),
}

#[derive(Deserialize)]
pub struct LayerQueryParams {
    pub t: String,
}

#[derive(Deserialize, Serialize)]
pub struct BotCommand {
    pub command: String,
    pub description: String,
}

#[derive(Deserialize)]
pub struct MakeTestDonationQueryParam {
    pub target_bot_id: String,
    pub amount: u32,
    pub media_source: String,
}

// Goal types

#[derive(Deserialize)]
pub struct GoalEnabledQueryParam {
    pub target_bot_id: String,
    pub enabled: bool,
}

#[derive(Deserialize)]
pub struct GoalPropsQueryParam {
    pub goal_config: GoalProps,
    pub target_bot_id: String,
}

// export const defaultProps: GoalProps = {
//     url: 'https://tg-stars.s3-website.nl-ams.scw.cloud/goal/some-uuid',
//     // 0 general settings
//     title: "Donation Goal Title",
//     maxLimit: 444,
//     progress: 106,

//     // 1 elements settings
//     titlePosition: "inside",

//     progressPosition: "inside",
//     progressType: "cur_stars_w_percent",

//     displayLimits: false,
//     minLimit: 0,

//     displayBackground: false,

//     isVertical: false,

//     // 2 progress bar design
//     barHeight: 29,
//     roundingRadius: 4,
//     barStrokeThickness: 0.4,
//     strokeColor: "rgba(255,0,0,0.91)",

//     // Background bar styling
//     bgBarColor: {
//       colorType: "solid",
//       color: "#424242",
//       },

//     // Progress bar styling
//     progressBarColor: {
//       colorType: "gradient",
//       color: "linear-gradient(0deg, #f57507,rgb(255, 248, 235))",
//       },

//     // 3 font settings
//     titleFontSettings: {
//       // 1 font
//       fontFamily: "Rubik",
//       color: "#f1f1f1",
//       style: [TextStyle.BOLD],
//       transformation: "uppercase",
//       horizontalAlignment: "center",

//       // 2 text
//       fontSize: 2.0,
//       lineHeight: 1,
//       letterSpacing: 0,
//       wordSpacing: 0,

//       // 3 shadow
//       shadowColor: "rgba(0, 0, 0, 0.24)",
//       shadowOffsetX: 0.3,
//       shadowOffsetY: 0.3,
//       shadowBlur: 0.5,
//     },
//   };

#[derive(Deserialize, Serialize)]
pub struct GoalProps {
    pub url: String,

    pub title: String,
    #[serde(rename = "maxLimit")]
    pub max_limit: u32,
    pub progress: f32,

    #[serde(rename = "titlePosition")]
    pub title_position: ElementPosition,
    #[serde(rename = "progressPosition")]
    pub progress_position: ElementPosition,
    #[serde(rename = "progressType")]
    pub progress_type: ProgressType,

    #[serde(rename = "displayLimits")]
    pub display_limits: bool,
    #[serde(rename = "minLimit")]
    pub min_limit: u32,
    #[serde(rename = "displayBackground")]
    pub display_background: bool,
    #[serde(rename = "isVertical")]
    pub is_vertical: bool,

    #[serde(rename = "barHeight")]
    pub bar_height: f32,
    #[serde(rename = "roundingRadius")]
    pub rounding_radius: f32,
    #[serde(rename = "barStrokeThickness")]
    pub bar_stroke_thickness: f32,
    #[serde(rename = "strokeColor")]
    pub stroke_color: String,

    #[serde(rename = "bgBarColor")]
    pub bg_bar_color: ColorSettings,
    #[serde(rename = "progressBarColor")]
    pub progress_bar_color: ColorSettings,

    #[serde(rename = "titleFontSettings")]
    pub title_font_settings: FontSettings,
    #[serde(rename = "progressBarFontSettings")]
    pub progress_bar_font_settings: FontSettings,
    #[serde(rename = "limitsFontSettings")]
    pub limits_font_settings: FontSettings,
}

#[derive(Deserialize, Serialize)]
pub enum ElementPosition {
    #[serde(rename = "top")]
    Top,
    #[serde(rename = "inside")]
    Inside,
    #[serde(rename = "below")]
    Below,
    #[serde(rename = "invisible")]
    Invisible,
}

#[derive(Deserialize, Serialize)]
pub enum ProgressType {
    #[serde(rename = "percent")]
    Percent,
    #[serde(rename = "cur_stars")]
    CurStars,
    #[serde(rename = "cur_stars_w_percent")]
    CurStarsWPercent,
    #[serde(rename = "cur_stars/target_stars")]
    CurStarsDivTargetStars,
    #[serde(rename = "cur_stars/target_stars_w_percent")]
    CurStarsDivTargetStarsWPercent,
}

#[derive(Deserialize, Serialize)]
pub struct ColorSettings {
    #[serde(rename = "colorType")]
    pub color_type: ColorType,
    pub color: String,
}

#[derive(Deserialize, Serialize)]
pub enum ColorType {
    #[serde(rename = "solid")]
    Solid,
    #[serde(rename = "gradient")]
    Gradient,
}

#[derive(Deserialize, Serialize)]
pub struct FontSettings {
    // font
    #[serde(rename = "fontFamily")]
    pub font_family: String,
    pub color: String,
    pub style: Vec<TextStyle>,
    pub transformation: Option<TextTransformation>,
    #[serde(rename = "horizontalAlignment")]
    pub horizontal_alignment: HorizontalAlignment,
    // text
    #[serde(rename = "fontSize")]
    pub font_size: f32,
    #[serde(rename = "lineHeight")]
    pub line_height: f32,
    #[serde(rename = "letterSpacing")]
    pub letter_spacing: f32,
    #[serde(rename = "wordSpacing")]
    pub word_spacing: f32,
    // shadow
    #[serde(rename = "shadowColor")]
    pub shadow_color: Option<String>,
    #[serde(rename = "shadowOffsetX")]
    pub shadow_offset_x: Option<f32>,
    #[serde(rename = "shadowOffsetY")]
    pub shadow_offset_y: Option<f32>,
    #[serde(rename = "shadowBlur")]
    pub shadow_blur: Option<f32>,
}

#[derive(Deserialize, Serialize)]
pub enum TextStyle {
    #[serde(rename = "bold")]
    Bold,
    #[serde(rename = "italic")]
    Italic,
}

#[derive(Deserialize, Serialize)]
pub enum TextTransformation {
    #[serde(rename = "uppercase")]
    Uppercase,
    #[serde(rename = "lowercase")]
    Lowercase,
}

#[derive(Deserialize, Serialize)]
pub enum HorizontalAlignment {
    #[serde(rename = "left")]
    Left,
    #[serde(rename = "center")]
    Center,
    #[serde(rename = "right")]
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_event() {
        let event = serde_json::to_string(&WSEvent::Success(Box::new(WSEventSuccess {
            ok: true,
            data: WSEventData::GoalProps {
                props: Box::new(GoalProps {
                    url: "https://example.com".to_string(),
                    title: "Test Goal".to_string(),
                    max_limit: 100,
                    progress: 50.0,
                    title_position: ElementPosition::Top,
                    progress_position: ElementPosition::Top,
                    progress_type: ProgressType::Percent,
                    display_limits: false,
                    min_limit: 0,
                    display_background: false,
                    is_vertical: false,
                    bar_height: 15.0,
                    rounding_radius: 4.0,
                    bar_stroke_thickness: 0.4,
                    stroke_color: "rgba(255,0,0,0.91)".to_string(),
                    bg_bar_color: ColorSettings {
                        color_type: ColorType::Solid,
                        color: "#424242".to_string(),
                    },
                    progress_bar_color: ColorSettings {
                        color_type: ColorType::Solid,
                        color: "rgb(255, 215, 0)".to_string(),
                    },
                    title_font_settings: FontSettings {
                        font_family: "Rubik".to_string(),
                        color: "#f1f1f1".to_string(),
                        style: vec![TextStyle::Bold],
                        horizontal_alignment: HorizontalAlignment::Center,
                        font_size: 2.75,
                        line_height: 1.0,
                        letter_spacing: 0.0,
                        word_spacing: 0.0,
                        transformation: None,
                        shadow_color: None,
                        shadow_offset_x: None,
                        shadow_offset_y: None,
                        shadow_blur: None,
                    },
                    limits_font_settings: FontSettings {
                        font_family: "Rubik".to_string(),
                        color: "#f1f1f1".to_string(),
                        style: vec![TextStyle::Bold],
                        horizontal_alignment: HorizontalAlignment::Center,
                        font_size: 2.75,
                        line_height: 1.0,
                        letter_spacing: 0.0,
                        word_spacing: 0.0,
                        transformation: None,
                        shadow_color: None,
                        shadow_offset_x: None,
                        shadow_offset_y: None,
                        shadow_blur: None,
                    },
                    progress_bar_font_settings: FontSettings {
                        font_family: "Rubik".to_string(),
                        color: "#f1f1f1".to_string(),
                        style: vec![TextStyle::Bold],
                        horizontal_alignment: HorizontalAlignment::Center,
                        font_size: 2.75,
                        line_height: 1.0,
                        letter_spacing: 0.0,
                        word_spacing: 0.0,
                        transformation: None,
                        shadow_color: None,
                        shadow_offset_x: None,
                        shadow_offset_y: None,
                        shadow_blur: None,
                    },
                }),
            },
        })))
        .unwrap();

        println!("event: {}", event);
    }

    #[test]
    fn test_goal_props() {
        let props = ColorSettings {
            color_type: ColorType::Gradient,
            color: "linear-gradient(0deg, #f57507,rgb(255, 248, 235))".to_string(),
        };
        let props_str = serde_json::to_string(&props).unwrap();
        println!("{}", props_str);
    }

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
