mod config;
mod plugins;
mod router;

use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;
use tauri_plugin_autostart::MacosLauncher;
use tauri::{tray::TrayIconBuilder, menu::Menu, AppHandle, Manager};

/// 🔁 重启 HTTP 服务（停止后再启动，配置变更后自动调用）
fn restart_server(state: &Arc<Mutex<ServerState>>) -> Result<u16, String> {
    // 先停止
    {
        let mut guard = state.lock().map_err(|e| e.to_string())?;
        if let Some(token) = &guard.cancel_token {
            token.cancel();
            guard.cancel_token = None;
            println!("🛑 [restart] HTTP 服务已停止");
        }
    }
    // 短暂等待确保端口释放
    std::thread::sleep(std::time::Duration::from_millis(200));
    // 再启动
    do_spawn_server(state, "[reload] ")
}


struct ServerState {
    cancel_token: Option<CancellationToken>,
    app_config: config::AppConfig,
    /// 插件配置（运行时内存副本）
    plugins_config: plugins::PluginsConfig,
}

/// 🔧 Linux WebKitGTK 渲染修复（Ubuntu/GNOME 下莫名线条、色块、边框残缺）
/// 必须在 Tauri Builder 之前设置，WebKitGTK 初始化后就不能改了
fn apply_linux_rendering_fixes() {
    #[cfg(target_os = "linux")]
    {
        // ① 禁用 WebKit 合成模式，CPU 合成代替 GPU 合成，彻底规避 OpenGL 伪影
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        // ② 禁用 DMA-BUF 渲染器，解决 NVIDIA/AMD 驱动下的色块问题
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        // ③ 使用 Cairo 2D 渲染（禁用 OpenGL ES），抗锯齿更稳定
        std::env::set_var("GSK_RENDERER", "cairo");
        // ④ 禁止 WebKit 把页面渲染到离屏 GL 纹理（导致半透明边框缺失）
        std::env::set_var("WEBKIT_FORCE_DISK_CACHE", "0");

        println!("🐧 [Linux 渲染修复] 已应用 WebKitGTK 渲染兼容性设置");
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (); // 非 Linux 平台什么都不做
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // ⚠️ 务必放在所有 Tauri/WebKit 初始化之前！
    apply_linux_rendering_fixes();

    let app_config = config::load_config(None).expect("加载配置失败");
    config::validate_config(&app_config).expect("配置验证失败");
    let plugins_config = plugins::load_plugins_config(None).expect("加载插件配置失败");
    
    let server_state = Arc::new(Mutex::new(ServerState { 
        cancel_token: None,
        app_config: app_config.clone(),
        plugins_config: plugins_config.clone(),
    }));
    
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--hide-to-tray"]),
        ))
        .plugin(tauri_plugin_dialog::init())
        .manage(server_state.clone())
        .setup(|app| {
            setup_system_tray(app)?;

            // Rust 后端自动启动 HTTP 服务（不依赖前端 WebView，开机自启时可靠）
            let state = app.state::<Arc<Mutex<ServerState>>>();
            auto_start_server_if_needed(&state);

            let args: Vec<String> = std::env::args().collect();
            if args.contains(&"--hide-to-tray".to_string()) {
                // macOS 用 minimize() 代替 hide()，确保窗口可恢复
                hide_to_tray(app.handle());
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_server, 
            stop_server,
            show_window,
            hide_window,
            quit_app,
            get_config,
            save_config,
            get_server_status,
            get_version,
            choose_folder,
            // ✅ 插件配置管理
            get_plugins_config,
            save_plugin_extension_status,
            activate_plugin_handler,
            add_custom_plugin,
            get_plugins_dir,
            open_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 初始化系统托盘
fn setup_system_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let show_menu_item = &tauri::menu::MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
    let hide_menu_item = &tauri::menu::MenuItem::with_id(app, "hide", "隐藏到托盘", true, None::<&str>)?;
    let separator = &tauri::menu::PredefinedMenuItem::separator(app)?;
    let quit_menu_item = &tauri::menu::MenuItem::with_id(app, "quit", "退出应用", true, None::<&str>)?;
    
    let menu = Menu::with_items(app, &[show_menu_item, hide_menu_item, separator, quit_menu_item])?;
    
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("Tauri HTTP Server")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                show_main_window(app);
            }
            "hide" => {
                hide_to_tray(app);
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                button_state: tauri::tray::MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                show_main_window(&app);
            }
        })
        .build(app)?;
    
    Ok(())
}

