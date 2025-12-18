//! 好友模块的实体定义（已迁移到 sqlx）
//!
//! 原本这里使用 SeaORM 的实体宏，现在好友相关表结构全部由
//! `crate::im::friend::dao` 中的 SQL（基于 sqlx）维护。
