//! 会话 HTTP API 客户端
//!
//! 负责所有会话相关的 HTTP 请求

use crate::im::conversation::types::{AllConversationsResp, IncrementalConversationResp};
use crate::im::types::handle_http_response;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// 会话相关的 HTTP API 客户端
pub struct ConversationApi {
    client: reqwest::Client,
    api_base_url: String,
    user_id: String,
}

impl ConversationApi {
    /// 创建新的会话 API 客户端
    ///
    /// `client` 应该已经在外部配置好认证拦截器
    pub fn new(client: reqwest::Client, api_base_url: String, user_id: String) -> Self {
        Self {
            client,
            api_base_url,
            user_id,
        }
    }

    /// 从服务器获取每个会话的 MaxSeq 和 HasReadSeq
    pub async fn get_has_read_and_max_seqs(&self) -> Result<HashMap<String, (i64, i64)>> {
        let operation_id = Uuid::new_v4().to_string();
        let url = format!(
            "{}/msg/get_conversations_has_read_and_max_seq",
            self.api_base_url
        );

        info!("[ConvAPI/Seq] 📡 请求会话 Seq 信息");
        debug!("[ConvAPI/Seq]   请求URL: {}", url);
        debug!(
            "[ConvAPI/Seq]   用户ID: {}, 操作ID: {}",
            self.user_id, operation_id
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("operationID", &operation_id)
            .json(&serde_json::json!({
                "userID": self.user_id,
            }))
            .send()
            .await
            .context("请求失败")?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            error!(
                "[ConvAPI/Seq] 会话 Seq 请求失败，HTTP状态: {}, 响应: {}",
                status, text
            );
            return Err(anyhow::anyhow!("HTTP 错误 {}: {}", status, text));
        }
        debug!("[ConvAPI/Seq] 会话 Seq 请求成功，HTTP状态: {}", status);

        let text = response.text().await.context("读取响应失败")?;
        let json_value: serde_json::Value =
            serde_json::from_str(&text).context("解析 JSON 失败")?;

        // 输出原始响应数据（用于调试）
        info!("[ConvAPI/Seq] 📥 服务器响应原始数据: {}", text);

        // 检查错误码
        if let Some(err_code) = json_value.get("errCode").and_then(|v| v.as_i64()) {
            if err_code != 0 {
                let err_msg = json_value
                    .get("errMsg")
                    .and_then(|v| v.as_str())
                    .unwrap_or("未知错误");
                error!(
                    "[ConvAPI/Seq] 会话 Seq 服务器错误，错误码: {}, 错误信息: {}",
                    err_code, err_msg
                );
                return Err(anyhow::anyhow!("服务器错误 {}: {}", err_code, err_msg));
            }
        }

        let data = json_value
            .get("data")
            .ok_or_else(|| anyhow::anyhow!("响应中缺少 data 字段"))?;

        // 输出 data 字段内容（用于调试）
        if let Ok(data_str) = serde_json::to_string_pretty(data) {
            info!("[ConvAPI/Seq] 📊 服务器返回的 data 字段: {}", data_str);
        }

        // 期望结构：data.seqs: { conversationID: { maxSeq, hasReadSeq, maxSeqTime }, ... }
        let mut result = HashMap::new();

        // 先尝试作为对象（HashMap）解析
        if let Some(seqs_obj) = data.get("seqs").and_then(|v| v.as_object()) {
            info!(
                "[ConvAPI/Seq] 📋 解析会话 Seq 对象，条目数: {}",
                seqs_obj.len()
            );
            for (conv_id, seq_data) in seqs_obj.iter() {
                if let Some(seq_obj) = seq_data.as_object() {
                    let max_seq = seq_obj.get("maxSeq").and_then(|v| v.as_i64()).unwrap_or(0);
                    let has_read_seq = seq_obj
                        .get("hasReadSeq")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    let unread = (max_seq - has_read_seq).max(0);
                    info!(
                        "[ConvAPI/Seq]   conversationID={}, maxSeq={}, hasReadSeq={}, unreadCount={}",
                        conv_id, max_seq, has_read_seq, unread
                    );
                    result.insert(conv_id.clone(), (max_seq, has_read_seq));
                } else {
                    warn!("[ConvAPI/Seq]   跳过无效条目（seq 数据不是对象）: conversationID={}, data={:?}", conv_id, seq_data);
                }
            }
        }
        // 兼容旧格式：数组格式（虽然服务器不返回，但保留兼容性）
        else if let Some(arr) = data.get("seqs").and_then(|v| v.as_array()) {
            warn!(
                "[ConvAPI/Seq] ⚠️ 收到数组格式的 seqs（旧格式），条目数: {}",
                arr.len()
            );
            for item in arr {
                if let Some(obj) = item.as_object() {
                    if let Some(conv_id) = obj.get("conversationID").and_then(|v| v.as_str()) {
                        let max_seq = obj.get("maxSeq").and_then(|v| v.as_i64()).unwrap_or(0);
                        let has_read_seq =
                            obj.get("hasReadSeq").and_then(|v| v.as_i64()).unwrap_or(0);
                        result.insert(conv_id.to_string(), (max_seq, has_read_seq));
                    }
                }
            }
        } else {
            warn!("[ConvAPI/Seq] ⚠️ data.seqs 字段不存在或格式不正确");
        }

        info!(
            "[ConvAPI/Seq] ✅ 解析完成，共 {} 个会话的 Seq 信息",
            result.len()
        );

        Ok(result)
    }

