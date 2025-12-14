//! 消息数据访问层（DAO）
//!
//! 负责所有消息相关的数据库操作，将数据访问逻辑与业务逻辑分离

use crate::im::message::models::LocalChatLog;
use anyhow::Result;
use chrono::Utc;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Row, Sqlite};

/// 本地消息存储（使用 sqlx / SQLite，仿 Go 版按会话建表）
///
/// Go 版会为每个会话动态建表；SeaORM 无法动态建表，因此这里用原生 SQLx
/// 在运行时创建/访问按会话命名的表（msg_<conversation_id_sanitized>）。
pub struct MessageStore {
    pool: Pool<Sqlite>,
    /// 当前登录用户，用于过滤自发消息的已读逻辑
    pub login_user_id: String,
}

impl MessageStore {
    pub async fn new(db_url: &str, login_user_id: String) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await?;
        let store = Self {
            pool,
            login_user_id,
        };
        Ok(store)
    }

    /// 将会话 ID 转为表名（去掉非法字符，前缀 msg_）
    fn table_name(&self, conversation_id: &str) -> String {
        let sanitized: String = conversation_id
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();
        format!("msg_{}", sanitized)
    }

    /// 确保表存在，仿 Go 版 schema
    async fn ensure_table(&self, conversation_id: &str) -> Result<String> {
        let table = self.table_name(conversation_id);
        let sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {table} (
                client_msg_id         TEXT PRIMARY KEY,
                server_msg_id         TEXT,
                send_id               TEXT,
                recv_id               TEXT,
                sender_platform_id    INTEGER,
                sender_nickname       TEXT,
                sender_face_url       TEXT,
                session_type          INTEGER,
                msg_from              INTEGER,
                content_type          INTEGER,
                content               TEXT,
                is_read               INTEGER DEFAULT 0,
                status                INTEGER,
                seq                   INTEGER DEFAULT 0,
                send_time             INTEGER,
                create_time           INTEGER,
                attached_info         TEXT,
                ex                    TEXT,
                local_ex              TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_{table}_seq ON {table}(seq);
            CREATE INDEX IF NOT EXISTS idx_{table}_send_time ON {table}(send_time);
            CREATE INDEX IF NOT EXISTS idx_{table}_content_type ON {table}(content_type);
            "#,
            table = table
        );
        sqlx::query(&sql).execute(&self.pool).await?;
        Ok(table)
    }

    fn placeholders(n: usize) -> String {
        if n == 0 {
            String::new()
        } else {
            vec!["?"; n].join(",")
        }
    }

    pub async fn insert_message(&self, msg: &LocalChatLog) -> Result<()> {
        let table = self.ensure_table(&msg.conversation_id).await?;
        let sql = r#"
        INSERT OR REPLACE INTO {table} (
            client_msg_id, server_msg_id, send_id, recv_id, sender_platform_id,
            sender_nickname, sender_face_url, session_type, msg_from, content_type, content,
            is_read, status, seq, send_time, create_time, attached_info, ex, local_ex, group_id
        ) VALUES (
            ?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?
        );
        "#;
        let sql = sql.replace("{table}", &table);
        sqlx::query(&sql)
            .bind(&msg.client_msg_id)
            .bind(&msg.server_msg_id)
            .bind(&msg.send_id)
            .bind(&msg.recv_id)
            .bind(msg.sender_platform_id)
            .bind(&msg.sender_nickname)
            .bind(&msg.sender_face_url)
            .bind(msg.session_type)
            .bind(msg.msg_from)
            .bind(msg.content_type)
            .bind(&msg.content)
            .bind(if msg.is_read { 1 } else { 0 })
            .bind(msg.status)
            .bind(msg.seq)
            .bind(msg.send_time)
            .bind(msg.create_time)
            .bind(&msg.attached_info)
            .bind(&msg.ex)
            .bind(&msg.local_ex)
            .bind(&msg.group_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_by_client_msg_id(
        &self,
        conversation_id: &str,
        client_msg_id: &str,
    ) -> Result<Option<LocalChatLog>> {
        let table = self.ensure_table(conversation_id).await?;
        let sql = format!(
            r#"
        SELECT * FROM {table}
        WHERE client_msg_id = ?
        LIMIT 1;
        "#,
            table = table
        );
        let row = sqlx::query(&sql)
            .bind(client_msg_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(Self::row_to_log))
    }

    pub async fn delete_by_client_msg_id(
        &self,
        conversation_id: &str,
        client_msg_id: &str,
    ) -> Result<()> {
        let table = self.ensure_table(conversation_id).await?;
        let sql = format!(
            "DELETE FROM {table} WHERE client_msg_id = ?;",
            table = table
        );
        sqlx::query(&sql)
            .bind(client_msg_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_conversation(&self, conversation_id: &str) -> Result<()> {
        let table = self.ensure_table(conversation_id).await?;
        let sql = format!("DROP TABLE IF EXISTS {table};", table = table);
        sqlx::query(&sql).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn mark_as_read_by_msg_ids(
        &self,
        conversation_id: &str,
        msg_ids: &[String],
    ) -> Result<i64> {
        if msg_ids.is_empty() {
            return Ok(0);
        }
        let table = self.ensure_table(conversation_id).await?;
        let placeholders = Self::placeholders(msg_ids.len());
        let sql = format!(
            "UPDATE {table} SET is_read = 1 WHERE client_msg_id IN ({}) AND send_id != ?",
            placeholders,
            table = table
        );
        let mut query = sqlx::query(&sql);
        for id in msg_ids {
            query = query.bind(id);
        }
        query = query.bind(self.login_user_id.clone());
        let res = query.execute(&self.pool).await?;
        Ok(res.rows_affected() as i64)
    }

    pub async fn mark_as_read_by_seqs(&self, conversation_id: &str, seqs: &[i64]) -> Result<i64> {
        if seqs.is_empty() {
            return Ok(0);
        }
        let table = self.ensure_table(conversation_id).await?;
        let placeholders = Self::placeholders(seqs.len());
        let sql = format!(
            "UPDATE {table} SET is_read = 1 WHERE seq IN ({}) AND send_id != ?",
            placeholders,
            table = table
        );
        let mut query = sqlx::query(&sql);
        for s in seqs {
            query = query.bind(s);
        }
        query = query.bind(self.login_user_id.clone());
        let res = query.execute(&self.pool).await?;
        Ok(res.rows_affected() as i64)
    }

    pub async fn get_unread_by_conversation(
        &self,
        conversation_id: &str,
    ) -> Result<Vec<LocalChatLog>> {
        let table = self.ensure_table(conversation_id).await?;
        let sql = format!(
            r#"
        SELECT * FROM {table}
        WHERE is_read = 0 AND send_id != ?
        ORDER BY send_time DESC;
        "#,
            table = table
        );
        let rows = sqlx::query(&sql)
            .bind(&self.login_user_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(Self::row_to_log).collect())
    }

    pub async fn get_messages_by_seq(
        &self,
        conversation_id: &str,
        seqs: &[i64],
    ) -> Result<Vec<LocalChatLog>> {
        if seqs.is_empty() {
            return Ok(vec![]);
        }
        let table = self.ensure_table(conversation_id).await?;
        let placeholders = Self::placeholders(seqs.len());
        let sql = format!(
            "SELECT * FROM {table} WHERE seq IN ({}) ORDER BY send_time DESC",
            placeholders,
            table = table
        );
        let mut query = sqlx::query(&sql);
        for s in seqs {
            query = query.bind(s);
        }
        let rows = query.fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(Self::row_to_log).collect())
    }

    pub async fn get_messages_by_client_msg_ids(
        &self,
        conversation_id: &str,
        ids: &[String],
    ) -> Result<Vec<LocalChatLog>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let table = self.ensure_table(conversation_id).await?;
        let placeholders = Self::placeholders(ids.len());
        let sql = format!(
            "SELECT * FROM {table} WHERE client_msg_id IN ({}) ORDER BY send_time DESC",
            placeholders,
            table = table
        );
        let mut query = sqlx::query(&sql);
        for id in ids {
            query = query.bind(id);
        }
        let rows = query.fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(Self::row_to_log).collect())
    }

    pub async fn max_seq(&self, conversation_id: &str) -> Result<i64> {
        let table = self.ensure_table(conversation_id).await?;
        let sql = format!(
            r#"SELECT IFNULL(MAX(seq),0) as max_seq FROM {table}"#,
            table = table
        );
        let row = sqlx::query(&sql).fetch_one(&self.pool).await?;
        Ok(row.try_get::<i64, _>("max_seq")?)
    }

    pub async fn peer_max_seq(&self, conversation_id: &str) -> Result<i64> {
        let table = self.ensure_table(conversation_id).await?;
        let sql = format!(
            r#"SELECT IFNULL(MAX(seq),0) as max_seq FROM {table} WHERE send_id != ?"#,
            table = table
        );
        let row = sqlx::query(&sql)
            .bind(&self.login_user_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.try_get::<i64, _>("max_seq")?)
    }

    pub async fn update_local_ex(
        &self,
        conversation_id: &str,
        client_msg_id: &str,
        local_ex: &str,
    ) -> Result<u64> {
        let table = self.ensure_table(conversation_id).await?;
        let sql = format!(
            r#"UPDATE {table} SET local_ex = ? WHERE client_msg_id = ?"#,
            table = table
        );
        let res = sqlx::query(&sql)
            .bind(local_ex)
            .bind(client_msg_id)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected())
    }

    pub async fn search_local_messages(
        &self,
        conversation_id: Option<&str>,
        keyword: Option<&str>,
        content_types: Option<&[i32]>,
        send_time_begin: Option<i64>,
        send_time_end: Option<i64>,
    ) -> Result<Vec<LocalChatLog>> {
        let conversation_id = conversation_id.ok_or_else(|| {
            anyhow::anyhow!("search_local_messages 需要指定 conversation_id（按会话分表）")
        })?;
        let table = self.ensure_table(conversation_id).await?;
        let mut clauses = Vec::new();
        enum Bind {
            Str(String),
            I64(i64),
            I32(i32),
        }
        let mut binds: Vec<Bind> = Vec::new();

        clauses.push("1=1".to_string()); // 起始占位
        if let Some(kw) = keyword {
            clauses.push("content LIKE ?".to_string());
            binds.push(Bind::Str(format!("%{}%", kw)));
        }
        if let Some(cts) = content_types {
            if !cts.is_empty() {
                let placeholders = Self::placeholders(cts.len());
                // 需持有字符串，避免临时字符串悬垂
                let cond = format!("content_type IN ({})", placeholders);
                clauses.push(cond);
                for ct in cts {
                    binds.push(Bind::I32(*ct));
                }
            }
        }
        if let Some(start) = send_time_begin {
            clauses.push("send_time >= ?".to_string());
            binds.push(Bind::I64(start));
        }
        if let Some(end) = send_time_end {
            clauses.push("send_time <= ?".to_string());
            binds.push(Bind::I64(end));
        }

        let where_sql = if clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", clauses.join(" AND "))
        };

        let sql = format!(
            "SELECT * FROM {table} {where_sql} ORDER BY send_time DESC LIMIT 200",
            table = table,
            where_sql = where_sql
        );

        let mut query = sqlx::query(&sql);
        for val in binds {
            match val {
                Bind::Str(s) => query = query.bind(s),
                Bind::I64(i) => query = query.bind(i),
                Bind::I32(i) => query = query.bind(i),
            }
        }

        let rows = query.fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(Self::row_to_log).collect())
    }

    fn row_to_log(row: sqlx::sqlite::SqliteRow) -> LocalChatLog {
        LocalChatLog {
            conversation_id: row
                .try_get::<String, _>("conversation_id")
                .unwrap_or_default(),
            client_msg_id: row
                .try_get::<String, _>("client_msg_id")
                .unwrap_or_default(),
            server_msg_id: row
                .try_get::<String, _>("server_msg_id")
                .unwrap_or_default(),
            send_id: row.try_get::<String, _>("send_id").unwrap_or_default(),
            recv_id: row.try_get::<String, _>("recv_id").unwrap_or_default(),
            sender_platform_id: row
                .try_get::<i32, _>("sender_platform_id")
                .unwrap_or_default(),
            sender_nickname: row
                .try_get::<String, _>("sender_nickname")
                .unwrap_or_default(),
            sender_face_url: row
                .try_get::<String, _>("sender_face_url")
                .unwrap_or_default(),
            session_type: row.try_get::<i32, _>("session_type").unwrap_or_default(),
            msg_from: row.try_get::<i32, _>("msg_from").unwrap_or_default(),
            content_type: row.try_get::<i32, _>("content_type").unwrap_or_default(),
            content: row.try_get::<String, _>("content").unwrap_or_default(),
            is_read: row.try_get::<i32, _>("is_read").unwrap_or_default() != 0,
            status: row.try_get::<i32, _>("status").unwrap_or_default(),
            seq: row.try_get::<i64, _>("seq").unwrap_or_default(),
            send_time: row.try_get::<i64, _>("send_time").unwrap_or_default(),
            create_time: row
                .try_get::<i64, _>("create_time")
                .unwrap_or_else(|_| Utc::now().timestamp_millis()),
            attached_info: row
                .try_get::<String, _>("attached_info")
                .unwrap_or_default(),
            ex: row.try_get::<String, _>("ex").unwrap_or_default(),
            local_ex: row.try_get::<String, _>("local_ex").unwrap_or_default(),
            group_id: String::new(),
        }
    }
}
