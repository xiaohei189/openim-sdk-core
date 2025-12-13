//! 会话同步模块
//!
//! 实现 OpenIM SDK 的会话增量同步逻辑，参考 Go 版本的实现

use crate::im::entities::local_conversations;
use anyhow::{Context, Result};
use async_trait::async_trait;
use openim_protocol::constant;
use openim_protocol::conversation;
use openim_protocol::sdkws;
use sea_orm::{
    ColumnTrait, ConnectOptions, Database, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// 本地会话数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConversation {
    /// 会话 ID
    #[serde(rename = "conversationID")]
    pub conversation_id: String,
    /// 会话类型：1=单聊, 2=普通群聊, 3=超级群聊, 4=通知会话
    #[serde(rename = "conversationType")]
    pub conversation_type: i32,
    /// 用户 ID（单聊时使用）
    #[serde(rename = "userID")]
    pub user_id: String,
    /// 群组 ID（群聊时使用）
    #[serde(rename = "groupID")]
    pub group_id: String,
    /// 显示名称
    #[serde(rename = "showName")]
    pub show_name: String,
    /// 头像 URL
    #[serde(rename = "faceURL")]
    pub face_url: String,
    /// 最新消息
    #[serde(rename = "latestMsg")]
    pub latest_msg: String,
    /// 最新消息发送时间
    #[serde(rename = "latestMsgSendTime")]
    pub latest_msg_send_time: i64,
    /// 未读消息数
    #[serde(rename = "unreadCount")]
    pub unread_count: i32,
    /// 接收消息选项：0=接收并通知, 1=接收不通知, 2=屏蔽
    #[serde(rename = "recvMsgOpt")]
    pub recv_msg_opt: i32,
    /// 是否置顶
    #[serde(rename = "isPinned")]
    pub is_pinned: bool,
    /// 是否私聊
    #[serde(rename = "isPrivateChat")]
    pub is_private_chat: bool,
    /// 阅后即焚时长（秒）
    #[serde(rename = "burnDuration")]
    pub burn_duration: i32,
    /// 群@类型
    #[serde(rename = "groupAtType")]
    pub group_at_type: i32,
    /// 是否不在群内
    #[serde(rename = "isNotInGroup")]
    pub is_not_in_group: bool,
    /// 更新未读数时间
    #[serde(rename = "updateUnreadCountTime")]
    pub update_unread_count_time: i64,
    /// 附加信息
    #[serde(rename = "attachedInfo")]
    pub attached_info: String,
    /// 扩展信息
    #[serde(rename = "ex")]
    pub ex: String,
    /// 草稿文本
    #[serde(rename = "draftText")]
    pub draft_text: String,
    /// 草稿文本时间
    #[serde(rename = "draftTextTime")]
    pub draft_text_time: i64,
    /// 最大序列号
    #[serde(rename = "maxSeq")]
    pub max_seq: i64,
    /// 最小序列号
    #[serde(rename = "minSeq")]
    pub min_seq: i64,
    /// 是否消息销毁
    #[serde(rename = "isMsgDestruct")]
    pub is_msg_destruct: bool,
    /// 消息销毁时间
    #[serde(rename = "msgDestructTime")]
    pub msg_destruct_time: i64,
}

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

/// 会话监听器回调接口（对应 Go 版本的 OnConversationListener）
#[async_trait]
pub trait ConversationListener: Send + Sync {
    /// 同步服务器开始
    async fn on_sync_server_start(&self, reinstalled: bool);

    /// 同步服务器完成
    async fn on_sync_server_finish(&self, reinstalled: bool);

    /// 同步服务器进度
    async fn on_sync_server_progress(&self, progress: i32);

    /// 同步服务器失败
    async fn on_sync_server_failed(&self, reinstalled: bool);

    /// 新会话
    async fn on_new_conversation(&self, conversation_list: String);

    /// 会话变更
    async fn on_conversation_changed(&self, conversation_list: String);

    /// 总未读消息数变更
    async fn on_total_unread_message_count_changed(&self, total_unread_count: i32);

    /// 会话用户输入状态变更
    async fn on_conversation_user_input_status_changed(&self, change: String);
}

/// 空实现（默认监听器）
pub struct EmptyConversationListener;

#[async_trait]
impl ConversationListener for EmptyConversationListener {
    async fn on_sync_server_start(&self, _reinstalled: bool) {}
    async fn on_sync_server_finish(&self, _reinstalled: bool) {}
    async fn on_sync_server_progress(&self, _progress: i32) {}
    async fn on_sync_server_failed(&self, _reinstalled: bool) {}
    async fn on_new_conversation(&self, _conversation_list: String) {}
    async fn on_conversation_changed(&self, _conversation_list: String) {}
    async fn on_total_unread_message_count_changed(&self, _total_unread_count: i32) {}
    async fn on_conversation_user_input_status_changed(&self, _change: String) {}
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

/// 会话同步器
pub struct ConversationSyncer {
    config: ConversationSyncerConfig,
    /// HTTP 客户端
    client: reqwest::Client,
    /// 数据库连接
    db: DatabaseConnection,
    /// 会话监听器
    listener: Arc<dyn ConversationListener>,
}

impl ConversationSyncer {
    /// 创建新的会话同步器（使用默认空监听器）
    pub async fn new(config: ConversationSyncerConfig) -> Result<Self> {
        Self::with_listener(config, Arc::new(EmptyConversationListener)).await
    }

    /// 创建新的会话同步器（带自定义监听器）
    pub async fn with_listener(
        config: ConversationSyncerConfig,
        listener: Arc<dyn ConversationListener>,
    ) -> Result<Self> {
        // 构建SQLite数据库连接URL
        let db_url = config.db_path.clone();
        info!(
            "[ConvSync/DB] 创建会话同步器，用户ID: {}, SQLite数据库: {}",
            config.user_id, db_url
        );
        let mut opt = ConnectOptions::new(db_url.clone());
        opt.sqlx_logging(false);
        // 创建SQLite数据库连接
        let db = Database::connect(opt)
            .await
            .context(format!("连接SQLite数据库失败: {}", db_url))?;

        // 初始化数据库表
        let syncer = Self {
            client: reqwest::Client::new(),
            db: db.clone(),
            listener,
            config,
        };

        syncer.init_db().await?;
        Ok(syncer)
    }

    /// 初始化数据库表结构
    async fn init_db(&self) -> Result<()> {
        info!("[ConvSync/DB] 初始化数据库表结构");

        // 使用Sea-ORM的Schema创建表
        use sea_orm::ConnectionTrait;

        let sql1 = r#"
            CREATE TABLE IF NOT EXISTS local_conversations (
                conversation_id TEXT PRIMARY KEY,
                conversation_type INTEGER NOT NULL,
                user_id TEXT NOT NULL DEFAULT '',
                group_id TEXT NOT NULL DEFAULT '',
                show_name TEXT NOT NULL DEFAULT '',
                face_url TEXT NOT NULL DEFAULT '',
                latest_msg TEXT NOT NULL DEFAULT '',
                latest_msg_send_time INTEGER NOT NULL DEFAULT 0,
                unread_count INTEGER NOT NULL DEFAULT 0,
                recv_msg_opt INTEGER NOT NULL DEFAULT 0,
                is_pinned INTEGER NOT NULL DEFAULT 0,
                is_private_chat INTEGER NOT NULL DEFAULT 0,
                burn_duration INTEGER NOT NULL DEFAULT 0,
                group_at_type INTEGER NOT NULL DEFAULT 0,
                is_not_in_group INTEGER NOT NULL DEFAULT 0,
                update_unread_count_time INTEGER NOT NULL DEFAULT 0,
                attached_info TEXT NOT NULL DEFAULT '',
                ex TEXT NOT NULL DEFAULT '',
                draft_text TEXT NOT NULL DEFAULT '',
                draft_text_time INTEGER NOT NULL DEFAULT 0,
                max_seq INTEGER NOT NULL DEFAULT 0,
                min_seq INTEGER NOT NULL DEFAULT 0,
                is_msg_destruct INTEGER NOT NULL DEFAULT 0,
                msg_destruct_time INTEGER NOT NULL DEFAULT 0
            )
        "#;
        self.db
            .execute_unprepared(sql1)
            .await
            .context("创建会话表失败")?;

        let sql2 = r#"
            CREATE TABLE IF NOT EXISTS local_version_sync (
                table_name TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                version INTEGER NOT NULL DEFAULT 0,
                version_id TEXT NOT NULL DEFAULT '',
                PRIMARY KEY (table_name, entity_id)
            )
        "#;
        self.db
            .execute_unprepared(sql2)
            .await
            .context("创建版本同步表失败")?;

        info!("[ConvSync/DB] 数据库表初始化完成");
        Ok(())
    }

