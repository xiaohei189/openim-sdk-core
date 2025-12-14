//! 会话 HTTP API 客户端
//!
//! 负责所有会话相关的 HTTP 请求

use crate::im::conversation::types::{AllConversationsResp, IncrementalConversationResp};
use crate::im::types::ApiResponse;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info};
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

        #[derive(Deserialize, Serialize)]
        struct SeqInfo {
            #[serde(rename = "maxSeq")]
            max_seq: i64,
            #[serde(rename = "hasReadSeq")]
            has_read_seq: i64,
            #[serde(rename = "maxSeqTime", default)]
            max_seq_time: i64,
        }

        #[derive(Deserialize)]
        struct SeqsData {
            seqs: HashMap<String, SeqInfo>,
        }

        let status = response.status();
        let body_bytes = response.bytes().await.context("读取响应 body 失败")?;
        let body_str = String::from_utf8_lossy(&body_bytes);
        info!("[ConvAPI/Seq] 📥 服务器响应原始数据: {}", body_str);

        if !status.is_success() {
            error!(
                "[ConvAPI/Seq] 会话 Seq 请求失败，HTTP状态: {}, 响应: {}",
                status, body_str
            );
            return Err(anyhow::anyhow!("HTTP 错误 {}: {}", status, body_str));
        }

        let api_resp: ApiResponse<SeqsData> = serde_json::from_slice(&body_bytes).map_err(|e| {
            error!(
                "[ConvAPI/Seq] 会话 Seq 反序列化失败: {:?}\n原始响应: {}",
                e, body_str
            );
            anyhow::anyhow!("反序列化响应失败: {:?}", e)
        })?;

        if api_resp.err_code != 0 {
            error!(
                "[ConvAPI/Seq] 会话 Seq 服务器错误，错误码: {}, 错误信息: {}",
                api_resp.err_code, api_resp.err_msg
            );
            return Err(anyhow::anyhow!(
                "服务器错误 {}: {}",
                api_resp.err_code,
                api_resp.err_msg
            ));
        }

        let data = api_resp
            .data
            .ok_or_else(|| anyhow::anyhow!("响应中缺少 data 字段"))?;

        // 输出 data 字段内容（用于调试）
        if let Ok(data_str) = serde_json::to_string_pretty(&data.seqs) {
            info!("[ConvAPI/Seq] 📊 服务器返回的 data.seqs 字段: {}", data_str);
        }

        let mut result = HashMap::new();
        info!(
            "[ConvAPI/Seq] 📋 解析会话 Seq 对象，条目数: {}",
            data.seqs.len()
        );

        for (conv_id, seq_info) in data.seqs.iter() {
            let max_seq = seq_info.max_seq;
            let has_read_seq = seq_info.has_read_seq;
            let unread = (max_seq - has_read_seq).max(0);
            info!(
                "[ConvAPI/Seq]   conversationID={}, maxSeq={}, hasReadSeq={}, unreadCount={}",
                conv_id, max_seq, has_read_seq, unread
            );
            result.insert(conv_id.clone(), (max_seq, has_read_seq));
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

        let status = response.status();
        let body_bytes = response.bytes().await.context("读取响应 body 失败")?;
        let body_str = String::from_utf8_lossy(&body_bytes);
        info!("[ConvAPI] 增量会话同步响应 Body: {}", body_str);

        if !status.is_success() {
            error!(
                "[ConvAPI] 增量会话同步请求失败，HTTP状态: {}, 响应: {}",
                status, body_str
            );
            return Err(anyhow::anyhow!("HTTP 错误 {}: {}", status, body_str));
        }

        let api_resp: ApiResponse<IncrementalConversationResp> =
            serde_json::from_slice(&body_bytes).map_err(|e| {
                error!(
                    "[ConvAPI] 增量会话同步反序列化失败: {:?}\n原始响应: {}",
                    e, body_str
                );
                anyhow::anyhow!("反序列化响应失败: {:?}", e)
            })?;

        if api_resp.err_code != 0 {
            error!(
                "[ConvAPI] 增量会话同步服务器错误，错误码: {}, 错误信息: {}",
                api_resp.err_code, api_resp.err_msg
            );
            return Err(anyhow::anyhow!(
                "服务器错误 {}: {}",
                api_resp.err_code,
                api_resp.err_msg
            ));
        }

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

        let status = response.status();
        let body_bytes = response.bytes().await.context("读取响应 body 失败")?;
        let body_str = String::from_utf8_lossy(&body_bytes);
        info!("[ConvAPI] 全量会话同步响应 Body: {}", body_str);

        if !status.is_success() {
            error!(
                "[ConvAPI] 全量会话同步请求失败，HTTP状态: {}, 响应: {}",
                status, body_str
            );
            return Err(anyhow::anyhow!("HTTP 错误 {}: {}", status, body_str));
        }

        let api_resp: ApiResponse<AllConversationsResp> = serde_json::from_slice(&body_bytes)
            .map_err(|e| {
                error!(
                    "[ConvAPI] 全量会话同步反序列化失败: {:?}\n原始响应: {}",
                    e, body_str
                );
                anyhow::anyhow!("反序列化响应失败: {:?}", e)
            })?;

        if api_resp.err_code != 0 {
            error!(
                "[ConvAPI] 全量会话同步服务器错误，错误码: {}, 错误信息: {}",
                api_resp.err_code, api_resp.err_msg
            );
            return Err(anyhow::anyhow!(
                "服务器错误 {}: {}",
                api_resp.err_code,
                api_resp.err_msg
            ));
        }

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

        #[derive(Deserialize)]
        struct ConversationIdsData {
            #[serde(rename = "conversationIDs")]
            conversation_ids: Vec<String>,
        }

        let status = response.status();
        let body_bytes = response.bytes().await.context("读取响应 body 失败")?;
        let body_str = String::from_utf8_lossy(&body_bytes);
        info!("[ConvAPI] 会话ID列表响应 Body: {}", body_str);

        if !status.is_success() {
            error!(
                "[ConvAPI] 会话ID列表请求失败，HTTP状态: {}, 响应: {}",
                status, body_str
            );
            return Err(anyhow::anyhow!("HTTP 错误 {}: {}", status, body_str));
        }

        let api_resp: ApiResponse<ConversationIdsData> = serde_json::from_slice(&body_bytes)
            .map_err(|e| {
                error!(
                    "[ConvAPI] 会话ID列表反序列化失败: {:?}\n原始响应: {}",
                    e, body_str
                );
                anyhow::anyhow!("反序列化响应失败: {:?}", e)
            })?;

        if api_resp.err_code != 0 {
            error!(
                "[ConvAPI] 会话ID列表服务器错误，错误码: {}, 错误信息: {}",
                api_resp.err_code, api_resp.err_msg
            );
            return Err(anyhow::anyhow!(
                "服务器错误 {}: {}",
                api_resp.err_code,
                api_resp.err_msg
            ));
        }

        let data = api_resp
            .data
            .ok_or_else(|| anyhow::anyhow!("响应中缺少 data 字段"))?;

        info!("[ConvAPI] ✅ 会话 ID 列表响应");
        info!("[ConvAPI]   会话ID数: {}", data.conversation_ids.len());
        debug!("[ConvAPI]   会话ID列表: {:?}", data.conversation_ids);

        Ok(data.conversation_ids)
    }
}
