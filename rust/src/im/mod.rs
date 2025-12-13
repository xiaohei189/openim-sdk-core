pub mod advanced_msg_listener;
pub mod auth;
pub mod client;
pub mod conversation;
pub mod conversation_dao;
pub mod entities;
pub mod friend;
pub mod message_store;
pub mod msg;
pub mod serialization;
pub mod types;

// 重新导出认证相关函数
pub use auth::login_async;

// 重新导出会话同步相关类型和函数
pub use conversation::{ConversationSyncer, ConversationSyncerConfig, LocalVersionSync};

// 重新导出类型相关结构体和函数
pub use types::{
    handle_http_response, AllConversationsResp, ApiResponse, IncrementalConversationResp,
    LocalConversation, WebSocketConnectResp,
};