    /// 从数据库获取所有本地会话
    pub async fn get_all_conversations(&self) -> Result<Vec<LocalConversation>> {
        let models = local_conversations::Entity::find()
            .all(&self.db)
            .await
            .context("查询会话列表失败")?;

        let conversations: Vec<LocalConversation> = models
            .into_iter()
            .map(|model| LocalConversation {
                conversation_id: model.conversation_id,
                conversation_type: model.conversation_type,
                user_id: model.user_id,
                group_id: model.group_id,
                show_name: model.show_name,
                face_url: model.face_url,
                latest_msg: model.latest_msg,
                latest_msg_send_time: model.latest_msg_send_time,
                unread_count: model.unread_count,
                recv_msg_opt: model.recv_msg_opt,
                is_pinned: model.is_pinned != 0,
                is_private_chat: model.is_private_chat != 0,
                burn_duration: model.burn_duration,
                group_at_type: model.group_at_type,
                is_not_in_group: model.is_not_in_group != 0,
                update_unread_count_time: model.update_unread_count_time,
                attached_info: model.attached_info,
                ex: model.ex,
                draft_text: model.draft_text,
                draft_text_time: model.draft_text_time,
                max_seq: model.max_seq,
                min_seq: model.min_seq,
                is_msg_destruct: model.is_msg_destruct != 0,
                msg_destruct_time: model.msg_destruct_time,
            })
            .collect();

        debug!(
            "[ConvSync/DB] 获取本地会话列表，共 {} 个会话",
            conversations.len()
        );
        Ok(conversations)
    }

    /// 从数据库获取所有会话 ID
    pub async fn get_all_conversation_ids(&self) -> Result<Vec<String>> {
        let models = local_conversations::Entity::find()
            .all(&self.db)
            .await
            .context("查询会话ID列表失败")?;

        let ids: Vec<String> = models
            .into_iter()
            .map(|model| model.conversation_id)
            .collect();

        debug!("[ConvSync/DB] 获取本地会话ID列表，共 {} 个", ids.len());
        Ok(ids)
    }

    /// 从数据库获取版本同步信息
    async fn get_version_sync(&self) -> Result<Option<LocalVersionSync>> {
        use crate::im::entities::local_version_sync::{Column, Entity};

        let model = Entity::find()
            .filter(Column::TableName.eq("local_conversations"))
            .filter(Column::EntityId.eq(&self.config.user_id))
            .one(&self.db)
            .await
            .context("查询版本同步信息失败")?;

        if let Some(model) = model {
            Ok(Some(LocalVersionSync {
                table_name: model.table_name,
                entity_id: model.entity_id,
                version: model.version as u64,
                version_id: model.version_id,
            }))
        } else {
            Ok(None)
        }
    }

    /// 保存版本同步信息到数据库
    async fn save_version_sync(&self, version_sync: &LocalVersionSync) -> Result<()> {
        use crate::im::entities::local_version_sync::{ActiveModel, Column, Entity};

        let active_model = ActiveModel {
            table_name: Set(version_sync.table_name.clone()),
            entity_id: Set(version_sync.entity_id.clone()),
            version: Set(version_sync.version as i64),
            version_id: Set(version_sync.version_id.clone()),
        };

        Entity::insert(active_model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::columns([Column::TableName, Column::EntityId])
                    .update_columns([Column::Version, Column::VersionId])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .context("保存版本同步信息失败")?;
        Ok(())
    }

    /// 插入或更新会话到数据库
    async fn upsert_conversation(&self, conv: &LocalConversation) -> Result<()> {
        use crate::im::entities::local_conversations::ActiveModel;

        let active_model = ActiveModel {
            conversation_id: Set(conv.conversation_id.clone()),
            conversation_type: Set(conv.conversation_type),
            user_id: Set(conv.user_id.clone()),
            group_id: Set(conv.group_id.clone()),
            show_name: Set(conv.show_name.clone()),
            face_url: Set(conv.face_url.clone()),
            latest_msg: Set(conv.latest_msg.clone()),
            latest_msg_send_time: Set(conv.latest_msg_send_time),
            unread_count: Set(conv.unread_count),
            recv_msg_opt: Set(conv.recv_msg_opt),
            is_pinned: Set(if conv.is_pinned { 1 } else { 0 }),
            is_private_chat: Set(if conv.is_private_chat { 1 } else { 0 }),
            burn_duration: Set(conv.burn_duration),
            group_at_type: Set(conv.group_at_type),
            is_not_in_group: Set(if conv.is_not_in_group { 1 } else { 0 }),
            update_unread_count_time: Set(conv.update_unread_count_time),
            attached_info: Set(conv.attached_info.clone()),
            ex: Set(conv.ex.clone()),
            draft_text: Set(conv.draft_text.clone()),
            draft_text_time: Set(conv.draft_text_time),
            max_seq: Set(conv.max_seq),
            min_seq: Set(conv.min_seq),
            is_msg_destruct: Set(if conv.is_msg_destruct { 1 } else { 0 }),
            msg_destruct_time: Set(conv.msg_destruct_time),
        };

        local_conversations::Entity::insert(active_model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(local_conversations::Column::ConversationId)
                    .update_columns([
                        local_conversations::Column::ConversationType,
                        local_conversations::Column::UserId,
                        local_conversations::Column::GroupId,
                        local_conversations::Column::ShowName,
                        local_conversations::Column::FaceUrl,
                        local_conversations::Column::LatestMsg,
                        local_conversations::Column::LatestMsgSendTime,
                        local_conversations::Column::UnreadCount,
                        local_conversations::Column::RecvMsgOpt,
                        local_conversations::Column::IsPinned,
                        local_conversations::Column::IsPrivateChat,
                        local_conversations::Column::BurnDuration,
                        local_conversations::Column::GroupAtType,
                        local_conversations::Column::IsNotInGroup,
                        local_conversations::Column::UpdateUnreadCountTime,
                        local_conversations::Column::AttachedInfo,
                        local_conversations::Column::Ex,
                        local_conversations::Column::DraftText,
                        local_conversations::Column::DraftTextTime,
                        local_conversations::Column::MaxSeq,
                        local_conversations::Column::MinSeq,
                        local_conversations::Column::IsMsgDestruct,
                        local_conversations::Column::MsgDestructTime,
                    ])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .context("插入或更新会话失败")?;
        Ok(())
    }

    /// 根据消息内容生成 latestMsg 摘要（仿 Go 版 SDK 的简化实现）
    fn build_latest_msg_summary(msg: &sdkws::MsgData) -> String {
        // 文本消息：尽量展示正文
        if msg.content_type == constant::TEXT {
            if let Ok(s) = String::from_utf8(msg.content.clone()) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&s) {
                    if let Some(text) = json.get("content").and_then(|v| v.as_str()) {
                        if !text.is_empty() {
                            return text.to_string();
                        }
                    }
                }
                if !s.is_empty() {
                    return s;
                }
            }
            return "[文本]".to_string();
        }

        // 其他常见内容类型：按类型给固定文案
        match msg.content_type {
            t if t == constant::PICTURE => "[图片]".to_string(),
            t if t == constant::VOICE => "[语音]".to_string(),
            t if t == constant::VIDEO => "[视频]".to_string(),
            t if t == constant::FILE => "[文件]".to_string(),
            t if t == constant::AT_TEXT => "[@消息]".to_string(),
            t if t == constant::LOCATION => "[位置]".to_string(),
            t if t == constant::MERGER => "[聊天记录]".to_string(),
            t if t == constant::CARD => "[名片]".to_string(),

            // 好友相关通知
            1201 | 1203 | 1204 => "[好友通知]".to_string(),
            // 群相关通知（部分示例）
            1501 | 1504 | 1508 => "[群通知]".to_string(),
            // 已读回执
            2200 => "[已读回执]".to_string(),

            // 兜底
            _ => "[新消息]".to_string(),
        }
    }

