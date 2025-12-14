//! 好友模块的 Sea-ORM 实体定义

pub mod local_friends {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    /// 本地好友实体（对应 Go SDK 中的 LocalFriend）
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
    #[sea_orm(table_name = "local_friends")]
    pub struct Model {
        /// 拥有者用户 ID
        #[sea_orm(primary_key)]
        #[serde(rename = "ownerUserID")]
        pub owner_user_id: String,

        /// 好友用户 ID
        #[sea_orm(primary_key)]
        #[serde(rename = "userID")]
        pub friend_user_id: String,

        /// 备注
        #[serde(rename = "remark")]
        pub remark: String,

        /// 创建时间（毫秒）
        #[serde(rename = "createTime")]
        pub create_time: i64,

        /// 添加来源
        #[serde(rename = "addSource")]
        pub add_source: i32,

        /// 操作人 ID
        #[serde(rename = "operatorUserID")]
        pub operator_user_id: String,

        /// 昵称
        #[serde(rename = "nickname")]
        pub nickname: String,

        /// 头像 URL
        #[serde(rename = "faceURL")]
        pub face_url: String,

        /// 扩展信息
        #[serde(rename = "ex")]
        pub ex: String,

        /// 附加信息
        #[serde(rename = "attachedInfo")]
        pub attached_info: String,

        /// 是否置顶（SQLite 中使用 INTEGER 存储布尔值）
        #[serde(rename = "isPinned")]
        pub is_pinned: i32,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

