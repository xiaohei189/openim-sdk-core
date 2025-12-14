//! 会话模块的 Sea-ORM 实体定义

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

