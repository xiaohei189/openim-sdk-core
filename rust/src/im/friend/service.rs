//! 好友同步服务层
//!
//! 实现 OpenIM SDK 的好友增量同步逻辑，参考 Go 版本的实现

use crate::im::conversation::models::LocalVersionSync;
use crate::im::friend::api::FriendApi;
use crate::im::friend::dao::FriendDao;
use crate::im::friend::listener::{EmptyFriendListener, FriendListener};
use crate::im::friend::models::{FriendSyncerConfig, LocalFriend};
use anyhow::{Context, Result};
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info};

/// 好友同步器
pub struct FriendSyncer {
    config: FriendSyncerConfig,
    /// 好友 API 客户端
    api: FriendApi,
    /// 好友 DAO
    friend_dao: FriendDao,
    /// 好友监听器
    listener: Arc<dyn FriendListener>,
}

impl FriendSyncer {
    /// 创建新的好友同步器（使用默认空监听器）
    pub async fn new(config: FriendSyncerConfig) -> Result<Self> {
        Self::with_listener(config, Arc::new(EmptyFriendListener)).await
    }

    /// 创建新的好友同步器（带自定义监听器，内部创建连接池）
    pub async fn with_listener(
        config: FriendSyncerConfig,
        listener: Arc<dyn FriendListener>,
    ) -> Result<Self> {
        let db_url = config.db_path.clone();
        info!(
            "[FriendSync] 创建好友同步器，用户ID: {}, SQLite数据库: {}",
            config.user_id, db_url
        );

        let db = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await
            .context(format!("连接SQLite数据库失败: {}", db_url))?;

        // 创建带认证拦截器的 HTTP 客户端（token 通过 default_headers 自动添加）
        let http_client = reqwest::ClientBuilder::new()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::HeaderName::from_static("token"),
                    reqwest::header::HeaderValue::from_str(&config.token)
                        .context("无效的 token")?,
                );
                headers
            })
            .build()
            .context("创建 HTTP 客户端失败")?;
        let api = FriendApi::new(
            http_client,
            config.api_base_url.clone(),
            config.user_id.clone(),
        );
        let friend_dao = FriendDao::new(db, config.user_id.clone());
        Ok(Self {
            api,
            friend_dao,
            listener,
            config,
        })
    }

    /// 创建新的好友同步器（使用共享连接池）
    pub async fn with_listener_and_db(
        config: FriendSyncerConfig,
        listener: Arc<dyn FriendListener>,
        db: Arc<Pool<Sqlite>>,
    ) -> Result<Self> {
        // 创建带认证拦截器的 HTTP 客户端（token 通过 default_headers 自动添加）
        let http_client = reqwest::ClientBuilder::new()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::HeaderName::from_static("token"),
                    reqwest::header::HeaderValue::from_str(&config.token)
                        .context("无效的 token")?,
                );
                headers
            })
            .build()
            .context("创建 HTTP 客户端失败")?;

        info!(
            "[FriendSync] 创建好友同步器（使用共享连接池），用户ID: {}",
            config.user_id
        );

        Ok(Self {
            api: FriendApi::new(
                http_client,
                config.api_base_url.clone(),
                config.user_id.clone(),
            ),
            friend_dao: FriendDao::new((*db).clone(), config.user_id.clone()),
            listener,
            config,
        })
    }

    /// 从数据库获取所有好友
    pub async fn get_all_friends(&self) -> Result<Vec<LocalFriend>> {
        self.friend_dao.get_all_friends().await
    }

    /// 获取本地所有好友的 userID 列表
    async fn get_all_friend_ids(&self) -> Result<Vec<String>> {
        self.friend_dao.get_all_friend_ids().await
    }

    /// 从数据库获取版本同步信息
    async fn get_version_sync(&self) -> Result<Option<LocalVersionSync>> {
        self.friend_dao.get_version_sync().await
    }

    /// 保存版本同步信息到数据库
    async fn save_version_sync(&self, version_sync: &LocalVersionSync) -> Result<()> {
        self.friend_dao.save_version_sync(version_sync).await
    }

    /// 插入或更新好友到数据库
    async fn upsert_friend(&self, f: &LocalFriend) -> Result<()> {
        self.friend_dao.upsert_friend(f).await
    }

    /// 从数据库删除好友
    async fn delete_friend(&self, friend_user_id: &str) -> Result<()> {
        self.friend_dao.delete_friend(friend_user_id).await
    }

    /// 同步好友列表（对比服务器和本地数据）
    async fn sync_friends(
        &self,
        server_friends: Vec<LocalFriend>,
        local_friends: Vec<LocalFriend>,
        is_full: bool,
    ) -> Result<()> {
        info!(
            "[FriendSync] 开始同步好友，服务器好友数: {}, 本地好友数: {}",
            server_friends.len(),
            local_friends.len()
        );

        let local_map: HashMap<String, LocalFriend> = local_friends
            .into_iter()
            .map(|f| (f.friend_user_id.clone(), f))
            .collect();
        let server_map: HashMap<String, LocalFriend> = server_friends
            .into_iter()
            .map(|f| (f.friend_user_id.clone(), f))
            .collect();

        let mut insert_count = 0;
        let mut update_count = 0;
        let mut delete_count = 0;

        // 插入或更新
        for (id, server_friend) in server_map.iter() {
            if let Some(local_friend) = local_map.get(id) {
                if !Self::friends_equal(local_friend, server_friend) {
                    info!("[FriendSync]   更新好友: {}", id);
                    self.upsert_friend(server_friend).await?;
                    update_count += 1;
                } else {
                    debug!("[FriendSync]   好友 {} 无需更新", id);
                }
            } else {
                info!("[FriendSync]   新增好友: {}", id);
                self.upsert_friend(server_friend).await?;
                insert_count += 1;
            }
        }

        // 删除：当 is_full=true 时，服务器列表视为权威，删除本地多余好友
        if is_full {
            let local_ids: std::collections::HashSet<String> =
                local_map.keys().cloned().collect();
            let server_ids: std::collections::HashSet<String> =
                server_map.keys().cloned().collect();
            for id in local_ids.difference(&server_ids) {
                info!("[FriendSync]   删除本地多余好友: {}", id);
                self.delete_friend(id).await?;
                delete_count += 1;
            }
        }

        // 触发好友变更回调（新增或更新的好友）
        if insert_count > 0 || update_count > 0 {
            let mut changed = Vec::new();
            // 这里使用 server_map 中的值即可（已是最新状态）
            for (id, friend) in server_map.iter() {
                if local_map.get(id).is_none() {
                    // 新增
                    changed.push(friend.clone());
                } else if !Self::friends_equal(local_map.get(id).unwrap(), friend) {
                    // 更新
                    changed.push(friend.clone());
                }
            }

            if !changed.is_empty() {
                if let Ok(json) = serde_json::to_string(&changed) {
                    self.listener.on_friend_list_changed(json).await;
                }
            }
        }

        info!(
            "[FriendSync] 好友同步完成 - 新增: {}, 更新: {}, 删除: {}",
            insert_count, update_count, delete_count
        );
        Ok(())
    }

    /// 比较两个好友是否相等（用于判断是否需要更新）
    fn friends_equal(local: &LocalFriend, server: &LocalFriend) -> bool {
        local.remark == server.remark
            && local.add_source == server.add_source
            && local.operator_user_id == server.operator_user_id
            && local.nickname == server.nickname
            && local.face_url == server.face_url
            && local.ex == server.ex
            && local.attached_info == server.attached_info
            && local.is_pinned == server.is_pinned
    }

    /// 增量同步好友列表
    pub async fn incr_sync_friends(&self) -> Result<()> {
        info!("[FriendSync] 🔄 开始增量同步好友...");

        let version_sync = self.get_version_sync().await?;

        if let Some(ref vs) = version_sync {
            debug!(
                "[FriendSync] 本地好友版本信息 - 版本: {}, 版本ID: {}",
                vs.version, vs.version_id
            );
        } else {
            debug!("[FriendSync] 本地无好友版本信息");
        }

        let local_friends = self.get_all_friends().await?;
        let local_ids = self.get_all_friend_ids().await?;

        // 如果本地没有版本信息，先用全量好友ID列表与本地做一次对比，必要时执行全量同步
        if version_sync.is_none() {
            if let Ok((srv_version, srv_version_id, server_ids)) =
                self.api.get_full_friend_user_ids().await
            {
                let server_set: std::collections::HashSet<String> =
                    server_ids.iter().cloned().collect();
                let local_set: std::collections::HashSet<String> =
                    local_ids.iter().cloned().collect();

                if server_set != local_set {
                    info!(
                        "[FriendSync] 好友ID列表与服务器不一致，执行全量好友同步..."
                    );

                    // 全量拉取好友列表并对齐
                    let server_friends = self.api.get_all_friends().await?;
                    self.sync_friends(server_friends, local_friends, true).await?;

                    // 以 full friend IDs 的版本信息为起点写入 version_sync
                    let new_version_sync = LocalVersionSync {
                        table_name: "local_friends".to_string(),
                        entity_id: self.config.user_id.clone(),
                        version: srv_version,
                        version_id: srv_version_id.clone(),
                    };
                    self.save_version_sync(&new_version_sync).await?;
                    info!(
                        "[FriendSync] 已通过全量好友同步初始化版本信息 - 版本: {}, 版本ID: {}",
                        new_version_sync.version, new_version_sync.version_id
                    );

                    info!("[FriendSync] ✅ 全量好友同步完成");
                    return Ok(());
                } else {
                    debug!("[FriendSync] 好友ID列表与服务器一致，直接使用增量同步");

                    // 如果服务器有合法的版本信息，也可以在这里初始化本地 version_sync
                    if srv_version > 0 && !srv_version_id.is_empty() {
                        let new_version_sync = LocalVersionSync {
                            table_name: "local_friends".to_string(),
                            entity_id: self.config.user_id.clone(),
                            version: srv_version,
                            version_id: srv_version_id.clone(),
                        };
                        self.save_version_sync(&new_version_sync).await?;
                        info!(
                            "[FriendSync] 通过全量ID列表初始化版本信息 - 版本: {}, 版本ID: {}",
                            new_version_sync.version, new_version_sync.version_id
                        );
                    }
                }
            } else {
                debug!(
                    "[FriendSync] 获取全量好友ID列表失败，将直接尝试增量同步"
                );
            }
        }

        // 继续增量同步路径
        let (version, version_id) = if let Some(vs) = version_sync {
            (vs.version, vs.version_id)
        } else {
            (0, "".to_string())
        };

        let resp = match self.api.get_incremental_friends(version, &version_id).await {
            Ok(resp) => resp,
            Err(e) => {
                error!("[FriendSync] 增量好友同步失败: {:?}", e);
                return Err(e);
            }
        };

        // 如果服务器标记 full=true，则以服务器为权威做一次全量对齐
        if resp.full {
            info!("[FriendSync] 服务器要求全量好友同步...");
            let server_friends = self.api.get_all_friends().await?;
            self.sync_friends(server_friends, local_friends, true).await?;

            if !resp.version_id.is_empty() {
                let new_version = if resp.version > 0 {
                    resp.version
                } else {
                    version + 1
                };
                let new_version_sync = LocalVersionSync {
                    table_name: "local_friends".to_string(),
                    entity_id: self.config.user_id.clone(),
                    version: new_version,
                    version_id: resp.version_id.clone(),
                };
                self.save_version_sync(&new_version_sync).await?;
                info!(
                    "[FriendSync] 全量好友同步后更新版本信息 - 版本: {} -> {}, 版本ID: {}",
                    version, new_version_sync.version, new_version_sync.version_id
                );
            }

            info!("[FriendSync] ✅ 全量好友同步完成");
            return Ok(());
        }

        // 处理 insert/update（增量）
        let mut server_friends = Vec::new();
        server_friends.extend(resp.insert.into_iter());
        server_friends.extend(resp.update.into_iter());

        self.sync_friends(server_friends, local_friends, false).await?;

        // 处理删除
        if !resp.delete.is_empty() {
            info!(
                "[FriendSync] 处理删除好友，数量: {}",
                resp.delete.len()
            );
            for id in resp.delete.iter() {
                info!("[FriendSync]   删除好友: {}", id);
                self.delete_friend(id).await?;
            }
        }

        // 更新版本信息
        if !resp.version_id.is_empty() {
            let new_version = if resp.version > 0 {
                resp.version
            } else {
                version + 1
            };
            let new_version_sync = LocalVersionSync {
                table_name: "local_friends".to_string(),
                entity_id: self.config.user_id.clone(),
                version: new_version,
                version_id: resp.version_id.clone(),
            };
            self.save_version_sync(&new_version_sync).await?;
            info!(
                "[FriendSync] 已更新好友版本信息 - 版本: {} -> {}, 版本ID: {}",
                version, new_version_sync.version, new_version_sync.version_id
            );
        }

        info!("[FriendSync] ✅ 增量同步好友完成");

        // 增量好友同步完成后，顺带同步一次黑名单和好友申请列表，触发对应监听器
        if let Ok(blacks) = self.api.get_black_list().await {
            if let Ok(json) = serde_json::to_string(&blacks) {
                self.listener.on_black_list_changed(json).await;
            }
        }

        if let Ok(requests) = self.api.get_friend_requests().await {
            if let Ok(json) = serde_json::to_string(&requests) {
                self.listener.on_friend_request_list_changed(json).await;
            }
        }

        Ok(())
    }
}

