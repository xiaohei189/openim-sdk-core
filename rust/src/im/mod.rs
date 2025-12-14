pub mod auth;
pub mod client;
pub mod conversation;
pub mod friend;
pub mod message;
pub mod serialization;
pub mod types;

// 重新导出认证相关函数
pub use auth::login_async;

// 重新导出会话同步相关类型和函数
pub use conversation::{ConversationSyncer, ConversationSyncerConfig, LocalVersionSync};

// 重新导出好友相关类型和函数
pub use friend::{FriendSyncer, FriendSyncerConfig, LocalFriend};

// 重新导出消息相关类型和函数
pub use message::{
    AdvancedMsgListener, AtElem, AtInfo, CustomElem, EmptyAdvancedMsgListener, FileElem,
    LocalChatLog, LocationElem, MarkdownEntityElem, MarkdownTextElem, MessageStore, MsgStruct,
    PictureBaseInfo, PictureElem, QuoteElem, SoundElem, VideoElem,
};

// 重新导出类型相关结构体和函数
pub use types::{
    AllConversationsResp, ApiResponse, IncrementalConversationResp, LocalConversation,
    WebSocketConnectResp,
};
