//! 好友 API DTO（请求和响应结构体）

use crate::im::friend::models::LocalFriend;
use serde::{Deserialize, Deserializer, Serialize};

/// 反序列化数组字段，处理 null 值
pub(crate) fn deserialize_vec_or_null<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let opt = Option::<Vec<T>>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

/// 增量好友响应（业务逻辑层结构体，可直接从 API 响应反序列化）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalFriendsResp {
    pub full: bool,
    pub version: u64,
    #[serde(rename = "versionID")]
    pub version_id: String,
    #[serde(deserialize_with = "deserialize_vec_or_null")]
    pub delete: Vec<String>,
    #[serde(deserialize_with = "deserialize_vec_or_null")]
    pub insert: Vec<LocalFriend>,
    #[serde(deserialize_with = "deserialize_vec_or_null")]
    pub update: Vec<LocalFriend>,
}

/// 全量好友响应（业务逻辑层结构体，可直接从 API 响应反序列化）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllFriendsResp {
    #[serde(deserialize_with = "deserialize_vec_or_null")]
    pub friends_info: Vec<LocalFriend>,
}

/// 好友申请信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FriendRequest {
    #[serde(rename = "fromUserID")]
    pub from_user_id: String,
    #[serde(rename = "fromNickname")]
    pub from_nickname: String,
    #[serde(rename = "fromFaceURL")]
    pub from_face_url: String,
    #[serde(rename = "toUserID")]
    pub to_user_id: String,
    #[serde(rename = "toNickname")]
    pub to_nickname: String,
    #[serde(rename = "toFaceURL")]
    pub to_face_url: String,
    #[serde(rename = "handleResult")]
    pub handle_result: i32,
    #[serde(rename = "reqMsg")]
    pub req_msg: String,
    #[serde(rename = "createTime")]
    pub create_time: i64,
    #[serde(rename = "handlerUserID")]
    pub handler_user_id: String,
    #[serde(rename = "handleMsg")]
    pub handle_msg: String,
    #[serde(rename = "handleTime")]
    pub handle_time: i64,
    pub ex: String,
}

/// 好友申请列表响应
#[derive(Debug, Clone, Deserialize)]
pub struct FriendRequestsResp {
    #[serde(rename = "FriendRequests")]
    #[serde(deserialize_with = "deserialize_vec_or_null")]
    pub friend_requests: Vec<FriendRequest>,
    #[serde(default)]
    pub total: Option<i32>,
}