    /// 从服务器获取增量会话
    pub async fn get_incremental_conversations(
        &self,
        version: u64,
        version_id: &str,
    ) -> Result<IncrementalConversationResp> {
        let operation_id = Uuid::new_v4().to_string();
        let url = format!(
            "{}/conversation/get_incremental_conversations",
            self.api_base_url
        );

        info!(
            "[ConvAPI] 📡 请求增量会话同步\n   请求URL: {}\n   版本: {}, 版本ID: {}\n   用户ID: {}\n   操作ID: {}",
            url, version, version_id, self.user_id, operation_id
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("operationID", &operation_id)
            .json(&serde_json::json!({
                "userID": self.user_id,
                "version": version,
                "versionID": version_id
            }))
            .send()
            .await
            .context("请求失败")?;

        // 直接反序列化为业务逻辑层结构体
        let api_resp =
            handle_http_response::<IncrementalConversationResp>(response, "增量会话同步").await?;
        let resp = api_resp
            .data
            .ok_or_else(|| anyhow::anyhow!("响应中缺少 data 字段"))?;

        Ok(resp)
    }

    /// 从服务器获取所有会话
    pub async fn get_all_conversations(&self) -> Result<AllConversationsResp> {
        let operation_id = Uuid::new_v4().to_string();
        let url = format!("{}/conversation/get_all_conversations", self.api_base_url);

        info!("[ConvAPI] 📡 请求全量会话同步");
        debug!("[ConvAPI]   请求URL: {}", url);
        debug!(
            "[ConvAPI]   用户ID: {}, 操作ID: {}",
            self.user_id, operation_id
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("operationID", &operation_id)
            .json(&serde_json::json!({
                "ownerUserID": self.user_id
            }))
            .send()
            .await
            .context("请求失败")?;

        // 直接反序列化为业务逻辑层结构体
        let api_resp =
            handle_http_response::<AllConversationsResp>(response, "全量会话同步").await?;
        let resp = api_resp
            .data
            .ok_or_else(|| anyhow::anyhow!("响应中缺少 data 字段"))?;

        info!(
            "[ConvAPI] ✅ 全量会话同步响应，会话数: {}",
            resp.conversations.len()
        );
        debug!(
            "[ConvAPI]   会话详情: {:?}",
            resp.conversations
                .iter()
                .map(|c| &c.conversation_id)
                .collect::<Vec<_>>()
        );

        Ok(resp)
    }

    /// 从服务器获取所有会话 ID
    pub async fn get_all_conversation_ids(&self) -> Result<Vec<String>> {
        let operation_id = Uuid::new_v4().to_string();
        let url = format!(
            "{}/conversation/get_full_conversation_ids",
            self.api_base_url
        );

        info!("[ConvAPI] 📡 请求会话 ID 列表");
        debug!("[ConvAPI]   请求URL: {}, 操作ID: {}", url, operation_id);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("operationID", &operation_id)
            .json(&serde_json::json!({
                "userID": self.user_id
            }))
            .send()
            .await
            .context("请求失败")?;

        // 使用通用响应处理
        #[derive(Deserialize)]
        struct ConversationIdsData {
            #[serde(rename = "conversationIDs")]
            conversation_ids: Vec<String>,
        }

        let api_resp = handle_http_response::<ConversationIdsData>(response, "会话ID列表").await?;
        let data = api_resp
            .data
            .ok_or_else(|| anyhow::anyhow!("响应中缺少 data 字段"))?;

        info!("[ConvAPI] ✅ 会话 ID 列表响应");
        info!("[ConvAPI]   会话ID数: {}", data.conversation_ids.len());
        debug!("[ConvAPI]   会话ID列表: {:?}", data.conversation_ids);

        Ok(data.conversation_ids)
    }
}