/// 显示主窗口
fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        println!("🪟 [show_main_window] 尝试恢复窗口…");
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        println!("🪟 [show_main_window] 窗口恢复完成");
    }
}

/// 隐藏到托盘
fn hide_to_tray(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        println!("🪟 [hide_to_tray] 隐藏窗口到托盘…");
        // macOS 用 minimize()，其他平台用 hide()（hide() 在 macOS 下可能无法恢复）
        #[cfg(target_os = "macos")]
        {
            let _ = window.minimize();
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = window.hide();
        }
    }
}

/// 启动 HTTP 服务（被自动启动 / 手动启动复用）
/// log_prefix: 日志前缀，如 "[auto_start] " 或 ""
fn do_spawn_server(
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
    let (bind_tx, bind_rx) = std::sync::mpsc::channel::<Result<(), String>>();

    // 启动异步 HTTP 服务
    tauri::async_runtime::spawn(async move {
        let public_folder = app_config.public_folder.clone();
        let enable_upload = app_config.enable_upload;
        let plugins_for_router = plugins_config;
        let version_str = env!("CARGO_PKG_VERSION").to_string();
        let app = router::create_router(public_folder, enable_upload, version_str, port, plugins_for_router);

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

    // 等待绑定结果（阻塞当前线程），把错误传递给调用方
    match bind_rx.recv() {
        Ok(Ok(())) => Ok(port),
        Ok(Err(msg)) => Err(msg),
        Err(_) => Err("HTTP 服务启动任务异常退出".to_string()),
    }
}

/// setup 阶段自动启动 HTTP 服务（不依赖前端 WebView）
fn auto_start_server_if_needed(state: &Arc<Mutex<ServerState>>) {
    match do_spawn_server(state, "[auto_start] ") {
        Ok(_port) => { /* 已在 do_spawn_server 中打日志，这里不用处理 */ }
        Err(msg) => {
            // "已在运行" 不算错误，只打 info；其他情况打 error
            if msg.contains("已在运行") {
                println!("ℹ️  {msg}，跳过启动");
            } else {
                eprintln!("❌ {msg}");
            }
        }
    }
}

#[tauri::command]
fn start_server(state: tauri::State<'_, Arc<Mutex<ServerState>>>) -> Result<String, String> {
    let port = do_spawn_server(state.inner(), "")?;
    Ok(format!("HTTP 服务已启动，访问 http://127.0.0.1:{}", port))
}

#[tauri::command]
fn stop_server(state: tauri::State<Arc<Mutex<ServerState>>>) -> Result<String, String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    
    match &state.cancel_token {
        Some(token) => {
            token.cancel();
            state.cancel_token = None;
            Ok("HTTP 服务已停止".to_string())
        }
        None => Err("HTTP 服务未运行".to_string()),
    }
}

#[tauri::command]
fn show_window(app: tauri::AppHandle) -> Result<(), String> {
    show_main_window(&app);
    Ok(())
}

#[tauri::command]
fn hide_window(app: tauri::AppHandle) -> Result<(), String> {
    hide_to_tray(&app);
    Ok(())
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

/// 获取当前配置
#[tauri::command]
fn get_config(state: tauri::State<'_, Arc<Mutex<ServerState>>>) -> Result<serde_json::Value, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "port": state.app_config.port,
        "publicFolder": state.app_config.public_folder.to_string_lossy().to_string(),
        "enableUpload": state.app_config.enable_upload,
    }))
}

