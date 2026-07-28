// e:\dev\test-tauri\tauri-app\src-tauri\src\lib.rs

mod config;
mod plugins;
mod router;

use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;
use tauri_plugin_autostart::MacosLauncher;
use tauri::{tray::TrayIconBuilder, menu::Menu, AppHandle, Manager};

struct ServerState {
    cancel_token: Option<CancellationToken>,
    app_config: config::AppConfig,
    /// ✅ 插件配置（运行时可变内存副本，保存后直接更新这里）
    plugins_config: plugins::PluginsConfig,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
            Some(vec!["--minimized"]),
        ))
        // ================================================================
        // ✅ 初始化 dialog 插件（和 autostart 一样是独立插件 init）
        // ================================================================
        .plugin(tauri_plugin_dialog::init())
        .manage(server_state.clone())
        .setup(|app| {
            setup_system_tray(app)?;
            let args: Vec<String> = std::env::args().collect();
    
            if args.contains(&"--minimized".to_string()) {
                if let Some(window) = app.get_webview_window("main") {
                    window.hide()?;
                }
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
        .tooltip("Tauri HTTP 服务")
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
        window.show().unwrap();
        window.set_focus().unwrap();
    }
}

/// 隐藏到托盘
fn hide_to_tray(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().unwrap();
    }
}

#[tauri::command]
fn start_server(state: tauri::State<'_, Arc<Mutex<ServerState>>>) -> Result<String, String> {  
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    
    if guard.cancel_token.is_some() {
        return Err("HTTP 服务已在运行".to_string());
    }
    
    let app_config = guard.app_config.clone();
    let plugins_config = guard.plugins_config.clone();  // ✅ 必须在 drop(guard) 之前取出
    let port = app_config.port;
    
    let cancel_token = CancellationToken::new();
    let token_clone = cancel_token.clone();
    guard.cancel_token = Some(cancel_token);
    drop(guard); 
    
    tauri::async_runtime::spawn(async move {
        let static_folder = app_config.static_folder.clone();
        let enable_upload = app_config.enable_upload; // ✅ 读开关
        let plugins_for_router = plugins_config;
        // ✅ 编译期嵌入 Cargo.toml 的版本号 → 传给 /api/about 接口
        let version_str = env!("CARGO_PKG_VERSION").to_string();
        let app = router::create_router(static_folder, enable_upload, version_str, port, plugins_for_router); // ✅ 传 plugins_config
        
        let addr = format!("127.0.0.1:{}", port);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .expect(&format!("无法绑定端口 {}", port));
        println!("HTTP 服务器启动在 http://{}", addr);
        
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                token_clone.cancelled().await;
            })
            .await
            .ok();
        println!("HTTP 服务器已停止");
    });
    
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
        "staticFolder": state.app_config.static_folder.to_string_lossy().to_string(),
        "enableUpload": state.app_config.enable_upload,
    }))
}

/// 保存配置到 config.json
#[tauri::command]
fn save_config(
    state: tauri::State<'_, Arc<Mutex<ServerState>>>,
    port: u16,
    static_folder: String,
    enable_upload: bool,
) -> Result<String, String> {
    // 1. 基础验证
    if port == 0 {
        return Err("端口号不能为 0".to_string());
    }
    if static_folder.trim().is_empty() {
        return Err("公共文件目录不能为空".to_string());
    }

    let path = config::get_default_config_path().map_err(|e| e.to_string())?;
    let config_dir = path.parent()
        .ok_or_else(|| "无法获取配置目录".to_string())?.to_path_buf();

    // 2. 处理相对路径 → 转为绝对路径用于内存
    let static_path = std::path::PathBuf::from(&static_folder);
    let abs_static_path = if static_path.is_absolute() {
        static_path.clone()
    } else {
        config_dir.join(&static_path)
    };

    // 3. 构建新的 AppConfig 对象
    let new_config = config::AppConfig {
        port,
        static_folder: abs_static_path.clone(),
        enable_upload,
    };

    // 4. 写入文件（使用 config.rs 的标准函数，自动写相对路径）
    config::save_config_to_path(&new_config, &path).map_err(|e| e.to_string())?;

    // 5. 更新内存中的配置
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.app_config.port = port;
    state.app_config.static_folder = abs_static_path;
    state.app_config.enable_upload = enable_upload;

    let msg = if state.cancel_token.is_some() {
        "配置已保存，重启 HTTP 服务后生效"
    } else {
        "配置已保存"
    };

    Ok(msg.to_string())
}

