pub mod im;

// 重新导出常用类型和函数，方便外部使用
pub use im::{
    client::{ClientConfig, OpenIMClient},
    conversation::{ConversationSyncer, ConversationSyncerConfig, LocalConversation},
    friend::LocalFriend,
    login_async,
};
