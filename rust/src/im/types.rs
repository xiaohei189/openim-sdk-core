use serde::{Deserialize, Serialize};

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

/// OpenIM 响应结构
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

/// 服务器响应结构
#[derive(Debug, Deserialize)]
pub struct ServerResponse {
    #[serde(rename = "errCode")]
    pub err_code: i32,
    #[serde(rename = "errMsg")]
    pub err_msg: String,
}

