//! 消息类型定义
//!
//! 定义了 OpenIM 消息的各种元素类型，对应 Go 版本的 `pkg/apistruct/msg.go`

use serde::{Deserialize, Serialize};

/// 图片基础信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PictureBaseInfo {
    #[serde(rename = "uuid")]
    pub uuid: String,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(rename = "size")]
    pub size: i64,
    #[serde(rename = "width")]
    pub width: i32,
    #[serde(rename = "height")]
    pub height: i32,
    #[serde(rename = "url")]
    pub url: String,
}

/// 图片元素
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PictureElem {
    #[serde(rename = "sourcePath")]
    pub source_path: String,
    #[serde(rename = "sourcePicture")]
    pub source_picture: PictureBaseInfo,
    #[serde(rename = "bigPicture")]
    pub big_picture: PictureBaseInfo,
    #[serde(rename = "snapshotPicture")]
    pub snapshot_picture: PictureBaseInfo,
}

/// 语音元素
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundElem {
    #[serde(rename = "uuid")]
    pub uuid: String,
    #[serde(rename = "soundPath")]
    pub sound_path: String,
    #[serde(rename = "sourceUrl")]
    pub source_url: String,
    #[serde(rename = "dataSize")]
    pub data_size: i64,
    #[serde(rename = "duration")]
    pub duration: i64,
}

/// 视频元素
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoElem {
    #[serde(rename = "videoPath")]
    pub video_path: String,
    #[serde(rename = "videoUUID")]
    pub video_uuid: String,
    #[serde(rename = "videoUrl")]
    pub video_url: String,
    #[serde(rename = "videoType")]
    pub video_type: String,
    #[serde(rename = "videoSize")]
    pub video_size: i64,
    #[serde(rename = "duration")]
    pub duration: i64,
    #[serde(rename = "snapshotPath")]
    pub snapshot_path: String,
    #[serde(rename = "snapshotUUID")]
    pub snapshot_uuid: String,
    #[serde(rename = "snapshotSize")]
    pub snapshot_size: i64,
    #[serde(rename = "snapshotUrl")]
    pub snapshot_url: String,
    #[serde(rename = "snapshotWidth")]
    pub snapshot_width: i32,
    #[serde(rename = "snapshotHeight")]
    pub snapshot_height: i32,
}

/// 文件元素
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileElem {
    #[serde(rename = "filePath")]
    pub file_path: String,
    #[serde(rename = "uuid")]
    pub uuid: String,
    #[serde(rename = "sourceUrl")]
    pub source_url: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "fileSize")]
    pub file_size: i64,
}

/// @ 元素
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtElem {
    #[serde(rename = "text")]
    pub text: String,
    #[serde(rename = "atUserList")]
    pub at_user_list: Vec<String>,
    #[serde(rename = "atUsersInfo")]
    pub at_users_info: Option<Vec<AtInfo>>,
    #[serde(rename = "quoteMessage")]
    pub quote_message: Option<Box<MsgStruct>>,
    #[serde(rename = "isAtSelf")]
    pub is_at_self: bool,
}

/// 位置元素
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationElem {
    #[serde(rename = "description")]
    pub description: String,
    #[serde(rename = "longitude")]
    pub longitude: f64,
    #[serde(rename = "latitude")]
    pub latitude: f64,
}

/// 自定义元素
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomElem {
    #[serde(rename = "data")]
    pub data: String,
    #[serde(rename = "description")]
    pub description: String,
    #[serde(rename = "extension")]
    pub extension: String,
}

/// 文本元素
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextElem {
    #[serde(rename = "content")]
    pub content: String,
}

/// Markdown 文本元素
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownTextElem {
    #[serde(rename = "content")]
    pub content: String,
}

/// Markdown + 实体（扩展用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownEntityElem {
    #[serde(rename = "content")]
    pub content: String,
    #[serde(rename = "messageEntityList", skip_serializing_if = "Option::is_none")]
    pub message_entity_list: Option<String>,
}

/// 流式消息元素
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMsgElem {
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(rename = "content")]
    pub content: String,
}

/// 撤回元素
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeElem {
    #[serde(rename = "revokeMsgClientID")]
    pub revoke_msg_client_id: String,
}

/// 引用元素
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteElem {
    #[serde(rename = "text", skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(rename = "quoteMessage", skip_serializing_if = "Option::is_none")]
    pub quote_message: Option<Box<MsgStruct>>,
}

/// OA 通知元素
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OANotificationElem {
    #[serde(rename = "notificationName")]
    pub notification_name: String,
    #[serde(rename = "notificationFaceURL")]
    pub notification_face_url: String,
    #[serde(rename = "notificationType")]
    pub notification_type: i32,
    #[serde(rename = "text")]
    pub text: String,
    #[serde(rename = "url")]
    pub url: String,
    #[serde(rename = "mixType")]
    pub mix_type: i32,
    #[serde(rename = "pictureElem", skip_serializing_if = "Option::is_none")]
    pub picture_elem: Option<PictureElem>,
    #[serde(rename = "soundElem", skip_serializing_if = "Option::is_none")]
    pub sound_elem: Option<SoundElem>,
    #[serde(rename = "videoElem", skip_serializing_if = "Option::is_none")]
    pub video_elem: Option<VideoElem>,
    #[serde(rename = "fileElem", skip_serializing_if = "Option::is_none")]
    pub file_elem: Option<FileElem>,
    #[serde(rename = "ex")]
    pub ex: String,
}