/// 获取服务器运行状态
#[tauri::command]
fn get_server_status(state: tauri::State<'_, Arc<Mutex<ServerState>>>) -> Result<serde_json::Value, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    let is_running = state.cancel_token.is_some();
    Ok(serde_json::json!({
        "isRunning": is_running,
        "port": state.app_config.port,
        "staticFolder": state.app_config.static_folder.to_string_lossy().to_string(),
        "enableUpload": state.app_config.enable_upload,
        "url": if is_running {
            serde_json::Value::String(format!("http://127.0.0.1:{}", state.app_config.port))
        } else {
            serde_json::Value::Null
        }
    }))
}

/// 获取应用版本号（从 Cargo.toml 的 version 字段读取）
#[tauri::command]
fn get_version() -> String {
    // env! 是编译期宏，编译时直接把 Cargo.toml 里的 version 值嵌入进来
    // 不需要读文件，零运行时开销，打包后也准确
    let version = env!("CARGO_PKG_VERSION");
    let tauri_version = tauri::VERSION;
    println!("🔖 版本查询: app={}, tauri={}", version, tauri_version);
    format!("{}", version)
}

/// 弹出「选择文件夹」对话框 - Tauri 2.x (tauri-plugin-dialog 2)
///
/// 前端调用示例：await invoke('choose_folder', { initialDir: 'E:/dev' })
/// 返回：选中的绝对路径字符串（正斜杠分隔）；用户点取消 → 返回 null
#[tauri::command]
async fn choose_folder(
    app: tauri::AppHandle,
    initial_dir: Option<String>,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    use std::path::{Path, PathBuf};
    use std::cell::RefCell;

    // ── 用 oneshot channel 把回调 API 桥接成 async Future ──
    let (tx, rx) = tokio::sync::oneshot::channel::<Option<PathBuf>>();
    // 用 RefCell 包 Sender，避免闭包只能 FnOnce 的限制
    // （dialog 插件回调签名要求 FnMut，move 进闭包的 tx 只能用一次，需要内部可变性）
    let tx_cell = RefCell::new(Some(tx));

    // ── 第 1 步：构建对话框 ──
    let mut builder = app.dialog().file();
    builder = builder.set_title("选择公共文件根目录");

    if let Some(dir) = initial_dir {
        let p = PathBuf::from(&dir);
        if p.exists() {
            builder = builder.set_directory(p);
        }
    }

    // ── 第 2 步：pick_folder + 回调里 as_path() 取路径 ──
    builder.pick_folder(move |fp_opt| {
        let path_opt: Option<PathBuf> = fp_opt.and_then(|fp| {
            // ✅ 用编译器提示的 as_path() 方法：
            //    enum FilePath { Path(PathBuf), Url(Url) }
            //    as_path() → 本地路径变体返回 Some(&Path)，URL 返回 None
            fp.as_path().map(|p: &Path| p.to_path_buf())
        });

        if let Some(tx) = tx_cell.borrow_mut().take() {
            let _ = tx.send(path_opt);
        }
    });

    // ── 第 3 步：await 结果 ──
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

// ================================================================
// ✅ 插件配置：获取完整 plugins.json 内容
// ================================================================
#[tauri::command]
fn get_plugins_config(
    state: tauri::State<'_, Arc<Mutex<ServerState>>>,
) -> Result<plugins::PluginsConfig, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    Ok(guard.plugins_config.clone())
}

// ================================================================
// ✅ 插件配置：切换 / 设置某扩展名的状态（enabled ↔ disabled）
//    ext: 扩展名（不带点），status: "enabled" / "disabled"
// ================================================================
#[tauri::command]
fn save_plugin_extension_status(
    state: tauri::State<'_, Arc<Mutex<ServerState>>>,
    ext: String,
    status: String,
) -> Result<String, String> {
    use plugins::ExtensionStatus;

    // 1. 解析 status 字符串 → 枚举
    let new_status = match status.as_str() {
        "enabled" => ExtensionStatus::Enabled,
        "disabled" => ExtensionStatus::Disabled,
        "browser-default" => ExtensionStatus::BrowserDefault,
        "undeveloped" => ExtensionStatus::Undeveloped,
        other => return Err(format!("未知状态值: {}", other)),
    };

    // 2. 更新内存
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard
        .plugins_config
        .set_extension_status(&ext, new_status.clone())?;

    // 3. 持久化到 plugins.json
    plugins::save_plugins_config(&guard.plugins_config)?;

    Ok(format!(
        "✅ 扩展名 .{} 状态已更新为 {:?}",
        ext.to_lowercase(),
        new_status
    ))
}