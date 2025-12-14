//! 会话数据访问层（DAO）
//!
//! 负责所有会话相关的数据库操作，将数据访问逻辑与业务逻辑分离

use crate::im::conversation::entities::local_conversations;
use crate::im::types::LocalConversation;
use anyhow::{Context, Result};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use tracing::{debug, info};

/// 会话 DAO
pub struct ConversationDao {
    db: DatabaseConnection,
}

impl ConversationDao {
    /// 创建新的会话 DAO
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// 初始化数据库表结构
    pub async fn init_db(&self) -> Result<()> {
        info!("[ConvDAO/DB] 初始化数据库表结构");

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

        info!("[ConvDAO/DB] 数据库表初始化完成");
        Ok(())
    }

    /// 使用共享数据库连接初始化数据库表结构（静态方法）
    pub async fn init_db_with_connection(db: &DatabaseConnection) -> Result<()> {
        info!("[ConvDAO/DB] 初始化会话数据库表结构");

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
        db.execute_unprepared(sql1)
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
        db.execute_unprepared(sql2)
            .await
            .context("创建版本同步表失败")?;

        info!("[ConvDAO/DB] 数据库表初始化完成");
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
            "[ConvDAO] 获取本地会话列表，共 {} 个会话",
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

        debug!("[ConvDAO] 获取本地会话ID列表，共 {} 个", ids.len());
        Ok(ids)
    }

    /// 根据会话ID查询单个会话
    pub async fn get_conversation_by_id(
        &self,
        conversation_id: &str,
    ) -> Result<Option<LocalConversation>> {
        use crate::im::conversation::entities::local_conversations::{Column, Entity};
        use sea_orm::QueryFilter;

        let model = Entity::find()
            .filter(Column::ConversationId.eq(conversation_id))
            .one(&self.db)
            .await
            .context("查询单个会话失败")?;

        Ok(model.map(|model| LocalConversation {
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
        }))
    }

    /// 插入或更新会话到数据库
    pub async fn upsert_conversation(&self, conv: &LocalConversation) -> Result<()> {
        use crate::im::conversation::entities::local_conversations::ActiveModel;

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

    /// 从数据库删除会话
    pub async fn delete_conversation(&self, conversation_id: &str) -> Result<()> {
        use sea_orm::QueryFilter;

        local_conversations::Entity::delete_many()
            .filter(local_conversations::Column::ConversationId.eq(conversation_id))
            .exec(&self.db)
            .await
            .context("删除会话失败")?;
        Ok(())
    }

    /// 获取总未读消息数
    pub async fn get_total_unread_count(&self) -> Result<i32> {
        let conversations = local_conversations::Entity::find()
            .all(&self.db)
            .await
            .context("查询会话列表失败")?;

        let total: i32 = conversations.iter().map(|c| c.unread_count).sum();

        Ok(total)
    }
}

/// 版本同步 DAO
pub struct VersionSyncDao {
    db: DatabaseConnection,
    user_id: String,
}

impl VersionSyncDao {
    /// 创建新的版本同步 DAO
    pub fn new(db: DatabaseConnection, user_id: String) -> Self {
        Self { db, user_id }
    }

    /// 从数据库获取版本同步信息
    pub async fn get_version_sync(
        &self,
    ) -> Result<Option<crate::im::conversation::models::LocalVersionSync>> {
        use crate::im::conversation::entities::local_version_sync::{Column, Entity};

        let model = Entity::find()
            .filter(Column::TableName.eq("local_conversations"))
            .filter(Column::EntityId.eq(&self.user_id))
            .one(&self.db)
            .await
            .context("查询版本同步信息失败")?;

        Ok(
            model.map(|model| crate::im::conversation::models::LocalVersionSync {
                table_name: model.table_name,
                entity_id: model.entity_id,
                version: model.version as u64,
                version_id: model.version_id,
            }),
        )
    }

    /// 保存版本同步信息到数据库
    pub async fn save_version_sync(
        &self,
        version_sync: &crate::im::conversation::models::LocalVersionSync,
    ) -> Result<()> {
        use crate::im::conversation::entities::local_version_sync::{ActiveModel, Column, Entity};
        use sea_orm::Set;

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
}
