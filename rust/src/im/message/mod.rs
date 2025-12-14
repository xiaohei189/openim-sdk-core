//! 消息模块
//!
//! 实现 OpenIM SDK 的消息处理功能

pub mod dao;
pub mod listener;
pub mod models;
pub mod types;

// 重新导出主要类型和函数
pub use dao::MessageStore;
pub use listener::{AdvancedMsgListener, EmptyAdvancedMsgListener};
pub use models::LocalChatLog;
pub use types::{
    AtElem, AtInfo, CustomElem, FileElem, LocationElem, MarkdownEntityElem, MarkdownTextElem,
    MessageRevoked, MsgStruct, OANotificationElem, PictureElem, PictureBaseInfo, QuoteElem,
    RevokeElem, SoundElem, StreamMsgElem, TextElem, VideoElem,
};