/// 保存配置到 config.json
#[tauri::command]
fn save_config(
    state: tauri::State<'_, Arc<Mutex<ServerState>>>,
    port: u16,
    public_folder: String,
    enable_upload: bool,
) -> Result<String, String> {
    let arc_state = Arc::clone(state.inner());
    if port == 0 {
        return Err("端口号不能为 0".to_string());
    }
    if public_folder.trim().is_empty() {
        return Err("指定文件目录不能为空".to_string());
    }

    let path = config::get_default_config_path().map_err(|e| e.to_string())?;
    let config_dir = path.parent()
        .ok_or_else(|| "无法获取配置目录".to_string())?.to_path_buf();

    // 相对路径转绝对路径
    let public_path = std::path::PathBuf::from(&public_folder);
    let abs_public_path = if public_path.is_absolute() {
        public_path.clone()
    } else {
        config_dir.join(&public_path)
    };

    let new_config = config::AppConfig {
        port,
        public_folder: abs_public_path.clone(),
        enable_upload,
    };

    config::save_config_to_path(&new_config, &path).map_err(|e| e.to_string())?;

    // 更新内存配置
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.app_config.port = port;
    state.app_config.public_folder = abs_public_path;
    state.app_config.enable_upload = enable_upload;

    // 服务运行中则重启使配置生效
    let was_running = state.cancel_token.is_some();
    drop(state);

    if was_running {
        if let Err(e) = restart_server(&arc_state) {
            return Ok(format!("__RESTART_FAILED__{}", e));
        }
    }

    Ok("__OK__".to_string())
}

/// 获取本机所有 IPv4 地址（排除回环地址）
fn get_local_ips() -> Vec<String> {
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

/// 获取服务器运行状态
#[tauri::command]
fn get_server_status(state: tauri::State<'_, Arc<Mutex<ServerState>>>) -> Result<serde_json::Value, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    let is_running = state.cancel_token.is_some();
    let urls: Vec<String> = if is_running {
        get_local_ips()
            .iter()
            .map(|ip| format!("http://{}:{}", ip, state.app_config.port))
            .collect()
    } else {
        vec![]
    };
    Ok(serde_json::json!({
        "isRunning": is_running,
        "port": state.app_config.port,
        "publicFolder": state.app_config.public_folder.to_string_lossy().to_string(),
        "enableUpload": state.app_config.enable_upload,
        "urls": urls
    }))
}

/// 获取应用版本号（从 Cargo.toml 的 version 字段读取）
#[tauri::command]
fn get_version() -> String {
    let version = env!("CARGO_PKG_VERSION");
    let tauri_version = tauri::VERSION;
    println!("🔖 版本查询: app={}, tauri={}", version, tauri_version);
    version.to_string()
}

/// 弹出「选择文件夹」对话框
/// 返回：选中的绝对路径（正斜杠分隔）；取消 → None
#[tauri::command]
async fn choose_folder(
    app: tauri::AppHandle,
    initial_dir: Option<String>,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    use std::path::{Path, PathBuf};
    use std::cell::RefCell;

    // oneshot channel 把回调 API 桥接成 async Future
    let (tx, rx) = tokio::sync::oneshot::channel::<Option<PathBuf>>();
    // RefCell 包装 Sender：dialog 回调要求 FnMut，tx 只能用一次
    let tx_cell = RefCell::new(Some(tx));

    let mut builder = app.dialog().file();
    builder = builder.set_title("选择指定文件根目录");

    if let Some(dir) = initial_dir {
        let p = PathBuf::from(&dir);
        if p.exists() {
            builder = builder.set_directory(p);
        }
    }

    // pick_folder 回调中 as_path() 取本地路径
    builder.pick_folder(move |fp_opt| {
        let path_opt: Option<PathBuf> = fp_opt.and_then(|fp| {
            fp.as_path().map(|p: &Path| p.to_path_buf())
        });

        if let Some(tx) = tx_cell.borrow_mut().take() {
            let _ = tx.send(path_opt);
        }
    });

    let result = rx.await.map_err(|e| format!("等待对话框失败: {}", e))?;

    match result {
        Some(pb) => {
            let s = pb.to_string_lossy().replace('\\', "/");
            println!("📁 [choose_folder] 用户选中: {}", s);
            Ok(Some(s))
        }
        None => {
            println!("📁 [choose_folder] 用户取消了选择");
            Ok(None)
        }
    }
}

// 获取完整 plugins.json 内容
#[tauri::command]
fn get_plugins_config(
    state: tauri::State<'_, Arc<Mutex<ServerState>>>,
) -> Result<plugins::PluginsConfig, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    Ok(guard.plugins_config.clone())
}

