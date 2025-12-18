-- 初始会话/好友相关表结构迁移（对齐原 DAO 中的建表 SQL）

-- 会话表
CREATE TABLE IF NOT EXISTS local_conversations (
    conversation_id TEXT PRIMARY KEY,
    conversation_type INTEGER NOT NULL,
    user_id TEXT NOT NULL DEFAULT '',
    group_id TEXT NOT NULL DEFAULT '',
    show_name TEXT NOT NULL DEFAULT '',
    face_url TEXT NOT NULL DEFAULT '',
    latest_msg TEXT NOT NULL DEFAULT '',
    latest_msg_send_time INTEGER NOT NULL DEFAULT 0,
    unread_count INTEGER NOT NULL DEFAULT 0,
    recv_msg_opt INTEGER NOT NULL DEFAULT 0,
    is_pinned INTEGER NOT NULL DEFAULT 0,
    is_private_chat INTEGER NOT NULL DEFAULT 0,
    burn_duration INTEGER NOT NULL DEFAULT 0,
    group_at_type INTEGER NOT NULL DEFAULT 0,
    is_not_in_group INTEGER NOT NULL DEFAULT 0,
    update_unread_count_time INTEGER NOT NULL DEFAULT 0,
    attached_info TEXT NOT NULL DEFAULT '',
    ex TEXT NOT NULL DEFAULT '',
    draft_text TEXT NOT NULL DEFAULT '',
    draft_text_time INTEGER NOT NULL DEFAULT 0,
    max_seq INTEGER NOT NULL DEFAULT 0,
    min_seq INTEGER NOT NULL DEFAULT 0,
    is_msg_destruct INTEGER NOT NULL DEFAULT 0,
    msg_destruct_time INTEGER NOT NULL DEFAULT 0
);

-- 版本同步表
CREATE TABLE IF NOT EXISTS local_version_sync (
    table_name TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0,
    version_id TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (table_name, entity_id)
);

-- 好友表
CREATE TABLE IF NOT EXISTS local_friends (
    owner_user_id TEXT NOT NULL,
    friend_user_id TEXT NOT NULL,
    remark TEXT NOT NULL DEFAULT '',
    create_time INTEGER NOT NULL DEFAULT 0,
    add_source INTEGER NOT NULL DEFAULT 0,
    operator_user_id TEXT NOT NULL DEFAULT '',
    nickname TEXT NOT NULL DEFAULT '',
    face_url TEXT NOT NULL DEFAULT '',
    ex TEXT NOT NULL DEFAULT '',
    attached_info TEXT NOT NULL DEFAULT '',
    is_pinned INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (owner_user_id, friend_user_id)
);