    /// 基于新消息/通知实时更新会话（未读数、最新消息等）
    pub async fn on_new_message(
        &self,
        conversation_id: &str,
        msg: &sdkws::MsgData,
        is_notification: bool,
    ) -> Result<()> {
        use crate::im::entities::local_conversations::{Column, Entity as local_conv};
        use sea_orm::QueryFilter;

        // 对部分会话相关通知，优先走“通知路由”：触发一次增量会话同步，而不是直接改本地结构，
        // 行为上更贴近 Go 版的 DoConversation*Notification → IncrSyncConversations 流程。
        if is_notification {
            use openim_protocol::constant;

            match msg.content_type {
                // 会话属性变更 / 私聊标记变更
                constant::CONVERSATION_CHANGE_NOTIFICATION
                | constant::CONVERSATION_PRIVATE_CHAT_NOTIFICATION
                // 会话清空 / 删除 / 未读数变更 / 已读回执
                | constant::CLEAR_CONVERSATION_NOTIFICATION
                | constant::CONVERSATION_UNREAD_NOTIFICATION
                | constant::CONVERSATION_DELETE_NOTIFICATION
                | constant::HAS_READ_RECEIPT => {
                    info!(
                        "[ConvSync/Notify] 收到会话通知，contentType={}，触发增量会话同步",
                        msg.content_type
                    );
                    if let Err(e) = self.incr_sync_conversations().await {
                        warn!(
                            "[ConvSync/Notify] 会话通知触发增量同步失败: {}",
                            e
                        );
                    }
                    // 交给增量同步统一刷新会话表，这里不直接修改本地会话
                    return Ok(());
                }
                _ => {
                    // 其他通知类型走通用路径（latestMsg 标签、回调等）
                }
            }
        }

        // 查询现有会话
        let model = local_conv::find()
            .filter(Column::ConversationId.eq(conversation_id))
            .one(&self.db)
            .await
            .context("查询单个会话失败")?;

        // 从现有记录或默认值构建 LocalConversation
        let mut conv = if let Some(ref model) = model {
            LocalConversation {
                conversation_id: model.conversation_id.clone(),
                conversation_type: model.conversation_type,
                user_id: model.user_id.clone(),
                group_id: model.group_id.clone(),
                show_name: model.show_name.clone(),
                face_url: model.face_url.clone(),
                latest_msg: model.latest_msg.clone(),
                latest_msg_send_time: model.latest_msg_send_time,
                unread_count: model.unread_count,
                recv_msg_opt: model.recv_msg_opt,
                is_pinned: model.is_pinned != 0,
                is_private_chat: model.is_private_chat != 0,
                burn_duration: model.burn_duration,
                group_at_type: model.group_at_type,
                is_not_in_group: model.is_not_in_group != 0,
                update_unread_count_time: model.update_unread_count_time,
                attached_info: model.attached_info.clone(),
                ex: model.ex.clone(),
                draft_text: model.draft_text.clone(),
                draft_text_time: model.draft_text_time,
                max_seq: model.max_seq,
                min_seq: model.min_seq,
                is_msg_destruct: model.is_msg_destruct != 0,
                msg_destruct_time: model.msg_destruct_time,
            }
        } else {
            // 新会话：仅用必要字段构建，其他使用默认值
            LocalConversation {
                conversation_id: conversation_id.to_string(),
                conversation_type: msg.session_type,
                user_id: msg.send_id.clone(),
                group_id: msg.group_id.clone(),
                show_name: String::new(),
                face_url: String::new(),
                latest_msg: String::new(),
                latest_msg_send_time: 0,
                unread_count: 0,
                recv_msg_opt: 0,
                is_pinned: false,
                is_private_chat: false,
                burn_duration: 0,
                group_at_type: 0,
                is_not_in_group: false,
                update_unread_count_time: 0,
                attached_info: String::new(),
                ex: String::new(),
                draft_text: String::new(),
                draft_text_time: 0,
                max_seq: msg.seq,
                min_seq: msg.seq,
                is_msg_destruct: false,
                msg_destruct_time: 0,
            }
        };

        let is_new = model.is_none();

        // 生成 latest_msg 摘要
        let latest = Self::build_latest_msg_summary(msg);

        // 更新时间与未读数
        // 参考 Go 版本：只有消息的 options 中 IsUnreadCount 为 true 时才计入未读数
        let send_time = if msg.send_time > 0 {
            msg.send_time
        } else {
            msg.create_time
        };
        conv.latest_msg = latest;
        conv.latest_msg_send_time = send_time;
        conv.max_seq = conv.max_seq.max(msg.seq);

        // 检查消息的 options 中的 unreadCount 字段
        // 参考 Go 版本：只有 options 中 unreadCount 为 true 且非自己发送的消息才计入未读数
        let should_count_unread = if msg.send_id == self.config.user_id || is_notification {
            // 自己发送的消息或通知消息不计入未读数
            false
        } else {
            // 检查 options 中的 unreadCount 字段
            // 默认情况下，如果 options 中没有明确设置，则视为 true（计入未读数）
            *msg.options.get("unreadCount").unwrap_or(&true) // 默认计入未读数
        };

        if should_count_unread {
            // 检查是否是新消息（避免重复计数）
            // 如果当前消息的 seq 大于已记录的 max_seq，说明是新消息
            let is_new_msg = msg.seq > conv.max_seq.saturating_sub(1);
            if is_new_msg {
                conv.unread_count += 1;
            }
        }

        // 落库
        self.upsert_conversation(&conv).await?;

        // 触发会话变更/新会话回调
        let json = serde_json::to_string(&vec![conv.clone()]).unwrap_or_else(|_| "[]".to_string());
        if is_new {
            self.listener.on_new_conversation(json).await;
        } else {
            self.listener.on_conversation_changed(json).await;
        }

        // 更新总未读数
        if let Ok(total_unread) = self.get_total_unread_count().await {
            self.listener
                .on_total_unread_message_count_changed(total_unread)
                .await;
        }

        Ok(())
    }

    /// 从数据库删除会话
    async fn delete_conversation(&self, conversation_id: &str) -> Result<()> {
        use sea_orm::QueryFilter;

        local_conversations::Entity::delete_many()
            .filter(local_conversations::Column::ConversationId.eq(conversation_id))
            .exec(&self.db)
            .await
            .context("删除会话失败")?;
        Ok(())
    }

    /// 获取总未读消息数（公开给上层调用）
    pub async fn get_total_unread_count(&self) -> Result<i32> {
        let conversations = local_conversations::Entity::find()
            .all(&self.db)
            .await
            .context("查询会话列表失败")?;

        let total: i32 = conversations
            .iter()
            .map(|c| c.unread_count)
            .sum();

        Ok(total)
    }

    /// 从服务器获取每个会话的 MaxSeq 和 HasReadSeq
    async fn get_has_read_and_max_seqs_from_server(&self) -> Result<HashMap<String, (i64, i64)>> {
        let operation_id = Uuid::new_v4().to_string();
        let url = format!(
            "{}/msg/get_conversations_has_read_and_max_seq",
            self.config.api_base_url
        );

        let req_json = serde_json::json!({
            "userID": self.config.user_id,
        });

        info!("[ConvSync/Seq] 📡 请求会话 Seq 信息");
        debug!("[ConvSync/Seq]   请求URL: {}", url);
        debug!("[ConvSync/Seq]   用户ID: {}", self.config.user_id);
        debug!("[ConvSync/Seq]   操作ID: {}", operation_id);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("operationID", &operation_id)
            .header("token", &self.config.token)
            .json(&req_json)
            .send()
            .await
            .context("请求失败")?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            error!(
                "[ConvSync/Seq] 会话 Seq 请求失败，HTTP状态: {}, 响应: {}",
                status, text
            );
            return Err(anyhow::anyhow!("HTTP 错误 {}: {}", status, text));
        }
        debug!("[ConvSync/Seq] 会话 Seq 请求成功，HTTP状态: {}", status);

        let text = response.text().await.context("读取响应失败")?;
        let json_value: serde_json::Value =
            serde_json::from_str(&text).context("解析 JSON 失败")?;

        // 输出原始响应数据（用于调试）
        info!("[ConvSync/Seq] 📥 服务器响应原始数据: {}", text);

        // 检查错误码
        if let Some(err_code) = json_value.get("errCode").and_then(|v| v.as_i64()) {
            if err_code != 0 {
                let err_msg = json_value
                    .get("errMsg")
                    .and_then(|v| v.as_str())
                    .unwrap_or("未知错误");
                error!(
                    "[ConvSync/Seq] 会话 Seq 服务器错误，错误码: {}, 错误信息: {}",
                    err_code, err_msg
                );
                return Err(anyhow::anyhow!("服务器错误 {}: {}", err_code, err_msg));
            }
        }

        let data = json_value
            .get("data")
            .ok_or_else(|| anyhow::anyhow!("响应中缺少 data 字段"))?;

        // 输出 data 字段内容（用于调试）
        if let Ok(data_str) = serde_json::to_string_pretty(data) {
            info!("[ConvSync/Seq] 📊 服务器返回的 data 字段: {}", data_str);
        }

