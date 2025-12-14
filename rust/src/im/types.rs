use serde::{Deserialize, Serialize};
use tracing::{debug, error};

/// WebSocket 消息类型标识符
pub mod msg_type {
    pub const WS_GET_NEWEST_SEQ: i32 = 1001;
    pub const WS_SEND_MSG: i32 = 1003;
    pub const WS_PUSH_MSG: i32 = 2001;
    pub const WS_KICK_ONLINE_MSG: i32 = 2002;
    pub const WS_LOGOUT_MSG: i32 = 2003;
    pub const WS_SEND_MSG_NOT_OSS: i32 = 3001; // 自定义：仿 go SendMessageNotOss
}

/// OpenIM 请求结构
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenIMReq {
    #[serde(rename = "reqIdentifier")]
    pub req_identifier: i32,
    pub token: String,
    #[serde(rename = "sendID")]
    pub send_id: String,
    #[serde(rename = "operationID")]
    pub operation_id: String,
    #[serde(rename = "msgIncr")]
    pub msg_incr: String,
    #[serde(default)]
    pub data: Vec<u8>,
}

/// OpenIM 响应结构（用于二进制消息）
#[derive(Debug, Deserialize, Serialize)]
pub struct OpenIMResp {
    #[serde(rename = "reqIdentifier")]
    pub req_identifier: i32,
    #[serde(rename = "msgIncr")]
    pub msg_incr: String,
    #[serde(rename = "operationID")]
    pub operation_id: String,
    #[serde(rename = "errCode")]
    pub err_code: i32,
    #[serde(rename = "errMsg")]
    pub err_msg: String,
    #[serde(
        default,
        deserialize_with = "crate::im::serialization::deserialize_base64"
    )]
    pub data: Vec<u8>,
}

/// WebSocket 连接响应结构（文本消息）
/// 用于 WebSocket 连接时的文本响应，包含 errDlt 字段
#[derive(Debug, Deserialize)]
pub struct WebSocketConnectResp {
    #[serde(rename = "errCode")]
    pub err_code: i32,
    #[serde(rename = "errMsg")]
    pub err_msg: String,
    #[serde(rename = "errDlt", default)]
    pub err_dlt: String,
    /// data 字段可能为 null、缺失或包含实际数据
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

/// 统一的 API 响应包装结构体（包含 errCode、errMsg、data）
/// data 字段可能为 null 或缺失，因此使用 Option<T>
/// serde 会自动将缺失或 null 的字段反序列化为 None
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    #[serde(rename = "errCode")]
    pub err_code: i32,
    #[serde(rename = "errMsg")]
    pub err_msg: String,
    pub data: Option<T>,
}

/// 通用 HTTP 响应处理函数：直接反序列化为统一的响应结构体
/// 返回 `ApiResponse<T>`，调用方可以根据需要处理 `data` 字段（可能为 None）
/// 所有 API 都可以共用此方法
pub async fn handle_http_response<T: serde::de::DeserializeOwned>(
    response: reqwest::Response,
    operation_name: &str,
) -> anyhow::Result<ApiResponse<T>> {
    use anyhow::Context;
    use tracing::{debug, error, info};

    let status = response.status();


    // 读取 body bytes（只能读取一次）
    let body_bytes = response.bytes().await.context("读取响应 body 失败")?;
    // 打印 body 内容
    let body_str = String::from_utf8_lossy(&body_bytes);
    info!("[HTTP] {}响应 Body: {}", operation_name, body_str);

    if !status.is_success() {
        error!(
            "[HTTP] {}请求失败，HTTP状态: {}, 响应: {}",
            operation_name, status, body_str
        );
        return Err(anyhow::anyhow!("HTTP 错误 {}: {}", status, body_str));
    }
    debug!("[HTTP] {}请求成功，HTTP状态: {}", operation_name, status);

    // 从 bytes 反序列化（因为 body 已经被消费了）
    let api_resp: ApiResponse<T> = serde_json::from_slice(&body_bytes).map_err(|e| {
        error!(
            "[HTTP] {}反序列化失败: {:?}\n原始响应: {}",
            operation_name, e, body_str
        );
        anyhow::anyhow!("反序列化响应失败: {:?}", e)
    })?;

    // 检查错误码
    if api_resp.err_code != 0 {
        error!(
            "[HTTP] {}服务器错误，错误码: {}, 错误信息: {}",
            operation_name, api_resp.err_code, api_resp.err_msg
        );
        return Err(anyhow::anyhow!(
            "服务器错误 {}: {}",
            api_resp.err_code,
            api_resp.err_msg
        ));
    }

    // 直接返回 ApiResponse，调用方可以根据需要处理 data 字段
    Ok(api_resp)
}

