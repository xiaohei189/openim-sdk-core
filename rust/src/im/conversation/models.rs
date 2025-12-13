//! 会话本地模型定义

use serde::{Deserialize, Serialize};

/// 版本同步信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalVersionSync {
    /// 表名
    #[serde(rename = "tableName")]
    pub table_name: String,
    /// 实体 ID（用户 ID）
    #[serde(rename = "entityID")]
    pub entity_id: String,
    /// 版本号
    pub version: u64,
    /// 版本 ID
    #[serde(rename = "versionID")]
    pub version_id: String,
}

/// 会话同步器配置
pub struct ConversationSyncerConfig {
    /// 用户 ID
    pub user_id: String,
    /// API 基础 URL
    pub api_base_url: String,
    /// Token
    pub token: String,
    /// 数据库路径（SQLite），可以是：
    /// - 相对路径：如 "conversations.db" 会转换为 "sqlite://conversations.db"
    /// - 绝对路径：如 "/path/to/db.db" 会转换为 "sqlite:///path/to/db.db"
    /// - 完整URL：如 "sqlite://conversations.db" 直接使用
    pub db_path: String,
}

impl ConversationSyncerConfig {}
