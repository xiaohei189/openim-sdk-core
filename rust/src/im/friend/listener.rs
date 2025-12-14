//! 好友监听器回调接口

use async_trait::async_trait;

/// 好友监听器回调接口（类似 Go SDK 中 RelationListener 的一部分能力）
#[async_trait]
pub trait FriendListener: Send + Sync {
    /// 好友列表发生变更（新增或更新），参数为 JSON 数组字符串
    async fn on_friend_list_changed(&self, friends_json: String);

    /// 黑名单列表发生变更（全量同步结果），参数为 JSON 数组字符串
    async fn on_black_list_changed(&self, blacks_json: String);

    /// 好友申请列表发生变更（全量同步结果），参数为 JSON 数组字符串
    async fn on_friend_request_list_changed(&self, requests_json: String);
}

/// 默认空实现（无操作）
pub struct EmptyFriendListener;

#[async_trait]
impl FriendListener for EmptyFriendListener {
    async fn on_friend_list_changed(&self, _friends_json: String) {
        // 默认不做任何处理
    }

    async fn on_black_list_changed(&self, _blacks_json: String) {
        // 默认不做任何处理
    }

    async fn on_friend_request_list_changed(&self, _requests_json: String) {
        // 默认不做任何处理
    }
}

