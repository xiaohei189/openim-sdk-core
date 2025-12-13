//! 高级消息监听器（参考 Go 版本的 OnAdvancedMsgListener）
//!
//! 此模块定义了消息相关的回调接口，与 Go 版本的 `OnAdvancedMsgListener` 对应。

use async_trait::async_trait;

/// 高级消息监听器（参考 Go 版本的 OnAdvancedMsgListener）
///
/// 与 Go 版本的 `OnAdvancedMsgListener` 接口对应，用于接收各种消息事件。
#[async_trait]
pub trait AdvancedMsgListener: Send + Sync {
    /// 收到新消息（在线消息）
    ///
    /// 参数 `message` 是消息的 JSON 字符串表示（对应 Go 版本的 `MsgStruct`）
    async fn on_recv_new_message(&self, message: String);

    /// 收到 C2C 已读回执
    ///
    /// 参数 `msg_receipt_list` 是已读回执列表的 JSON 字符串表示
    async fn on_recv_c2c_read_receipt(&self, msg_receipt_list: String);

    /// 收到消息撤回通知
    ///
    /// 参数 `message_revoked` 是撤回消息信息的 JSON 字符串表示
    async fn on_new_recv_message_revoked(&self, message_revoked: String);

    /// 收到离线新消息
    ///
    /// 参数 `message` 是消息的 JSON 字符串表示（对应 Go 版本的 `MsgStruct`）
    async fn on_recv_offline_new_message(&self, message: String);

    /// 消息被删除
    ///
    /// 参数 `message` 是删除消息信息的 JSON 字符串表示
    async fn on_msg_deleted(&self, message: String);

    /// 收到仅在线消息（不存储到本地）
    ///
    /// 参数 `message` 是消息的 JSON 字符串表示（对应 Go 版本的 `MsgStruct`）
    async fn on_recv_online_only_message(&self, message: String);

    /// 被踢下线
    async fn on_kicked_offline(&self);

    /// 连接状态变化
    ///
    /// 参数 `connected` 表示是否已连接
    /// 参数 `message` 是状态消息
    async fn on_connection_status_changed(&self, connected: bool, message: String);

    /// 收到输入提示（typing）状态
    ///
    /// 参数 `typing_info` 是输入提示信息的 JSON 字符串表示，包含：
    /// - `conversationID`: 会话 ID
    /// - `sendID`: 发送者 ID
    /// - `msgTip`: 提示信息
    async fn on_recv_typing_status(&self, typing_info: String);
}

/// 空的消息监听器实现（默认实现）
pub struct EmptyAdvancedMsgListener;

#[async_trait]
impl AdvancedMsgListener for EmptyAdvancedMsgListener {
    async fn on_recv_new_message(&self, _message: String) {}
    async fn on_recv_c2c_read_receipt(&self, _msg_receipt_list: String) {}
    async fn on_new_recv_message_revoked(&self, _message_revoked: String) {}
    async fn on_recv_offline_new_message(&self, _message: String) {}
    async fn on_msg_deleted(&self, _message: String) {}
    async fn on_recv_online_only_message(&self, _message: String) {}
    async fn on_kicked_offline(&self) {}
    async fn on_connection_status_changed(&self, _connected: bool, _message: String) {}
    async fn on_recv_typing_status(&self, _typing_info: String) {}
}
