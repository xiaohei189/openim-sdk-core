//! 好友本地模型定义

use serde::{Deserialize, Serialize};

/// 本地好友数据结构（与 Go 的 LocalFriend 字段基本对应）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalFriend {
    #[serde(rename = "ownerUserID")]
    pub owner_user_id: String,
    #[serde(rename = "userID")]
    pub friend_user_id: String,
    #[serde(rename = "remark")]
    pub remark: String,
    #[serde(rename = "createTime")]
    pub create_time: i64,
    #[serde(rename = "addSource")]
    pub add_source: i32,
    #[serde(rename = "operatorUserID")]
    pub operator_user_id: String,
    #[serde(rename = "nickname")]
    pub nickname: String,
    #[serde(rename = "faceURL")]
    pub face_url: String,
    #[serde(rename = "ex")]
    pub ex: String,
    #[serde(rename = "attachedInfo")]
    pub attached_info: String,
    #[serde(rename = "isPinned")]
    pub is_pinned: bool,
}

/// 黑名单数据结构（与好友结构类似）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackList {
    #[serde(rename = "ownerUserID")]
    pub owner_user_id: String,
    #[serde(rename = "blockUserID")]
    pub block_user_id: String,
    #[serde(rename = "createTime")]
    pub create_time: i64,
    #[serde(rename = "addSource")]
    pub add_source: i32,
    #[serde(rename = "operatorUserID")]
    pub operator_user_id: String,
    #[serde(rename = "nickname")]
    pub nickname: String,
    #[serde(rename = "faceURL")]
    pub face_url: String,
    #[serde(rename = "ex")]
    pub ex: String,
    #[serde(rename = "attachedInfo")]
    pub attached_info: String,
}

/// 好友同步器配置
pub struct FriendSyncerConfig {
    /// 用户 ID
    pub user_id: String,
    /// API 基础 URL
    pub api_base_url: String,
    /// Token
    pub token: String,
    /// 数据库路径（SQLite），与会话共用同一个文件即可
    pub db_path: String,
}
