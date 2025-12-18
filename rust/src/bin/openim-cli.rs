//! OpenIM CLI 客户端（测试版）
//!
//! 非交互式 CLI，用于测试和展示 IM 功能
//! 启动时通过命令行参数指定用户，自动登录连接，只展示接收到的信息

use anyhow::Result;
use clap::Parser;
use openim_sdk_core_rust::im::client::{ClientConfig, OpenIMClient};
use openim_sdk_core_rust::im::conversation::listener::ConversationListener;
use openim_sdk_core_rust::im::friend::FriendListener;
use openim_sdk_core_rust::im::message::listener::AdvancedMsgListener;
use openim_sdk_core_rust::login_async;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tracing::{error, info};

/// OpenIM CLI 客户端
#[derive(Parser, Debug)]
#[command(name = "openim-cli")]
#[command(about = "OpenIM CLI 客户端 - 用于测试和展示 IM 功能", long_about = None)]
struct Args {
    /// 手机号（默认: 17764338283）
    #[arg(short, long, default_value = "17764338283")]
    phone: String,

    /// 运行时长（秒），0 表示持续运行
    #[arg(short, long, default_value = "0")]
    duration: u64,

    /// 日志级别（默认: info,openim_sdk_core_rust=debug）
    #[arg(long, default_value = "info,openim_sdk_core_rust=debug")]
    log_level: String,
}

/// 初始化日志（同时输出到 stdout 和文件）
fn init_logger(log_level: &str) {
    use std::fs::OpenOptions;
    use std::io;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::EnvFilter;

    // 优先使用环境变量 RUST_LOG（如果设置了），否则使用命令行参数
    let filter_layer =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    // 创建日志文件（追加模式）
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("debug.log")
        .expect("无法创建日志文件 debug.log");

    // 输出到 stdout（控制台），保留 ANSI 颜色代码用于终端显示
    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_writer(io::stdout)
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_ansi(true);

    // 输出到文件，禁用 ANSI 颜色代码（文件不需要颜色）
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(log_file)
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_ansi(false);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(stdout_layer)
        .with(file_layer)
        .init();

    info!("[CLI] 📝 日志已同时输出到控制台和文件: debug.log");
}

