//! 会话模块
//!
//! 实现 OpenIM SDK 的会话同步功能

pub mod api;
pub mod dao;
pub mod entities;
pub mod listener;
pub mod models;
pub mod service;
pub mod types;

// 重新导出主要类型和函数
pub use api::ConversationApi;
pub use dao::{ConversationDao, VersionSyncDao};
pub use listener::{ConversationListener, EmptyConversationListener};
pub use models::{ConversationSyncerConfig, LocalVersionSync};
pub use service::ConversationSyncer;
pub use types::{AllConversationsResp, IncrementalConversationResp};