// 切换某扩展名的状态（enabled/disabled/browser-default/undeveloped）
#[tauri::command]
fn save_plugin_extension_status(
    state: tauri::State<'_, Arc<Mutex<ServerState>>>,
    ext: String,
    status: String,
) -> Result<String, String> {
    use plugins::ExtensionStatus;
    let arc_state = Arc::clone(state.inner());

    let new_status = match status.as_str() {
        "enabled" => ExtensionStatus::Enabled,
        "disabled" => ExtensionStatus::Disabled,
        "browser-default" => ExtensionStatus::BrowserDefault,
        "undeveloped" => ExtensionStatus::Undeveloped,
        other => return Err(format!("未知状态值: {}", other)),
    };

    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard
        .plugins_config
        .set_extension_status(&ext, new_status.clone())?;
    plugins::save_plugins_config(&guard.plugins_config)?;
    let was_running = guard.cancel_token.is_some();
    drop(guard);

    if was_running {
        if let Err(e) = restart_server(&arc_state) {
            return Ok(format!("__RESTART_FAILED__{}", e));
        }
    }

    Ok("__OK__".to_string())
}

// 激活指定扩展名的 handler（互斥：同扩展名其它 handler 自动 Disabled）
#[tauri::command]
fn activate_plugin_handler(
    state: tauri::State<'_, Arc<Mutex<ServerState>>>,
    ext: String,
    handler_id: String,
) -> Result<String, String> {
    let arc_state = Arc::clone(state.inner());
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard
        .plugins_config
        .activate_handler(&ext, &handler_id)?;
    plugins::save_plugins_config(&guard.plugins_config)?;
    let was_running = guard.cancel_token.is_some();
    drop(guard);

    if was_running {
        if let Err(e) = restart_server(&arc_state) {
            return Ok(format!("__RESTART_FAILED__{}", e));
        }
    }

    Ok("__OK__".to_string())
}

// 获取 dist-web/plugins 目录绝对路径
#[tauri::command]
fn get_plugins_dir() -> Result<String, String> {
    let dir = plugins::get_plugins_dir()?;
    Ok(dir.to_string_lossy().replace('\\', "/"))
}

// 添加自定义插件：校验路径在 plugins 目录下，提取文件夹名作 handlerId
#[tauri::command]
fn add_custom_plugin(
    state: tauri::State<'_, Arc<Mutex<ServerState>>>,
    ext: String,
    folder_path: String,
) -> Result<String, String> {
    let arc_state = Arc::clone(state.inner());
    let plugins_dir = plugins::get_plugins_dir()?;
    let plugins_dir_canonical = plugins_dir
        .canonicalize()
        .unwrap_or_else(|_| plugins_dir.clone());

    let user_path = std::path::PathBuf::from(&folder_path);
    let user_path_canonical = user_path
        .canonicalize()
        .map_err(|_| format!("目录不存在: {}", folder_path))?;

    // 校验：目录必须在 plugins 目录下
    if !user_path_canonical.starts_with(&plugins_dir_canonical) {
        return Err(format!(
            "插件目录必须在 dist-web/plugins 下，当前: {}",
            folder_path
        ));
    }

    // 提取文件夹名
    let folder_name = user_path_canonical
        .file_name()
        .ok_or_else(|| "无法提取目录名".to_string())?
        .to_string_lossy()
        .to_string();

    // 校验 index.html 存在
    let index_html = user_path_canonical.join("index.html");
    if !index_html.exists() {
        return Err(format!(
            "插件目录缺少 index.html: {}",
            index_html.display()
        ));
    }

    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard
        .plugins_config
        .add_custom_handler(&ext, &folder_name)?;
    plugins::save_plugins_config(&guard.plugins_config)?;
    let was_running = guard.cancel_token.is_some();
    drop(guard);

    if was_running {
        if let Err(e) = restart_server(&arc_state) {
            return Ok(format!("__RESTART_FAILED__{}", e));
        }
    }

    Ok("__OK__".to_string())
}

#[tauri::command]
async fn open_url(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| e.to_string())
}