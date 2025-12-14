//! 联系人（好友）模块
//!
//! 实现 OpenIM SDK 的好友同步功能

pub mod api;
pub mod dao;
pub mod entities;
pub mod listener;
pub mod models;
pub mod service;
pub mod types;

// 重新导出主要类型和函数
pub use api::FriendApi;
pub use dao::FriendDao;
pub use listener::{EmptyFriendListener, FriendListener};
pub use models::{FriendSyncerConfig, LocalFriend};
pub use service::FriendSyncer;
pub use types::{AllFriendsResp, FriendRequest, FriendRequestsResp, IncrementalFriendsResp};

