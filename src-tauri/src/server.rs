use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;

use crate::{config, plugins, router};

pub struct ServerState {
    pub cancel_token: Option<CancellationToken>,
    pub app_config: config::AppConfig,
    /// 插件配置（运行时内存副本）
    pub plugins_config: plugins::PluginsConfig,
}

/// 🔁 重启 HTTP 服务（停止后再启动，配置变更后自动调用）
///
/// 在后台线程中执行异步等待，避免阻塞 tokio 运行时线程（被 async 命令调用时）
pub fn restart_server(state: &Arc<Mutex<ServerState>>) -> Result<u16, String> {
    // 先停止
    let port = {
        let mut guard = state.lock().map_err(|e| e.to_string())?;
        if let Some(token) = &guard.cancel_token {
            token.cancel();
            guard.cancel_token = None;
            println!("🛑 [restart] HTTP 服务已停止");
        }
        guard.app_config.port
    };
    // 短暂等待确保端口释放
    std::thread::sleep(std::time::Duration::from_millis(500));

    // 在后台线程中启动，不阻塞 tokio 运行时
    let state_clone = Arc::clone(state);
    std::thread::spawn(move || {
        if let Err(e) = do_spawn_server(&state_clone, "[reload] ") {
            eprintln!("⚠️ [reload] 后台重启失败: {}", e);
        }
    });

    // 直接返回端口号（实际绑定在后台进行）
    Ok(port)
}

/// 启动 HTTP 服务（被自动启动 / 手动启动复用）
pub fn do_spawn_server(
    state: &Arc<Mutex<ServerState>>,
    log_prefix: &'static str,
) -> Result<u16, String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;

    // 重复运行检查
    if guard.cancel_token.is_some() {
        return Err(format!("{log_prefix}HTTP 服务已在运行"));
    }

    // 取出配置（必须在 drop(guard) 之前克隆）
    let app_config = guard.app_config.clone();
    let plugins_config = guard.plugins_config.clone();
    let port = app_config.port;

    // 创建取消令牌并存入状态
    let cancel_token = CancellationToken::new();
    let token_clone = cancel_token.clone();
    guard.cancel_token = Some(cancel_token);
    drop(guard);

    let addr = format!("0.0.0.0:{}", port);
    // 使用 std::sync::mpsc，可在任何线程上下文安全使用
    let (bind_tx, bind_rx) = std::sync::mpsc::channel::<Result<(), String>>();

    // 启动异步 HTTP 服务
    tauri::async_runtime::spawn(async move {
        let public_folder = app_config.public_folder.clone();
        let enable_upload = app_config.enable_upload;
        let plugins_for_router = plugins_config;
        let version_str = env!("CARGO_PKG_VERSION").to_string();
        // 从 config 解析最终插件目录
        let plugins_folder = crate::paths::resolve_plugins_dir(
            app_config.plugins_folder.as_deref()
        ).unwrap_or_else(|_| std::path::PathBuf::from("./plugins"));
        let app = router::create_router(
            public_folder, enable_upload, version_str, port,
            plugins_for_router, plugins_folder
        );

        let listener = match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => l,
            Err(e) => {
                let msg = format!("无法绑定端口 {}: {}", port, e);
                eprintln!("❌ {log_prefix}{}", msg);
                let _ = bind_tx.send(Err(msg));
                return;
            }
        };
        println!("🚀 {log_prefix}HTTP 服务器启动成功: http://{}", addr);
        let _ = bind_tx.send(Ok(()));

        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                token_clone.cancelled().await;
            })
            .await
            .ok();
        println!("🛑 {log_prefix}HTTP 服务器已停止");
    });

    // 等待绑定结果（阻塞当前线程，仅在 setup 阶段调用，不阻塞 tokio 运行时）
    match bind_rx.recv() {
        Ok(Ok(())) => Ok(port),
        Ok(Err(msg)) => Err(msg),
        Err(_) => Err("HTTP 服务启动任务异常退出".to_string()),
    }
}

/// setup 阶段自动启动 HTTP 服务（不依赖前端 WebView）
pub fn auto_start_server_if_needed(state: &Arc<Mutex<ServerState>>) {
    match do_spawn_server(state, "[auto_start] ") {
        Ok(_port) => { /* 已在 do_spawn_server 中打日志，这里不用处理 */ }
        Err(msg) => {
            if msg.contains("已在运行") {
                println!("ℹ️  {msg}，跳过启动");
            } else {
                eprintln!("❌ {msg}");
            }
        }
    }
}

/// 获取本机所有 IPv4 地址（排除回环地址）
pub fn get_local_ips() -> Vec<String> {
    let mut ips: Vec<String> = Vec::new();
    ips.push("127.0.0.1".to_string());
    if let Ok(ifaces) = get_if_addrs::get_if_addrs() {
        for iface in ifaces {
            if let get_if_addrs::IfAddr::V4(v4) = iface.addr {
                let ip = v4.ip;
                if !ip.is_loopback() && !ip.is_unspecified() {
                    ips.push(ip.to_string());
                }
            }
        }
    }
    ips
}