        // 期望结构：data.seqs: { conversationID: { maxSeq, hasReadSeq, maxSeqTime }, ... }
        // 参考 Go 版本：resp.Seqs 是 map[string]*msg.Seqs
        let mut result = HashMap::new();

        // 先尝试作为对象（HashMap）解析
        if let Some(seqs_obj) = data.get("seqs").and_then(|v| v.as_object()) {
            info!(
                "[ConvSync/Seq] 📋 解析会话 Seq 对象，条目数: {}",
                seqs_obj.len()
            );
            for (conv_id, seq_data) in seqs_obj.iter() {
                if let Some(seq_obj) = seq_data.as_object() {
                    let max_seq = seq_obj.get("maxSeq").and_then(|v| v.as_i64()).unwrap_or(0);
                    let has_read_seq = seq_obj
                        .get("hasReadSeq")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    let unread = (max_seq - has_read_seq).max(0);
                    info!(
                        "[ConvSync/Seq]   conversationID={}, maxSeq={}, hasReadSeq={}, unreadCount={}",
                        conv_id, max_seq, has_read_seq, unread
                    );
                    result.insert(conv_id.clone(), (max_seq, has_read_seq));
                } else {
                    warn!("[ConvSync/Seq]   跳过无效条目（seq 数据不是对象）: conversationID={}, data={:?}", conv_id, seq_data);
                }
            }
        }
        // 兼容旧格式：数组格式（虽然服务器不返回，但保留兼容性）
        else if let Some(arr) = data
            .get("seqs")
            .or_else(|| data.get("conversations"))
            .and_then(|v| v.as_array())
        {
            info!(
                "[ConvSync/Seq] 📋 解析会话 Seq 数组（兼容格式），条目数: {}",
                arr.len()
            );
            for (idx, item) in arr.iter().enumerate() {
                if let Some(conv_id) = item
                    .get("conversationID")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                {
                    let max_seq = item.get("maxSeq").and_then(|v| v.as_i64()).unwrap_or(0);
                    let has_read_seq = item.get("hasReadSeq").and_then(|v| v.as_i64()).unwrap_or(0);
                    let unread = (max_seq - has_read_seq).max(0);
                    info!(
                        "[ConvSync/Seq]   [{}/{}] conversationID={}, maxSeq={}, hasReadSeq={}, unreadCount={}",
                        idx + 1, arr.len(), conv_id, max_seq, has_read_seq, unread
                    );
                    result.insert(conv_id, (max_seq, has_read_seq));
                } else {
                    warn!(
                        "[ConvSync/Seq]   [{}/{}] 跳过无效条目（缺少 conversationID）: {:?}",
                        idx + 1,
                        arr.len(),
                        item
                    );
                }
            }
        } else {
            warn!("[ConvSync/Seq] ⚠️ 响应中未找到 seqs 字段（期望对象或数组）");
            if let Ok(data_str) = serde_json::to_string_pretty(data) {
                warn!("[ConvSync/Seq]   实际 data 内容: {}", data_str);
            }
        }

        info!(
            "[ConvSync/Seq] ✅ 会话 Seq 解析完成，成功解析 {} 个会话",
            result.len()
        );
        Ok(result)
    }

    /// 基于服务器的 MaxSeq / HasReadSeq 校正本地未读数
    pub async fn sync_unread_by_seq(&self) -> Result<()> {
        info!("[ConvSync/Seq] 🔄 开始按 Seq 校正未读数...");

        // 1. 获取本地会话
        let mut local_conversations = self.get_all_conversations().await?;
        let mut local_map: HashMap<String, LocalConversation> = HashMap::new();
        for conv in local_conversations.drain(..) {
            local_map.insert(conv.conversation_id.clone(), conv);
        }

        // 2. 从服务器获取每个会话的 MaxSeq/HasReadSeq
        let seqs = self.get_has_read_and_max_seqs_from_server().await?;
        if seqs.is_empty() {
            info!("[ConvSync/Seq] 服务器未返回会话 Seq 信息，跳过未读数校正");
            return Ok(());
        }

        // 3. 计算未读并更新本地记录，同时补齐本地缺失会话
        let mut changed_conversations: Vec<LocalConversation> = Vec::new();
        let mut new_conversations: Vec<LocalConversation> = Vec::new();
        let mut missing_convs: Vec<(String, (i64, i64))> = Vec::new();

        info!(
            "[ConvSync/Seq] 🔄 开始校正未读数，服务器返回 {} 个会话的 Seq 信息",
            seqs.len()
        );
        for (conv_id, (max_seq, has_read_seq)) in seqs.into_iter() {
            let unread = (max_seq - has_read_seq).max(0) as i32;

            if let Some(mut local) = local_map.remove(&conv_id) {
                // 仅在有实际变化时更新
                if local.unread_count != unread || local.max_seq != max_seq {
                    info!(
                        "[ConvSync/Seq] 📝 校正会话未读数: conversationID={}, 本地未读数: {} -> {}, maxSeq: {} -> {}, hasReadSeq: {}",
                        conv_id, local.unread_count, unread, local.max_seq, max_seq, has_read_seq
                    );
                    local.unread_count = unread;
                    local.max_seq = max_seq;
                    // 更新时间戳由上层逻辑维护，这里不强行覆盖
                    self.upsert_conversation(&local).await?;
                    changed_conversations.push(local);
                } else {
                    debug!(
                        "[ConvSync/Seq] ✓ 会话未读数无需更新: conversationID={}, unreadCount={}, maxSeq={}",
                        conv_id, unread, max_seq
                    );
                }
            } else {
                // 本地没有该会话，记录下来后续从服务器补齐
                info!(
                    "[ConvSync/Seq] ⚠️ 按 Seq 校正未读数时发现本地不存在的会话: conversationID={}, maxSeq={}, hasReadSeq={}, unreadCount={}",
                    conv_id, max_seq, has_read_seq, unread
                );
                missing_convs.push((conv_id, (max_seq, has_read_seq)));
            }
        }

        info!(
            "[ConvSync/Seq] 📊 未读数校正统计: 已更新 {} 个会话，发现 {} 个本地缺失会话",
            changed_conversations.len(),
            missing_convs.len()
        );

        // 输出总未读数（校正前）
        if let Ok(total_before) = self.get_total_unread_count().await {
            info!("[ConvSync/Seq] 📊 校正前总未读数: {}", total_before);
        }

        // 3.1 为本地缺失的会话从服务器补齐详情并按照 Seq 初始化未读数
        if !missing_convs.is_empty() {
            info!(
                "[ConvSync/Seq] 发现本地缺失会话 {} 个，尝试从服务器补齐详情",
                missing_convs.len()
            );
            match self.get_all_conversation_list_from_server().await {
                Ok(all_resp) => {
                    let server_map: HashMap<String, LocalConversation> = all_resp
                        .conversations
                        .iter()
                        .map(Self::server_conversation_to_local)
                        .map(|c| (c.conversation_id.clone(), c))
                        .collect();

                    for (conv_id, (max_seq, has_read_seq)) in missing_convs.into_iter() {
                        if let Some(mut conv) = server_map.get(&conv_id).cloned() {
                            let unread = (max_seq - has_read_seq).max(0) as i32;
                            debug!(
                                "[ConvSync/Seq] 为缺失会话补齐记录: {} (unread={}, maxSeq={}, hasReadSeq={})",
                                conv_id, unread, max_seq, has_read_seq
                            );

                            conv.unread_count = unread;
                            conv.max_seq = max_seq;
                            // 其他字段（latestMsg 等）暂由后续 on_new_message 或上层逻辑完善

                            self.upsert_conversation(&conv).await?;
                            new_conversations.push(conv);
                        } else {
                            warn!(
                                "[ConvSync/Seq] 按 Seq 校正时服务器会话列表中也不存在会话: {} (maxSeq={}, hasReadSeq={})",
                                conv_id, max_seq, has_read_seq
                            );
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "[ConvSync/Seq] 拉取服务器会话详情失败，无法为缺失会话补齐记录: {}",
                        e
                    );
                }
            }
        }

        // 4. 触发回调（参考 Go 版本：只要有会话变更就触发总未读数回调）
        if !new_conversations.is_empty() {
            let json =
                serde_json::to_string(&new_conversations).unwrap_or_else(|_| "[]".to_string());
            info!(
                "[ConvSync/Seq] 📢 触发新会话回调，数量: {}",
                new_conversations.len()
            );
            self.listener.on_new_conversation(json).await;
        }

        if !changed_conversations.is_empty() {
            let json =
                serde_json::to_string(&changed_conversations).unwrap_or_else(|_| "[]".to_string());
            info!(
                "[ConvSync/Seq] 📢 触发会话变更回调，数量: {}",
                changed_conversations.len()
            );
            self.listener.on_conversation_changed(json).await;
        }

        // 只要有会话变更（新会话或变更会话），就触发总未读数回调（参考 Go 版本）
        if !new_conversations.is_empty() || !changed_conversations.is_empty() {
            match self.get_total_unread_count().await {
                Ok(total_unread) => {
                    info!("[ConvSync/Seq] 📢 触发总未读数变更回调: {}", total_unread);
                    self.listener
                        .on_total_unread_message_count_changed(total_unread)
                        .await;
                }
                Err(e) => {
                    warn!("[ConvSync/Seq] ⚠️ 获取总未读数失败，无法触发回调: {}", e);
                }
            }
        } else {
            info!("[ConvSync/Seq] ℹ️ 无会话变更，跳过回调");
        }

        info!("[ConvSync/Seq] ✅ 按 Seq 校正未读数完成");
        Ok(())
    }

    /// 从服务器获取增量会话
    async fn get_incremental_conversation_from_server(
        &self,
        version: u64,
        version_id: &str,
    ) -> Result<conversation::GetIncrementalConversationResp> {
        let operation_id = Uuid::new_v4().to_string();
        let url = format!(
            "{}/conversation/get_incremental_conversations",
            self.config.api_base_url
        );

        // 手动构建 JSON 请求体
        let req_json = serde_json::json!({
            "userID": self.config.user_id,
            "version": version,
            "versionID": version_id
        });

        info!(
            "[ConvSync/HTTP] 📡 请求增量会话同步\n   请求URL: {}\n   版本: {}, 版本ID: {}\n   用户ID: {}\n   操作ID: {}",
            url,
            version, version_id,
            self.config.user_id,
            operation_id
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("operationID", &operation_id)
            .header("token", &self.config.token)
            .json(&req_json)
            .send()
            .await
            .context("请求失败")?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            error!(
                "[ConvSync/HTTP] 增量会话同步请求失败，HTTP状态: {}, 响应: {}",
                status, text
            );
            return Err(anyhow::anyhow!("HTTP 错误 {}: {}", status, text));
        }
        debug!("[ConvSync/HTTP] 增量会话同步请求成功，HTTP状态: {}", status);

        // 解析 JSON 响应
        let text = response.text().await.context("读取响应失败")?;
        let json_value: serde_json::Value =
            serde_json::from_str(&text).context("解析 JSON 失败")?;

        // 检查错误码
        if let Some(err_code) = json_value.get("errCode").and_then(|v| v.as_i64()) {
            if err_code != 0 {
                let err_msg = json_value
                    .get("errMsg")
                    .and_then(|v| v.as_str())
                    .unwrap_or("未知错误");
                error!(
                    "[ConvSync/HTTP] 增量会话同步服务器错误，错误码: {}, 错误信息: {}",
                    err_code, err_msg
                );
                return Err(anyhow::anyhow!("服务器错误 {}: {}", err_code, err_msg));
            }
        }

        // 从 data 字段解析响应
        let data = json_value
            .get("data")
            .ok_or_else(|| anyhow::anyhow!("响应中缺少 data 字段"))?;

        // 手动构建 protobuf 响应，直接从 JSON 解析
        let version_id_str = data
            .get("versionID")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let version_value = data.get("version").and_then(|v| v.as_u64()).unwrap_or(0);

        // 解析 insert 数组
        let inserts: Vec<conversation::Conversation> = data
            .get("insert")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(Self::json_to_server_conversation)
                    .collect()
            })
            .unwrap_or_default();