/// 消息撤回信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRevoked {
    #[serde(rename = "revokerID")]
    pub revoker_id: String,
    #[serde(rename = "revokerRole")]
    pub revoker_role: i32,
    #[serde(rename = "clientMsgID")]
    pub client_msg_id: String,
    #[serde(rename = "revokerNickname")]
    pub revoker_nickname: String,
    #[serde(rename = "sessionType")]
    pub session_type: i32,
    #[serde(rename = "seq")]
    pub seq: u32,
}

/// 消息结构体（对应 Go 的 MsgStruct）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsgStruct {
    #[serde(rename = "clientMsgID", skip_serializing_if = "Option::is_none")]
    pub client_msg_id: Option<String>,
    #[serde(rename = "serverMsgID", skip_serializing_if = "Option::is_none")]
    pub server_msg_id: Option<String>,
    #[serde(rename = "createTime")]
    pub create_time: i64,
    #[serde(rename = "sendTime")]
    pub send_time: i64,
    #[serde(rename = "sessionType")]
    pub session_type: i32,
    #[serde(rename = "sendID", skip_serializing_if = "Option::is_none")]
    pub send_id: Option<String>,
    #[serde(rename = "recvID", skip_serializing_if = "Option::is_none")]
    pub recv_id: Option<String>,
    #[serde(rename = "msgFrom")]
    pub msg_from: i32,
    #[serde(rename = "contentType")]
    pub content_type: i32,
    #[serde(rename = "senderPlatformID")]
    pub sender_platform_id: i32,
    #[serde(rename = "senderNickname", skip_serializing_if = "Option::is_none")]
    pub sender_nickname: Option<String>,
    #[serde(rename = "senderFaceUrl", skip_serializing_if = "Option::is_none")]
    pub sender_face_url: Option<String>,
    #[serde(rename = "groupID", skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    #[serde(rename = "content", skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(rename = "seq")]
    pub seq: i64,
    #[serde(rename = "isRead")]
    pub is_read: bool,
    #[serde(rename = "status")]
    pub status: i32,
    #[serde(rename = "isReact", skip_serializing_if = "Option::is_none")]
    pub is_react: Option<bool>,
    #[serde(rename = "isExternalExtensions", skip_serializing_if = "Option::is_none")]
    pub is_external_extensions: Option<bool>,
    // OfflinePushInfo 是 protobuf 生成的结构体，不支持 serde 序列化
    // 如果需要序列化，可以通过 protobuf 的 encode/decode 方法处理
    #[serde(skip)]
    pub offline_push: Option<openim_protocol::sdkws::OfflinePushInfo>,
    #[serde(rename = "attachedInfo", skip_serializing_if = "Option::is_none")]
    pub attached_info: Option<String>,
    #[serde(rename = "ex", skip_serializing_if = "Option::is_none")]
    pub ex: Option<String>,
    #[serde(rename = "localEx", skip_serializing_if = "Option::is_none")]
    pub local_ex: Option<String>,
    #[serde(rename = "textElem", skip_serializing_if = "Option::is_none")]
    pub text_elem: Option<TextElem>,
    #[serde(rename = "pictureElem", skip_serializing_if = "Option::is_none")]
    pub picture_elem: Option<PictureElem>,
    #[serde(rename = "soundElem", skip_serializing_if = "Option::is_none")]
    pub sound_elem: Option<SoundElem>,
    #[serde(rename = "videoElem", skip_serializing_if = "Option::is_none")]
    pub video_elem: Option<VideoElem>,
    #[serde(rename = "fileElem", skip_serializing_if = "Option::is_none")]
    pub file_elem: Option<FileElem>,
    #[serde(rename = "atTextElem", skip_serializing_if = "Option::is_none")]
    pub at_text_elem: Option<AtElem>,
    #[serde(rename = "locationElem", skip_serializing_if = "Option::is_none")]
    pub location_elem: Option<LocationElem>,
    #[serde(rename = "customElem", skip_serializing_if = "Option::is_none")]
    pub custom_elem: Option<CustomElem>,
    #[serde(rename = "quoteElem", skip_serializing_if = "Option::is_none")]
    pub quote_elem: Option<QuoteElem>,
}

/// @ 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtInfo {
    #[serde(rename = "atUserID", skip_serializing_if = "Option::is_none")]
    pub at_user_id: Option<String>,
    #[serde(rename = "groupNickname", skip_serializing_if = "Option::is_none")]
    pub group_nickname: Option<String>,
}
