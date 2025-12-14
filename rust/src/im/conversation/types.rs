//! 会话 API DTO（请求和响应结构体）

use crate::im::types::LocalConversation;
use serde::{Deserialize, Deserializer};

/// 反序列化数组字段，处理 null 值
fn deserialize_vec_or_null<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let opt = Option::<Vec<T>>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

/// 增量会话响应（业务逻辑层结构体，可直接从 API 响应反序列化）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalConversationResp {
    pub version: u64,
    #[serde(rename = "versionID")]
    pub version_id: String,
    pub full: bool,
    #[serde(deserialize_with = "deserialize_vec_or_null")]
    pub delete: Vec<String>,
    #[serde(deserialize_with = "deserialize_vec_or_null")]
    pub insert: Vec<LocalConversation>,
    #[serde(deserialize_with = "deserialize_vec_or_null")]
    pub update: Vec<LocalConversation>,
}

/// 全量会话响应（业务逻辑层结构体，可直接从 API 响应反序列化）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllConversationsResp {
    #[serde(deserialize_with = "deserialize_vec_or_null")]
    pub conversations: Vec<LocalConversation>,
}
