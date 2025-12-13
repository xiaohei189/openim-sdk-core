use serde::{Deserialize, Serialize};
use tracing::{debug, info};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    #[serde(rename = "areaCode")]
    pub area_code: String,
    #[serde(rename = "phoneNumber")]
    pub phone_number: String,
    pub password: String,
    pub platform: i32,
}

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    #[serde(rename = "errCode")]
    pub err_code: i32,
    #[serde(rename = "errMsg")]
    pub err_msg: String,
    pub data: Option<LoginData>,
}

#[derive(Debug, Deserialize)]
pub struct LoginData {
    #[serde(rename = "imToken")]
    pub im_token: String,
    #[serde(rename = "chatToken")]
    pub chat_token: String,
    #[serde(rename = "userID")]
    pub user_id: String,
}

pub async fn login_async(area_code: String, phone_number: String, password: String, platform: i32) -> Result<LoginResponse, String> {
    use uuid::Uuid;
    
    let client = reqwest::Client::new();
    let operation_id = Uuid::new_v4().to_string();
    
    let login_req = LoginRequest {
        area_code,
        phone_number,
        password,
        platform,
    };
    
    let url = "http://localhost:10008/account/login";
    
    info!("🔐 正在登录...");
    debug!("   URL: {}", url);
    debug!("   手机号: {}", login_req.phone_number);
    debug!("   OperationID: {}", operation_id);
    
    let response = client
        .post(url)
        .header("Accept", "application/json, text/plain, */*")
        .header("Accept-Language", "zh-CN,zh;q=0.9")
        .header("Cache-Control", "no-cache")
        .header("Connection", "keep-alive")
        .header("Content-Type", "application/json")
        .header("Origin", "http://localhost:11001")
        .header("Pragma", "no-cache")
        .header("Referer", "http://localhost:11001/")
        .header("Sec-Fetch-Dest", "empty")
        .header("Sec-Fetch-Mode", "cors")
        .header("Sec-Fetch-Site", "same-site")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36")
        .header("operationID", &operation_id)
        .header("sec-ch-ua", r#""Not)A;Brand";v="8", "Chromium";v="138", "Google Chrome";v="138""#)
        .header("sec-ch-ua-mobile", "?0")
        .header("sec-ch-ua-platform", r#""Windows""#)
        .json(&login_req)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;
    
    let status = response.status();
    let text = response.text().await.map_err(|e| format!("读取响应失败: {}", e))?;
    
    if !status.is_success() {
        return Err(format!("HTTP 错误 {}: {}", status, text));
    }
    
    debug!("✅ 登录响应: {}", text);
    
    let login_resp: LoginResponse = serde_json::from_str(&text)
        .map_err(|e| format!("解析响应失败: {}，原始响应: {}", e, text))?;
    
    return Ok(login_resp);

}

