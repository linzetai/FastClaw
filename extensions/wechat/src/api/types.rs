use serde::{Deserialize, Serialize};

// ── Message item type constants ──────────────────────────────────────────────

pub const MSG_ITEM_TYPE_TEXT: u32 = 1;
pub const MSG_ITEM_TYPE_IMAGE: u32 = 2;
pub const MSG_ITEM_TYPE_VOICE: u32 = 3;
pub const MSG_ITEM_TYPE_FILE: u32 = 4;
pub const MSG_ITEM_TYPE_VIDEO: u32 = 5;

pub const MESSAGE_TYPE_USER: u32 = 1;
pub const MESSAGE_TYPE_BOT: u32 = 2;

pub const MESSAGE_STATE_NEW: u32 = 0;
pub const MESSAGE_STATE_GENERATING: u32 = 1;
pub const MESSAGE_STATE_FINISH: u32 = 2;

pub const TYPING_STATUS_TYPING: u32 = 1;
pub const TYPING_STATUS_CANCEL: u32 = 2;

pub const UPLOAD_MEDIA_TYPE_IMAGE: u32 = 1;
pub const UPLOAD_MEDIA_TYPE_VIDEO: u32 = 2;
pub const UPLOAD_MEDIA_TYPE_FILE: u32 = 3;
pub const UPLOAD_MEDIA_TYPE_VOICE: u32 = 4;

// ── Common ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot_agent: Option<String>,
}

// ── CDN Media ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CDNMedia {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypt_query_param: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aes_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypt_type: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_url: Option<String>,
}

// ── Message Items ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TextItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<CDNMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb_media: Option<CDNMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aeskey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mid_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb_height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hd_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VoiceItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<CDNMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encode_type: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bits_per_sample: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playtime: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<CDNMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub len: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VideoItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<CDNMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub play_length: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_md5: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb_media: Option<CDNMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb_height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb_width: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RefMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_item: Option<Box<MessageItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageItem {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub item_type: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_time_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_completed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_msg: Option<RefMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_item: Option<TextItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_item: Option<ImageItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_item: Option<VoiceItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_item: Option<FileItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_item: Option<VideoItem>,
}

// ── Unified message ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WeixinMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_time_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_time_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_type: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_state: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_list: Option<Vec<MessageItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_token: Option<String>,
}

// ── API Request / Response types ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUpdatesReq {
    #[serde(default)]
    pub get_updates_buf: String,
    pub base_info: BaseInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetUpdatesResp {
    #[serde(default)]
    pub ret: Option<i32>,
    #[serde(default)]
    pub errcode: Option<i32>,
    #[serde(default)]
    pub errmsg: Option<String>,
    #[serde(default)]
    pub msgs: Option<Vec<WeixinMessage>>,
    #[serde(default)]
    pub get_updates_buf: Option<String>,
    #[serde(default)]
    pub longpolling_timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageReq {
    pub msg: WeixinMessage,
    pub base_info: BaseInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUploadUrlReq {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filekey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rawsize: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rawfilemd5: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filesize: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb_rawsize: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb_rawfilemd5: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb_filesize: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_need_thumb: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aeskey: Option<String>,
    pub base_info: BaseInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetUploadUrlResp {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upload_param: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb_upload_param: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upload_full_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetConfigReq {
    pub ilink_user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_token: Option<String>,
    pub base_info: BaseInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetConfigResp {
    #[serde(default)]
    pub ret: Option<i32>,
    #[serde(default)]
    pub errmsg: Option<String>,
    #[serde(default)]
    pub typing_ticket: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendTypingReq {
    pub ilink_user_id: String,
    pub typing_ticket: String,
    pub status: u32,
    pub base_info: BaseInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyReq {
    pub base_info: BaseInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotifyResp {
    #[serde(default)]
    pub ret: Option<i32>,
    #[serde(default)]
    pub errmsg: Option<String>,
}

// ── QR Login types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrLoginStartReq {
    #[serde(default)]
    pub local_token_list: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrCodeResponse {
    #[serde(default)]
    pub qrcode: String,
    #[serde(default)]
    pub qrcode_img_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrStatusResponse {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub bot_token: Option<String>,
    #[serde(default)]
    pub ilink_bot_id: Option<String>,
    #[serde(default)]
    pub baseurl: Option<String>,
    #[serde(default)]
    pub ilink_user_id: Option<String>,
    #[serde(default)]
    pub redirect_host: Option<String>,
}
