//! Sea-ORM 实体定义

pub mod local_conversations {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    /// 本地会话实体
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
    #[sea_orm(table_name = "local_conversations")]
    pub struct Model {
        #[sea_orm(primary_key)]
        #[serde(rename = "conversationID")]
        pub conversation_id: String,
        
        #[serde(rename = "conversationType")]
        pub conversation_type: i32,
        
        #[serde(rename = "userID")]
        pub user_id: String,
        
        #[serde(rename = "groupID")]
        pub group_id: String,
        
        #[serde(rename = "showName")]
        pub show_name: String,
        
        #[serde(rename = "faceURL")]
        pub face_url: String,
        
        #[serde(rename = "latestMsg")]
        pub latest_msg: String,
        
        #[serde(rename = "latestMsgSendTime")]
        pub latest_msg_send_time: i64,
        
        #[serde(rename = "unreadCount")]
        pub unread_count: i32,
        
        #[serde(rename = "recvMsgOpt")]
        pub recv_msg_opt: i32,
        
        #[serde(rename = "isPinned")]
        pub is_pinned: i32,  // SQLite使用INTEGER存储布尔值
        
        #[serde(rename = "isPrivateChat")]
        pub is_private_chat: i32,
        
        #[serde(rename = "burnDuration")]
        pub burn_duration: i32,
        
        #[serde(rename = "groupAtType")]
        pub group_at_type: i32,
        
        #[serde(rename = "isNotInGroup")]
        pub is_not_in_group: i32,
        
        #[serde(rename = "updateUnreadCountTime")]
        pub update_unread_count_time: i64,
        
        #[serde(rename = "attachedInfo")]
        pub attached_info: String,
        
        #[serde(rename = "ex")]
        pub ex: String,
        
        #[serde(rename = "draftText")]
        pub draft_text: String,
        
        #[serde(rename = "draftTextTime")]
        pub draft_text_time: i64,
        
        #[serde(rename = "maxSeq")]
        pub max_seq: i64,
        
        #[serde(rename = "minSeq")]
        pub min_seq: i64,
        
        #[serde(rename = "isMsgDestruct")]
        pub is_msg_destruct: i32,
        
        #[serde(rename = "msgDestructTime")]
        pub msg_destruct_time: i64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

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

pub mod local_version_sync {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    /// 版本同步实体
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
    #[sea_orm(table_name = "local_version_sync")]
    pub struct Model {
        #[sea_orm(primary_key)]
        #[serde(rename = "tableName")]
        pub table_name: String,
        
        #[sea_orm(primary_key)]
        #[serde(rename = "entityID")]
        pub entity_id: String,
        
        pub version: i64,
        
        #[serde(rename = "versionID")]
        pub version_id: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

