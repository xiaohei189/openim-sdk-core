//! 会话监听器回调接口

use async_trait::async_trait;

/// 会话监听器回调接口（对应 Go 版本的 OnConversationListener）
#[async_trait]
pub trait ConversationListener: Send + Sync {
    /// 同步服务器开始
    async fn on_sync_server_start(&self, reinstalled: bool);

    /// 同步服务器完成
    async fn on_sync_server_finish(&self, reinstalled: bool);

    /// 同步服务器进度
    async fn on_sync_server_progress(&self, progress: i32);

    /// 同步服务器失败
    async fn on_sync_server_failed(&self, reinstalled: bool);

    /// 新会话
    async fn on_new_conversation(&self, conversation_list: String);

    /// 会话变更
    async fn on_conversation_changed(&self, conversation_list: String);

    /// 总未读消息数变更
    async fn on_total_unread_message_count_changed(&self, total_unread_count: i32);

    /// 会话用户输入状态变更
    async fn on_conversation_user_input_status_changed(&self, change: String);
}

/// 空实现（默认监听器）
pub struct EmptyConversationListener;

#[async_trait]
impl ConversationListener for EmptyConversationListener {
    async fn on_sync_server_start(&self, _reinstalled: bool) {}
    async fn on_sync_server_finish(&self, _reinstalled: bool) {}
    async fn on_sync_server_progress(&self, _progress: i32) {}
    async fn on_sync_server_failed(&self, _reinstalled: bool) {}
    async fn on_new_conversation(&self, _conversation_list: String) {}
    async fn on_conversation_changed(&self, _conversation_list: String) {}
    async fn on_total_unread_message_count_changed(&self, _total_unread_count: i32) {}
    async fn on_conversation_user_input_status_changed(&self, _change: String) {}
}