// ========== 会话相关结构体 ==========

/// 增量会话响应（业务逻辑层结构体，可直接从 API 响应反序列化）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalConversationResp {
    pub version: u64,
    pub version_id: String,
    pub full: bool,
    pub delete: Vec<String>,
    pub insert: Vec<LocalConversation>,
    pub update: Vec<LocalConversation>,
}

/// 全量会话响应（业务逻辑层结构体，可直接从 API 响应反序列化）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllConversationsResp {
    pub conversations: Vec<LocalConversation>,
}

/// 本地会话数据结构
/// 可以直接从服务器返回的 JSON 反序列化，缺失的字段使用默认值
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalConversation {
    /// 会话 ID
    #[serde(rename = "conversationID")]
    pub conversation_id: String,
    /// 会话类型：1=单聊, 2=普通群聊, 3=超级群聊, 4=通知会话
    pub conversation_type: i32,
    /// 用户 ID（单聊时使用）
    #[serde(default)]
    pub user_id: String,
    /// 群组 ID（群聊时使用）
    #[serde(default)]
    pub group_id: String,
    /// 显示名称（服务器不返回，需要从用户/群组信息获取）
    #[serde(default)]
    pub show_name: String,
    /// 头像 URL（服务器不返回，需要从用户/群组信息获取）
    #[serde(default)]
    pub face_url: String,
    /// 最新消息（服务器不返回，需要从消息获取）
    #[serde(default)]
    pub latest_msg: String,
    /// 最新消息发送时间（服务器不返回，需要从消息获取）
    #[serde(default)]
    pub latest_msg_send_time: i64,
    /// 未读消息数（服务器可能不返回）
    #[serde(default)]
    pub unread_count: i32,
    /// 接收消息选项：0=接收并通知, 1=接收但不通知, 2=屏蔽
    #[serde(default)]
    pub recv_msg_opt: i32,
    /// 是否置顶
    #[serde(default)]
    pub is_pinned: bool,
    /// 是否私聊
    #[serde(default)]
    pub is_private_chat: bool,
    /// 阅后即焚时长（秒）
    #[serde(default)]
    pub burn_duration: i32,
    /// 群@类型：0=正常, 1=@我, 2=@所有人
    #[serde(default)]
    pub group_at_type: i32,
    /// 是否不在群中
    #[serde(default)]
    pub is_not_in_group: bool,
    /// 更新未读数时间
    #[serde(default)]
    pub update_unread_count_time: i64,
    /// 附加信息
    #[serde(default)]
    pub attached_info: String,
    /// 扩展信息
    #[serde(default)]
    pub ex: String,
    /// 草稿文本
    #[serde(default)]
    pub draft_text: String,
    /// 草稿文本时间
    #[serde(default)]
    pub draft_text_time: i64,
    /// 最大序列号
    #[serde(default)]
    pub max_seq: i64,
    /// 最小序列号
    #[serde(default)]
    pub min_seq: i64,
    /// 是否消息销毁
    #[serde(default)]
    pub is_msg_destruct: bool,
    /// 消息销毁时间
    #[serde(default)]
    pub msg_destruct_time: i64,
}
