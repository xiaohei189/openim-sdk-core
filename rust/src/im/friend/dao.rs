//! 好友数据访问层（DAO）
//!
//! 负责所有好友相关的数据库操作，将数据访问逻辑与业务逻辑分离

use crate::im::conversation::models::LocalVersionSync;
use crate::im::friend::entities::local_friends;
use crate::im::friend::models::LocalFriend;
use anyhow::{Context, Result};
use sea_orm::{
    sea_query::OnConflict, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use tracing::{debug, info};

/// 好友 DAO
pub struct FriendDao {
    db: DatabaseConnection,
    user_id: String,
}

impl FriendDao {
    /// 创建新的好友 DAO
    pub fn new(db: DatabaseConnection, user_id: String) -> Self {
        Self { db, user_id }
    }

    /// 初始化数据库表结构
    pub async fn init_db(&self) -> Result<()> {
        info!("[FriendDAO/DB] 初始化数据库表结构");

        use sea_orm::ConnectionTrait;

        let sql = r#"
            CREATE TABLE IF NOT EXISTS local_friends (
                owner_user_id TEXT NOT NULL,
                friend_user_id TEXT NOT NULL,
                remark TEXT NOT NULL DEFAULT '',
                create_time INTEGER NOT NULL DEFAULT 0,
                add_source INTEGER NOT NULL DEFAULT 0,
                operator_user_id TEXT NOT NULL DEFAULT '',
                nickname TEXT NOT NULL DEFAULT '',
                face_url TEXT NOT NULL DEFAULT '',
                ex TEXT NOT NULL DEFAULT '',
                attached_info TEXT NOT NULL DEFAULT '',
                is_pinned INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (owner_user_id, friend_user_id)
            )
        "#;

        self.db
            .execute_unprepared(sql)
            .await
            .context("创建好友表失败")?;

        info!("[FriendDAO/DB] 数据库表初始化完成");
        Ok(())
    }

    /// 使用共享数据库连接初始化数据库表结构（静态方法）
    pub async fn init_db_with_connection(db: &DatabaseConnection) -> Result<()> {
        info!("[FriendDAO/DB] 初始化好友数据库表结构");

        use sea_orm::ConnectionTrait;

        let sql = r#"
            CREATE TABLE IF NOT EXISTS local_friends (
                owner_user_id TEXT NOT NULL,
                friend_user_id TEXT NOT NULL,
                remark TEXT NOT NULL DEFAULT '',
                create_time INTEGER NOT NULL DEFAULT 0,
                add_source INTEGER NOT NULL DEFAULT 0,
                operator_user_id TEXT NOT NULL DEFAULT '',
                nickname TEXT NOT NULL DEFAULT '',
                face_url TEXT NOT NULL DEFAULT '',
                ex TEXT NOT NULL DEFAULT '',
                attached_info TEXT NOT NULL DEFAULT '',
                is_pinned INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (owner_user_id, friend_user_id)
            )
        "#;

        db.execute_unprepared(sql)
            .await
            .context("创建好友表失败")?;

        info!("[FriendDAO/DB] 数据库表初始化完成");
        Ok(())
    }

    /// 从数据库获取所有好友
    pub async fn get_all_friends(&self) -> Result<Vec<LocalFriend>> {
        let models = local_friends::Entity::find()
            .filter(local_friends::Column::OwnerUserId.eq(self.user_id.clone()))
            .all(&self.db)
            .await
            .context("查询好友列表失败")?;

        let friends: Vec<LocalFriend> = models
            .into_iter()
            .map(|m| LocalFriend {
                owner_user_id: m.owner_user_id,
                friend_user_id: m.friend_user_id,
                remark: m.remark,
                create_time: m.create_time,
                add_source: m.add_source,
                operator_user_id: m.operator_user_id,
                nickname: m.nickname,
                face_url: m.face_url,
                ex: m.ex,
                attached_info: m.attached_info,
                is_pinned: m.is_pinned != 0,
            })
            .collect();

        debug!(
            "[FriendDAO] 获取本地好友列表，共 {} 个好友",
            friends.len()
        );
        Ok(friends)
    }

    /// 获取本地所有好友的 userID 列表
    pub async fn get_all_friend_ids(&self) -> Result<Vec<String>> {
        let models = local_friends::Entity::find()
            .filter(local_friends::Column::OwnerUserId.eq(self.user_id.clone()))
            .all(&self.db)
            .await
            .context("查询好友ID列表失败")?;

        let ids = models
            .into_iter()
            .map(|m| m.friend_user_id)
            .collect::<Vec<_>>();
        debug!("[FriendDAO] 获取本地好友ID列表，共 {} 个", ids.len());
        Ok(ids)
    }

    /// 从数据库获取版本同步信息（tableName = local_friends）
    pub async fn get_version_sync(&self) -> Result<Option<LocalVersionSync>> {
        use crate::im::conversation::entities::local_version_sync::{Column, Entity};

        let model = Entity::find()
            .filter(Column::TableName.eq("local_friends"))
            .filter(Column::EntityId.eq(&self.user_id))
            .one(&self.db)
            .await
            .context("查询好友版本同步信息失败")?;

        Ok(model.map(|m| LocalVersionSync {
            table_name: m.table_name,
            entity_id: m.entity_id,
            version: m.version as u64,
            version_id: m.version_id,
        }))
    }

    /// 保存版本同步信息到数据库
    pub async fn save_version_sync(&self, version_sync: &LocalVersionSync) -> Result<()> {
        use crate::im::conversation::entities::local_version_sync::{ActiveModel, Column, Entity};

        let active = ActiveModel {
            table_name: Set(version_sync.table_name.clone()),
            entity_id: Set(version_sync.entity_id.clone()),
            version: Set(version_sync.version as i64),
            version_id: Set(version_sync.version_id.clone()),
        };

        Entity::insert(active)
            .on_conflict(
                OnConflict::columns([Column::TableName, Column::EntityId])
                    .update_columns([Column::Version, Column::VersionId])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .context("保存好友版本同步信息失败")?;
        Ok(())
    }

    /// 插入或更新好友到数据库
    pub async fn upsert_friend(&self, f: &LocalFriend) -> Result<()> {
        use crate::im::friend::entities::local_friends::ActiveModel;

        let active = ActiveModel {
            owner_user_id: Set(f.owner_user_id.clone()),
            friend_user_id: Set(f.friend_user_id.clone()),
            remark: Set(f.remark.clone()),
            create_time: Set(f.create_time),
            add_source: Set(f.add_source),
            operator_user_id: Set(f.operator_user_id.clone()),
            nickname: Set(f.nickname.clone()),
            face_url: Set(f.face_url.clone()),
            ex: Set(f.ex.clone()),
            attached_info: Set(f.attached_info.clone()),
            is_pinned: Set(if f.is_pinned { 1 } else { 0 }),
        };

        local_friends::Entity::insert(active)
            .on_conflict(
                OnConflict::columns([
                    local_friends::Column::OwnerUserId,
                    local_friends::Column::FriendUserId,
                ])
                .update_columns([
                    local_friends::Column::Remark,
                    local_friends::Column::CreateTime,
                    local_friends::Column::AddSource,
                    local_friends::Column::OperatorUserId,
                    local_friends::Column::Nickname,
                    local_friends::Column::FaceUrl,
                    local_friends::Column::Ex,
                    local_friends::Column::AttachedInfo,
                    local_friends::Column::IsPinned,
                ])
                .to_owned(),
            )
            .exec(&self.db)
            .await
            .context("插入或更新好友失败")?;
        Ok(())
    }

    /// 从数据库删除好友
    pub async fn delete_friend(&self, friend_user_id: &str) -> Result<()> {
        local_friends::Entity::delete_many()
            .filter(local_friends::Column::OwnerUserId.eq(self.user_id.clone()))
            .filter(local_friends::Column::FriendUserId.eq(friend_user_id))
            .exec(&self.db)
            .await
            .context("删除好友失败")?;
        Ok(())
    }
}