/// 设置监听器（输出所有接收到的信息）
fn setup_listeners(client: &mut OpenIMClient) {
    // 会话监听器
    struct CliConversationListener;
    #[async_trait::async_trait]
    impl ConversationListener for CliConversationListener {
        async fn on_sync_server_start(&self, reinstalled: bool) {
            info!(
                "[CLI/Conversation] 🔄 同步开始: reinstalled={}",
                reinstalled
            );
        }

        async fn on_sync_server_finish(&self, reinstalled: bool) {
            info!(
                "[CLI/Conversation] ✅ 同步完成: reinstalled={}",
                reinstalled
            );
        }

        async fn on_sync_server_progress(&self, progress: i32) {
            info!("[CLI/Conversation] 📊 同步进度: {}%", progress);
        }

        async fn on_sync_server_failed(&self, reinstalled: bool) {
            error!(
                "[CLI/Conversation] ❌ 同步失败: reinstalled={}",
                reinstalled
            );
        }

        async fn on_new_conversation(&self, conversation_list: String) {
            info!("[CLI/Conversation] 🆕 新会话: {}", conversation_list);
        }

        async fn on_conversation_changed(&self, conversation_list: String) {
            info!("[CLI/Conversation] 🔄 会话变更: {}", conversation_list);
        }

        async fn on_total_unread_message_count_changed(&self, total_unread_count: i32) {
            info!("[CLI/Conversation] 📬 总未读数: {}", total_unread_count);
        }

        async fn on_conversation_user_input_status_changed(&self, change: String) {
            info!("[CLI/Conversation] ⌨️ 输入状态: {}", change);
        }
    }
    client.set_conversation_listener(Arc::new(CliConversationListener));

    // 好友监听器
    struct CliFriendListener;
    #[async_trait::async_trait]
    impl FriendListener for CliFriendListener {
        async fn on_friend_list_changed(&self, friends_json: String) {
            info!("[CLI/Friend] 👥 好友列表变更: {}", friends_json);
        }

        async fn on_black_list_changed(&self, blacks_json: String) {
            info!("[CLI/Friend] 🚫 黑名单变更: {}", blacks_json);
        }

        async fn on_friend_request_list_changed(&self, requests_json: String) {
            info!("[CLI/Friend] 📝 好友申请变更: {}", requests_json);
        }
    }
    client.set_friend_listener(Arc::new(CliFriendListener));

    // 消息监听器
    struct CliAdvancedMsgListener;
    #[async_trait::async_trait]
    impl AdvancedMsgListener for CliAdvancedMsgListener {
        async fn on_recv_new_message(&self, message: String) {
            info!("[CLI/Message] 📨 收到新消息: {}", message);
        }

        async fn on_recv_c2c_read_receipt(&self, msg_receipt_list: String) {
            info!("[CLI/Message] 📖 已读回执: {}", msg_receipt_list);
        }

        async fn on_new_recv_message_revoked(&self, message_revoked: String) {
            info!("[CLI/Message] 🗑️ 消息撤回: {}", message_revoked);
        }

        async fn on_recv_offline_new_message(&self, message: String) {
            info!("[CLI/Message] 📬 离线消息: {}", message);
        }

        async fn on_msg_deleted(&self, message: String) {
            info!("[CLI/Message] 🗑️ 消息删除: {}", message);
        }

        async fn on_recv_online_only_message(&self, message: String) {
            info!("[CLI/Message] 💬 在线消息: {}", message);
        }

        async fn on_kicked_offline(&self) {
            error!("[CLI/Message] ⚠️ 被踢下线");
        }

        async fn on_connection_status_changed(&self, connected: bool, message: String) {
            if connected {
                info!("[CLI/Message] 🔗 已连接: {}", message);
            } else {
                error!("[CLI/Message] 🔗 断开连接: {}", message);
            }
        }

        async fn on_recv_typing_status(&self, typing_info: String) {
            info!("[CLI/Message] ⌨️ 输入状态: {}", typing_info);
        }
    }
    client.set_advanced_msg_listener(Arc::new(CliAdvancedMsgListener));
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // 初始化日志
    init_logger(&args.log_level);

    info!("[CLI] 🚀 OpenIM CLI 客户端（测试模式）");
    info!("[CLI] 📱 手机号: {}", args.phone);
    info!("[CLI] ⏱️  运行时长: {} 秒（0=持续运行）", args.duration);

    // 登录
    info!("[CLI] 🔐 正在登录...");
    let area_code = "+86".to_string();
    let password = "284f3d09ea0695538e4ded1c1766d73a".to_string(); // 测试密码
    let platform = 5;

    let token_info = login_async(area_code, args.phone.clone(), password, platform)
        .await
        .map_err(|e| anyhow::anyhow!("登录失败: {}", e))?;

    let (user_id, im_token) = if let Some(data) = &token_info.data {
        (data.user_id.clone(), data.im_token.clone())
    } else {
        return Err(anyhow::anyhow!("登录失败：服务器返回数据为空"));
    };

    info!("[CLI] ✅ 登录成功！用户ID: {}", user_id);

    // 创建客户端
    let config = ClientConfig::new(user_id.clone(), im_token, platform);
    let mut client = OpenIMClient::new(config);

    // 设置监听器
    setup_listeners(&mut client);

    let client = Arc::new(Mutex::new(client));

    // 连接
    info!("[CLI] 🔗 正在连接服务器...");
    {
        let mut client_guard = client.lock().await;
        client_guard
            .connect()
            .await
            .map_err(|e| anyhow::anyhow!("连接失败: {}", e))?;
    }
    info!("[CLI] ✅ 连接成功！");

    // 显示初始信息
    {
        let client_guard = client.lock().await;
        if let Ok(conversations) = client_guard.get_all_conversations().await {
            info!("[CLI] 📋 会话列表（共 {} 个）:", conversations.len());
            for conv in conversations.iter().take(5) {
                info!(
                    "[CLI]   - {} | 未读: {} | 最新: {}",
                    conv.show_name,
                    conv.unread_count,
                    if conv.latest_msg.len() > 30 {
                        &conv.latest_msg[..30]
                    } else {
                        &conv.latest_msg
                    }
                );
            }
        }

        if let Ok(friends) = client_guard.get_all_friends().await {
            info!("[CLI] 👥 好友列表（共 {} 个）", friends.len());
        }

        if let Ok(unread) = client_guard.get_total_unread_count().await {
            info!("[CLI] 📬 总未读数: {}", unread);
        }
    }

    info!("[CLI] 📥 开始监听消息...");
    info!("[CLI] 💡 提示：程序将持续运行并显示接收到的所有消息和事件");
    if args.duration > 0 {
        info!("[CLI] ⏰ {} 秒后自动退出", args.duration);
        sleep(Duration::from_secs(args.duration)).await;
        info!("[CLI] 👋 程序退出");
    } else {
        info!("[CLI] ⏰ 持续运行中，按 Ctrl+C 退出");
        // 持续运行直到被中断
        loop {
            sleep(Duration::from_secs(3600)).await;
        }
    }

    Ok(())
}
