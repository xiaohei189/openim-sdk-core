//! 会话模块的实体定义（已迁移到 sqlx）
//!
//! 之前这里使用 SeaORM 的实体宏，现在会话相关表结构完全由
//! `crate::im::conversation::dao` 中的 SQL（基于 sqlx）维护。
//! 若后续需要在此处定义结构体，可直接使用普通 Rust 结构体而不依赖 SeaORM。
