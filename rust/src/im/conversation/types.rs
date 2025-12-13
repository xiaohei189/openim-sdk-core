//! 会话 API DTO（请求和响应结构体）

use crate::im::types::LocalConversation;
use serde::Deserialize;

/// 增量会话响应（业务逻辑层结构体，可直接从 API 响应反序列化）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalConversationResp {
    pub version: u64,
    pub version_id: String,
    pub full: bool,
    pub delete: Vec<String>,
    pub insert: Vec<LocalConversation>,
    pub update: Vec<LocalConversation>,
}

/// 全量会话响应（业务逻辑层结构体，可直接从 API 响应反序列化）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllConversationsResp {
    pub conversations: Vec<LocalConversation>,
}
