//! 消息本地模型定义

/// 本地聊天记录结构体
#[derive(Debug, Clone)]
pub struct LocalChatLog {
    pub conversation_id: String,
    pub client_msg_id: String,
    pub server_msg_id: String,
    pub send_id: String,
    pub recv_id: String,
    pub sender_platform_id: i32,
    pub sender_nickname: String,
    pub sender_face_url: String,
    pub session_type: i32,
    pub msg_from: i32,
    pub content_type: i32,
    pub content: String,
    pub is_read: bool,
    pub status: i32,
    pub seq: i64,
    pub send_time: i64,
    pub create_time: i64,
    pub attached_info: String,
    pub ex: String,
    pub local_ex: String,
    pub group_id: String,
}