        // 解析 update 数组
        let updates: Vec<conversation::Conversation> = data
            .get("update")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(Self::json_to_server_conversation)
                    .collect()
            })
            .unwrap_or_default();

        let resp = conversation::GetIncrementalConversationResp {
            full: data.get("full").and_then(|v| v.as_bool()).unwrap_or(false),
            version_id: version_id_str.clone(),
            version: version_value,
            insert: inserts,
            update: updates,
            delete: data
                .get("delete")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
        };

        info!("[ConvSync/HTTP] ✅ 增量会话同步响应");
        info!("[ConvSync/HTTP]   全量同步: {}", resp.full);
        info!("[ConvSync/HTTP]   版本ID: {}", resp.version_id);
        info!("[ConvSync/HTTP]   版本: {}", resp.version);
        info!(
            "[ConvSync/HTTP]   新增: {} 个, 更新: {} 个, 删除: {} 个",
            resp.insert.len(),
            resp.update.len(),
            resp.delete.len()
        );
        debug!("[ConvSync/HTTP]   删除的会话ID: {:?}", resp.delete);

        Ok(resp)
    }

    /// 从服务器获取所有会话
    async fn get_all_conversation_list_from_server(
        &self,
    ) -> Result<conversation::GetAllConversationsResp> {
        let operation_id = Uuid::new_v4().to_string();
        let url = format!(
            "{}/conversation/get_all_conversations",
            self.config.api_base_url
        );

        let req_json = serde_json::json!({
            "ownerUserID": self.config.user_id
        });

        info!("[ConvSync/HTTP] 📡 请求全量会话同步");
        debug!("[ConvSync/HTTP]   请求URL: {}", url);
        debug!("[ConvSync/HTTP]   用户ID: {}", self.config.user_id);
        debug!("[ConvSync/HTTP]   操作ID: {}", operation_id);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("operationID", &operation_id)
            .header("token", &self.config.token)
            .json(&req_json)
            .send()
            .await
            .context("请求失败")?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            error!(
                "[ConvSync/HTTP] 全量会话同步请求失败，HTTP状态: {}, 响应: {}",
                status, text
            );
            return Err(anyhow::anyhow!("HTTP 错误 {}: {}", status, text));
        }
        debug!("[ConvSync/HTTP] 全量会话同步请求成功，HTTP状态: {}", status);

        let text = response.text().await.context("读取响应失败")?;
        let json_value: serde_json::Value =
            serde_json::from_str(&text).context("解析 JSON 失败")?;

        // 检查错误码
        if let Some(err_code) = json_value.get("errCode").and_then(|v| v.as_i64()) {
            if err_code != 0 {
                let err_msg = json_value
                    .get("errMsg")
                    .and_then(|v| v.as_str())
                    .unwrap_or("未知错误");
                error!(
                    "[ConvSync/HTTP] 全量会话同步服务器错误，错误码: {}, 错误信息: {}",
                    err_code, err_msg
                );
                return Err(anyhow::anyhow!("服务器错误 {}: {}", err_code, err_msg));
            }
        }

        let data = json_value
            .get("data")
            .ok_or_else(|| anyhow::anyhow!("响应中缺少 data 字段"))?;

        let conversations = data
            .get("conversations")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(Self::json_to_server_conversation)
                    .collect()
            })
            .unwrap_or_default();

        let resp = conversation::GetAllConversationsResp { conversations };

        info!("[ConvSync/HTTP] ✅ 全量会话同步响应");
        info!("[ConvSync/HTTP]   会话数: {}", resp.conversations.len());
        debug!(
            "[ConvSync/HTTP]   会话详情: {:?}",
            resp.conversations
                .iter()
                .map(|c| &c.conversation_id)
                .collect::<Vec<_>>()
        );

        Ok(resp)
    }

    /// 从服务器获取所有会话 ID
    async fn get_all_conversation_ids_from_server(&self) -> Result<Vec<String>> {
        let operation_id = Uuid::new_v4().to_string();
        let url = format!(
            "{}/conversation/get_full_conversation_ids",
            self.config.api_base_url
        );

        let req_json = serde_json::json!({
            "userID": self.config.user_id
        });

        info!("[ConvSync/HTTP] 📡 请求会话 ID 列表");
        debug!("[ConvSync/HTTP]   请求URL: {}", url);
        debug!("[ConvSync/HTTP]   操作ID: {}", operation_id);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("operationID", &operation_id)
            .header("token", &self.config.token)
            .json(&req_json)
            .send()
            .await
            .context("请求失败")?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            error!(
                "[ConvSync/HTTP] 会话ID列表请求失败，HTTP状态: {}, 响应: {}",
                status, text
            );
            return Err(anyhow::anyhow!("HTTP 错误 {}: {}", status, text));
        }
        debug!("[ConvSync/HTTP] 会话ID列表请求成功，HTTP状态: {}", status);

        let text = response.text().await.context("读取响应失败")?;
        let json_value: serde_json::Value =
            serde_json::from_str(&text).context("解析 JSON 失败")?;

        // 检查错误码
        if let Some(err_code) = json_value.get("errCode").and_then(|v| v.as_i64()) {
            if err_code != 0 {
                let err_msg = json_value
                    .get("errMsg")
                    .and_then(|v| v.as_str())
                    .unwrap_or("未知错误");
                error!(
                    "[ConvSync/HTTP] 会话ID列表服务器错误，错误码: {}, 错误信息: {}",
                    err_code, err_msg
                );
                return Err(anyhow::anyhow!("服务器错误 {}: {}", err_code, err_msg));
            }
        }

        let data = json_value
            .get("data")
            .ok_or_else(|| anyhow::anyhow!("响应中缺少 data 字段"))?;

        let conversation_ids: Vec<String> = data
            .get("conversationIDs")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        info!("[ConvSync/HTTP] ✅ 会话 ID 列表响应");
        info!("[ConvSync/HTTP]   会话ID数: {}", conversation_ids.len());
        debug!("[ConvSync/HTTP]   会话ID列表: {:?}", conversation_ids);

        Ok(conversation_ids)
    }

    /// 将服务器会话转换为本地会话
    fn server_conversation_to_local(server_conv: &conversation::Conversation) -> LocalConversation {
        LocalConversation {
            conversation_id: server_conv.conversation_id.clone(),
            conversation_type: server_conv.conversation_type,
            user_id: server_conv.user_id.clone(),
            group_id: server_conv.group_id.clone(),
            show_name: String::new(),  // 需要从用户/群组信息获取
            face_url: String::new(),   // 需要从用户/群组信息获取
            latest_msg: String::new(), // 需要从消息获取
            latest_msg_send_time: 0,   // 需要从消息获取
            unread_count: 0,           // 字段不存在，使用默认值
            recv_msg_opt: server_conv.recv_msg_opt,
            is_pinned: server_conv.is_pinned,
            is_private_chat: server_conv.is_private_chat,
            burn_duration: server_conv.burn_duration,
            group_at_type: server_conv.group_at_type,
            is_not_in_group: false,      // 字段不存在，使用默认值
            update_unread_count_time: 0, // 字段不存在，使用默认值
            attached_info: server_conv.attached_info.clone(),
            ex: server_conv.ex.clone(),
            draft_text: String::new(),
            draft_text_time: 0, // 字段不存在，使用默认值
            max_seq: server_conv.max_seq,
            min_seq: server_conv.min_seq,
            is_msg_destruct: server_conv.is_msg_destruct,
            msg_destruct_time: server_conv.msg_destruct_time,
        }
    }

    /// 将 JSON 对象转换为服务器会话结构（conversation::Conversation）
    fn json_to_server_conversation(v: &serde_json::Value) -> Option<conversation::Conversation> {
        Some(conversation::Conversation {
            owner_user_id: v
                .get("ownerUserID")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            conversation_id: v
                .get("conversationID")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            conversation_type: v
                .get("conversationType")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            user_id: v
                .get("userID")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            group_id: v
                .get("groupID")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            recv_msg_opt: v
                .get("recvMsgOpt")
                .and_then(|v| v.as_i64())
                .map(|v| v as i32)
                .unwrap_or(0),
            is_pinned: v.get("isPinned").and_then(|v| v.as_bool()).unwrap_or(false),
            is_private_chat: v
                .get("isPrivateChat")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            burn_duration: v
                .get("burnDuration")
                .and_then(|v| v.as_i64())
                .map(|v| v as i32)
                .unwrap_or(0),
            group_at_type: v
                .get("groupAtType")
                .and_then(|v| v.as_i64())
                .map(|v| v as i32)
                .unwrap_or(0),
            attached_info: v
                .get("attachedInfo")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            ex: v
                .get("ex")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            max_seq: v.get("maxSeq").and_then(|v| v.as_i64()).unwrap_or(0),
            min_seq: v.get("minSeq").and_then(|v| v.as_i64()).unwrap_or(0),
            is_msg_destruct: v
                .get("isMsgDestruct")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            msg_destruct_time: v
                .get("msgDestructTime")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            latest_msg_destruct_time: v
                .get("latestMsgDestructTime")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
        })
    }

    /// 同步会话（对比服务器和本地数据）
    ///
    /// - `server_conversations`: 服务器返回的会话列表
    /// - `local_conversations`: 本地已有的会话列表
    /// - `seqs_map`: 可选的 seqs 信息（conversationID -> (maxSeq, hasReadSeq)），用于设置未读数
    async fn sync_conversations(
        &self,
        server_conversations: Vec<LocalConversation>,
        local_conversations: Vec<LocalConversation>,
        seqs_map: Option<&HashMap<String, (i64, i64)>>,
    ) -> Result<()> {
        info!(
            "[ConvSync] 开始同步会话，服务器会话数: {}, 本地会话数: {}",
            server_conversations.len(),
            local_conversations.len()
        );

        let local_map: HashMap<String, LocalConversation> = local_conversations
            .into_iter()
            .map(|c| (c.conversation_id.clone(), c))
            .collect();

        let mut server_map: HashMap<String, LocalConversation> = server_conversations
            .into_iter()
            .map(|c| (c.conversation_id.clone(), c))
            .collect();

        let mut new_conversations = Vec::new();
        let mut changed_conversations = Vec::new();
        let mut insert_count = 0;
        let mut update_count = 0;
        let mut delete_count = 0;

        // 处理插入和更新
        // 先根据 seqs 信息更新未读数（参考 Go 版本）
        if let Some(seqs) = seqs_map {
            for (conv_id, &(max_seq, has_read_seq)) in seqs.iter() {
                if let Some(server_conv) = server_map.get_mut(conv_id) {
                    let unread = (max_seq - has_read_seq).max(0) as i32;
                    info!(
                        "[ConvSync]   会话 {} 根据 seqs 设置未读数: maxSeq={}, hasReadSeq={}, unreadCount={}",
                        conv_id, max_seq, has_read_seq, unread
                    );
                    server_conv.unread_count = unread;
                    server_conv.max_seq = max_seq;
                }
            }
        }

        // 然后处理插入和更新
        for (id, server_conv) in server_map.iter() {
            if let Some(local_conv) = local_map.get(id) {
                // 更新：比较并更新变化的字段
                // 注意：即使字段相同，如果未读数有变化也需要更新
                if !self.conversations_equal(local_conv, server_conv)
                    || local_conv.unread_count != server_conv.unread_count
                    || local_conv.max_seq != server_conv.max_seq
                {
                    info!(
                        "[ConvSync]   更新会话: {} (类型: {}), 未读数: {} -> {}",
                        id,
                        server_conv.conversation_type,
                        local_conv.unread_count,
                        server_conv.unread_count
                    );
                    debug!(
                        "[ConvSync]   会话详情 - 置顶: {}, 私聊: {}, maxSeq: {} -> {}",
                        server_conv.is_pinned,
                        server_conv.is_private_chat,
                        local_conv.max_seq,
                        server_conv.max_seq
                    );
                    self.upsert_conversation(server_conv).await?;
                    changed_conversations.push(server_conv.clone());
                    update_count += 1;
                } else {
                    debug!("[ConvSync]   会话 {} 无需更新", id);
                }
            } else {
                // 插入：新会话
                info!(
                    "[ConvSync]   新增会话: {} (类型: {}), 未读数: {}",
                    id, server_conv.conversation_type, server_conv.unread_count
                );
                debug!(
                    "[ConvSync]   会话详情 - 置顶: {}, 私聊: {}, maxSeq: {}",
                    server_conv.is_pinned, server_conv.is_private_chat, server_conv.max_seq
                );
                self.upsert_conversation(server_conv).await?;
                new_conversations.push(server_conv.clone());
                insert_count += 1;
            }
        }

        // 处理删除：服务器没有但本地有的会话
        let local_ids: std::collections::HashSet<String> = local_map.keys().cloned().collect();
        let server_ids: std::collections::HashSet<String> = server_map.keys().cloned().collect();
        for id in local_ids.difference(&server_ids) {
            warn!("[ConvSync]   删除会话: {}", id);
            self.delete_conversation(id).await?;
            delete_count += 1;
        }

        // 触发回调
        if !new_conversations.is_empty() {
            let json =
                serde_json::to_string(&new_conversations).unwrap_or_else(|_| "[]".to_string());
            self.listener.on_new_conversation(json).await;
        }

        if !changed_conversations.is_empty() {
            let json =
                serde_json::to_string(&changed_conversations).unwrap_or_else(|_| "[]".to_string());
            self.listener.on_conversation_changed(json).await;
        }

        // 更新总未读数回调
        if insert_count > 0 || update_count > 0 || delete_count > 0 {
            if let Ok(total_unread) = self.get_total_unread_count().await {
                self.listener
                    .on_total_unread_message_count_changed(total_unread)
                    .await;
            }
        }

        info!(
            "[ConvSync] 会话同步完成 - 新增: {}, 更新: {}, 删除: {}",
            insert_count, update_count, delete_count
        );
        Ok(())
    }

    /// 比较两个会话是否相等（用于判断是否需要更新）
    fn conversations_equal(&self, local: &LocalConversation, server: &LocalConversation) -> bool {
        local.recv_msg_opt == server.recv_msg_opt
            && local.is_pinned == server.is_pinned
            && local.is_private_chat == server.is_private_chat
            && local.burn_duration == server.burn_duration
            && local.is_not_in_group == server.is_not_in_group
            && local.group_at_type == server.group_at_type
            && local.update_unread_count_time == server.update_unread_count_time
            && local.attached_info == server.attached_info
            && local.ex == server.ex
            && local.max_seq == server.max_seq
            && local.min_seq == server.min_seq
            && local.msg_destruct_time == server.msg_destruct_time
            && local.is_msg_destruct == server.is_msg_destruct
    }

    /// 增量同步会话（核心函数，对应 Go 版本的 IncrSyncConversations）
    pub async fn incr_sync_conversations(&self) -> Result<()> {
        info!("[ConvSync] 🔄 开始增量同步会话...");

        // 1. 获取本地版本信息
        let version_sync = self.get_version_sync().await?;

        if let Some(ref vs) = version_sync {
            debug!(
                "[ConvSync] 本地版本信息 - 版本: {}, 版本ID: {}",
                vs.version, vs.version_id
            );
        } else {
            debug!("[ConvSync] 本地无版本信息");
        }

        // 2. 获取本地所有会话
        let local_conversations = self.get_all_conversations().await?;
        let local_ids = self.get_all_conversation_ids().await?;
        info!("[ConvSync] 本地会话数: {}", local_ids.len());

        // 3. 判断是否需要全量同步
        let reinstalled = local_ids.is_empty();
        if reinstalled {
            warn!("本地无会话，执行全量同步...");
            self.listener.on_sync_server_start(true).await;
            return self.full_sync().await;
        }

        // 4. 获取版本信息
        let (version, version_id) = if let Some(vs) = version_sync {
            (vs.version, vs.version_id)
        } else {
            // 如果没有版本信息，先获取全量会话 ID 列表
            let server_ids_vec = self.get_all_conversation_ids_from_server().await?;
            let server_ids: std::collections::HashSet<String> =
                server_ids_vec.iter().cloned().collect();
            let local_ids_set: std::collections::HashSet<String> =
                local_ids.iter().cloned().collect();

            // 如果服务器和本地的 ID 列表不一致，执行全量同步
            if server_ids != local_ids_set {
                warn!("[ConvSync] 会话 ID 列表不一致，执行全量同步...");
                debug!(
                    "[ConvSync] 服务器会话ID数: {}, 本地会话ID数: {}",
                    server_ids.len(),
                    local_ids_set.len()
                );
                let diff: Vec<_> = server_ids.difference(&local_ids_set).collect();
                if !diff.is_empty() {
                    debug!("[ConvSync]   服务器多出的会话ID: {:?}", diff);
                }
                let diff: Vec<_> = local_ids_set.difference(&server_ids).collect();
                if !diff.is_empty() {
                    debug!("[ConvSync]   本地多出的会话ID: {:?}", diff);
                }
                return self.full_sync().await;
            }

            // 否则从全量同步获取版本信息
            let all_resp = self.get_all_conversation_list_from_server().await?;
            let server_convs: Vec<LocalConversation> = all_resp
                .conversations
                .iter()
                .map(Self::server_conversation_to_local)
                .collect();

            // 先获取 seqs 信息用于设置未读数
            let seqs_map = match self.get_has_read_and_max_seqs_from_server().await {
                Ok(seqs) => {
                    info!(
                        "[ConvSync] 获取到 {} 个会话的 seqs 信息，用于设置未读数",
                        seqs.len()
                    );
                    Some(seqs)
                }
                Err(e) => {
                    warn!("[ConvSync] 获取 seqs 信息失败，将使用默认未读数: {}", e);
                    None
                }
            };

            // 同步数据（传入 seqs_map 用于设置未读数）
            self.sync_conversations(
                server_convs.clone(),
                local_conversations.clone(),
                seqs_map.as_ref(),
            )
            .await?;

            // 更新版本信息（这里简化处理，实际应该从响应中获取）
            let new_version = LocalVersionSync {
                table_name: "local_conversations".to_string(),
                entity_id: self.config.user_id.clone(),
                version: 1,
                version_id: Uuid::new_v4().to_string(),
            };
            self.save_version_sync(&new_version).await?;
            info!(
                "[ConvSync] 已更新版本信息 - 版本: {}, 版本ID: {}",
                new_version.version, new_version.version_id
            );

            return Ok(());
        };

        info!(
            "[ConvSync] 使用增量同步，版本: {}, 版本ID: {}",
            version, version_id
        );

        // 触发同步开始回调（非重新安装）
        self.listener.on_sync_server_start(false).await;
        self.listener.on_sync_server_progress(10).await;

        // 5. 调用增量同步接口
        let resp = match self
            .get_incremental_conversation_from_server(version, &version_id)
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                error!("[ConvSync] 增量同步失败: {}", e);
                self.listener.on_sync_server_failed(false).await;
                return Err(e);
            }
        };

        self.listener.on_sync_server_progress(50).await;

        // 6. 检查是否全量同步
        if resp.full {
            warn!("[ConvSync]   服务器要求全量同步...");
            return self.full_sync().await;
        }

        // 7. 处理增量数据
        let mut server_conversations = Vec::new();

        // 处理插入
        info!("[ConvSync] 处理新增会话，数量: {}", resp.insert.len());
        for server_conv in resp.insert.iter() {
            debug!("[ConvSync]   新增会话ID: {}", server_conv.conversation_id);
            server_conversations.push(Self::server_conversation_to_local(server_conv));
        }

        // 处理更新
        info!("[ConvSync] 处理更新会话，数量: {}", resp.update.len());
        for server_conv in resp.update.iter() {
            debug!("[ConvSync]   更新会话ID: {}", server_conv.conversation_id);
            server_conversations.push(Self::server_conversation_to_local(server_conv));
        }

        // 8. 先获取 seqs 信息用于设置未读数（参考 Go 版本的 SyncAllConversationHashReadSeqs）
        let seqs_map = match self.get_has_read_and_max_seqs_from_server().await {
            Ok(seqs) => {
                info!(
                    "[ConvSync] 获取到 {} 个会话的 seqs 信息，用于设置未读数",
                    seqs.len()
                );
                Some(seqs)
            }
            Err(e) => {
                warn!("[ConvSync] 获取 seqs 信息失败，将使用默认未读数: {}", e);
                None
            }
        };

        // 同步数据（传入 seqs_map 用于设置未读数）
        self.sync_conversations(server_conversations, local_conversations, seqs_map.as_ref())
            .await?;

        self.listener.on_sync_server_progress(80).await;

        // 9. 处理删除
        if !resp.delete.is_empty() {
            info!("[ConvSync] 处理删除会话，数量: {}", resp.delete.len());
            for id in resp.delete.iter() {
                warn!("[ConvSync]   删除会话: {}", id);
                self.delete_conversation(id).await?;
            }
        }

        // 10. 更新版本信息
        if !resp.version_id.is_empty() {
            let new_version = if resp.version > 0 {
                resp.version
            } else {
                version + 1
            };
            let new_version_sync = LocalVersionSync {
                table_name: "local_conversations".to_string(),
                entity_id: self.config.user_id.clone(),
                version: new_version,
                version_id: resp.version_id.clone(),
            };
            self.save_version_sync(&new_version_sync).await?;
            info!(
                "[ConvSync] 已更新版本信息 - 版本: {} -> {}, 版本ID: {}",
                version, new_version_sync.version, new_version_sync.version_id
            );
        }

        self.listener.on_sync_server_progress(100).await;
        self.listener.on_sync_server_finish(false).await;

        // 11. 增量同步后按 Seq 校正未读数（错误不影响整体结果）
        if let Err(e) = self.sync_unread_by_seq().await {
            warn!("[ConvSync/Seq] 增量同步后按 Seq 校正未读数失败: {}", e);
        }

        info!("[ConvSync] ✅ 增量同步完成\n");
        Ok(())
    }

    /// 全量同步会话
    async fn full_sync(&self) -> Result<()> {
        info!("[ConvSync] 🔄 开始全量同步会话...");

        let reinstalled = self.get_all_conversation_ids().await?.is_empty();
        debug!(
            "[ConvSync] full_sync -> on_sync_server_start(reinstalled={})",
            reinstalled
        );
        self.listener.on_sync_server_start(reinstalled).await;
        debug!("[ConvSync] full_sync -> on_sync_server_progress(10)");
        self.listener.on_sync_server_progress(10).await;

        // 1. 获取服务器所有会话
        let resp = match self.get_all_conversation_list_from_server().await {
            Ok(resp) => resp,
            Err(e) => {
                error!("[ConvSync] 全量同步失败: {}", e);
                debug!(
                    "[ConvSync] full_sync -> on_sync_server_failed(reinstalled={})",
                    reinstalled
                );
                self.listener.on_sync_server_failed(reinstalled).await;
                return Err(e);
            }
        };
        info!(
            "[ConvSync] 从服务器获取到 {} 个会话",
            resp.conversations.len()
        );
        self.listener.on_sync_server_progress(30).await;

        // 2. 转换为本地格式
        let server_conversations: Vec<LocalConversation> = resp
            .conversations
            .iter()
            .map(Self::server_conversation_to_local)
            .collect();
        debug!(
            "[ConvSync] 已转换 {} 个会话为本地格式",
            server_conversations.len()
        );
        debug!("[ConvSync] full_sync -> on_sync_server_progress(50)");
        self.listener.on_sync_server_progress(50).await;

        // 3. 获取本地会话
        let local_conversations = self.get_all_conversations().await?;
        info!("[ConvSync] 本地已有 {} 个会话", local_conversations.len());

        // 4. 先获取 seqs 信息用于设置未读数（参考 Go 版本的 SyncAllConversationHashReadSeqs）
        let seqs_map = match self.get_has_read_and_max_seqs_from_server().await {
            Ok(seqs) => {
                info!(
                    "[ConvSync] 获取到 {} 个会话的 seqs 信息，用于设置未读数",
                    seqs.len()
                );
                Some(seqs)
            }
            Err(e) => {
                warn!("[ConvSync] 获取 seqs 信息失败，将使用默认未读数: {}", e);
                None
            }
        };

        // 同步数据（传入 seqs_map 用于设置未读数）
        self.sync_conversations(server_conversations, local_conversations, seqs_map.as_ref())
            .await?;
        debug!("[ConvSync] full_sync -> on_sync_server_progress(80)");
        self.listener.on_sync_server_progress(80).await;

        // 5. 更新版本信息（简化处理）
        let new_version = LocalVersionSync {
            table_name: "local_conversations".to_string(),
            entity_id: self.config.user_id.clone(),
            version: 1,
            version_id: Uuid::new_v4().to_string(),
        };
        self.save_version_sync(&new_version).await?;
        info!(
            "[ConvSync] 已更新版本信息 - 版本: {}, 版本ID: {}",
            new_version.version, new_version.version_id
        );

        debug!("[ConvSync] full_sync -> on_sync_server_progress(100)");
        self.listener.on_sync_server_progress(100).await;
        debug!(
            "[ConvSync] full_sync -> on_sync_server_finish(reinstalled={})",
            reinstalled
        );
        self.listener.on_sync_server_finish(reinstalled).await;

        // 6. 全量同步后按 Seq 校正未读数（错误不影响整体结果）
        if let Err(e) = self.sync_unread_by_seq().await {
            warn!("[ConvSync/Seq] 全量同步后按 Seq 校正未读数失败: {}", e);
        }

        info!("[ConvSync] ✅ 全量同步完成\n");
        Ok(())
    }

    /// 获取会话列表（分页）
    pub async fn get_conversation_list_split(
        &self,
        offset: usize,
        count: usize,
    ) -> Result<Vec<LocalConversation>> {
        debug!("[ConvSync] 获取会话列表，偏移: {}, 数量: {}", offset, count);

        // 从数据库查询所有会话
        let mut list = self.get_all_conversations().await?;

        // 过滤掉无消息时间的会话
        list.retain(|c| c.latest_msg_send_time > 0);
        debug!(
            "[ConvSync] 过滤后会话数: {} (过滤掉无消息时间的会话)",
            list.len()
        );

        // 排序：置顶优先，然后按时间降序
        list.sort_by(|a, b| {
            // 置顶优先
            match (a.is_pinned, b.is_pinned) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => {
                    // 按时间降序
                    let time_a = a.latest_msg_send_time.max(a.draft_text_time);
                    let time_b = b.latest_msg_send_time.max(b.draft_text_time);
                    time_b.cmp(&time_a)
                }
            }
        });

        // 分页
        let start = offset.min(list.len());
        let end = (offset + count).min(list.len());
        let result = list[start..end].to_vec();
        debug!(
            "[ConvSync] 返回 {} 个会话 (范围: {} - {})",
            result.len(),
            start,
            end
        );
        Ok(result)
    }

    /// 获取所有会话列表
    pub async fn get_all_conversation_list(&self) -> Result<Vec<LocalConversation>> {
        debug!("[ConvSync] 获取所有会话列表");
        self.get_conversation_list_split(0, usize::MAX).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::im::login_async;
    use std::sync::Once;
    static INIT_LOGGER: Once = Once::new();

    fn init_test_logger() {
        INIT_LOGGER.call_once(|| {
            use tracing_subscriber::prelude::*;
            use tracing_subscriber::EnvFilter;

            // 关闭 hyper_util::client 等第三方库的 debug，只保留：
            // - 当前 crate（rust_lib_flutter_rust_demo）的 debug
            // - sqlx 的 debug（打印 SQL）
            let filter_layer = EnvFilter::new(
                "info,rust_lib_flutter_rust_demo=debug,sqlx=debug,hyper_util::client=info,reqwest=info",
            );

            let fmt_layer = tracing_subscriber::fmt::layer()
                .with_file(true)        // 包含文件名
                .with_line_number(true) // 包含行号
                .with_target(false)     // 不显示 target（可选，减少噪音）
                .with_test_writer();

            tracing_subscriber::registry()
                .with(filter_layer)
                .with(fmt_layer)
                .init();
        });
    }

    #[tokio::test]
    #[ignore]
    async fn test_conversation_sync() -> Result<()> {
        // 确保单测默认输出 debug 日志
        init_test_logger();
        let login_response = match login_async(
            "+86".to_string(),
            "17764008284".to_string(),
            "284f3d09ea0695538e4ded1c1766d73a".to_string(),
            5,
        )
        .await
        {
            Ok(info) => {
                info!("✅ 登录成功！");
                info
            }
            Err(e) => {
                error!("登录失败: {}", e);
                return Err(anyhow::anyhow!("登录失败: {}", e));
            }
        };
        let login_data = match login_response.data {
            Some(data) => data,
            None => {
                return Err(anyhow::anyhow!("登录响应中没有数据"));
            }
        };
        let config = ConversationSyncerConfig {
            user_id: login_data.user_id.clone(),
            api_base_url: "http://localhost:10002".to_string(),
            token: login_data.im_token.clone(),
            // 使用sqlite本地文件存储
            db_path: "sqlite://test_conversation.db?mode=rwc".to_string(),
        };

        let syncer =
            ConversationSyncer::with_listener(config, Arc::new(TestConversationListener)).await?;
        syncer.incr_sync_conversations().await?;

        // tokio::time::sleep(std::time::Duration::from_secs(100)).await;
        Ok(())
    }

    struct TestConversationListener;

    #[async_trait]
    impl ConversationListener for TestConversationListener {
        async fn on_sync_server_start(&self, reinstalled: bool) {
            info!("开始同步: reinstalled={}", reinstalled);
        }
        async fn on_sync_server_finish(&self, reinstalled: bool) {
            info!("同步完成: reinstalled={}", reinstalled);
        }
        async fn on_sync_server_progress(&self, progress: i32) {
            info!("同步进度: progress={}", progress);
        }
        async fn on_sync_server_failed(&self, reinstalled: bool) {
            info!("同步失败: reinstalled={}", reinstalled);
        }
        async fn on_new_conversation(&self, conversation_list: String) {
            info!("新会话: conversation_list={}", conversation_list);
        }
        async fn on_conversation_changed(&self, conversation_list: String) {
            info!("会话变更: conversation_list={}", conversation_list);
        }
        async fn on_total_unread_message_count_changed(&self, total_unread_count: i32) {
            info!(
                "总未读消息数变更: total_unread_count={}",
                total_unread_count
            );
        }
        async fn on_conversation_user_input_status_changed(&self, change: String) {
            info!("会话用户输入状态变更: change={}", change);
        }
    }
}
