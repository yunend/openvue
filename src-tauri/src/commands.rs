use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};
use std::cell::RefCell;

use crate::{server, config, plugins};

// ========== 窗口控制 ==========

#[tauri::command]
pub fn show_window(app: tauri::AppHandle) -> Result<(), String> {
    crate::tray::show_main_window(&app);
    Ok(())
}

#[tauri::command]
pub fn hide_window(app: tauri::AppHandle) -> Result<(), String> {
    crate::tray::hide_to_tray(&app);
    Ok(())
}

#[tauri::command]
pub fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

// ========== 配置管理 ==========

/// 获取当前配置
#[tauri::command]
pub fn get_config(state: tauri::State<'_, Arc<Mutex<server::ServerState>>>) -> Result<serde_json::Value, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    let plugins_folder = state.app_config.plugins_folder
        .as_ref()
        .map(|p| p.to_string_lossy().replace('\\', "/"));
    let effective_dir = crate::paths::resolve_plugins_dir(state.app_config.plugins_folder.as_deref())
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default();
    Ok(serde_json::json!({
        "port": state.app_config.port,
        "publicFolder": state.app_config.public_folder.to_string_lossy().to_string(),
        "enableUpload": state.app_config.enable_upload,
        "pluginsFolder": plugins_folder,
        "effectivePluginsDir": effective_dir,
    }))
}

/// 保存配置到 config.json
#[tauri::command]
pub fn save_config(
    state: tauri::State<'_, Arc<Mutex<server::ServerState>>>,
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

    let public_path = PathBuf::from(&public_folder);
    let abs_public_path = if public_path.is_absolute() {
        public_path.clone()
    } else {
        config_dir.join(&public_path)
    };

    // 保留原有的 plugins_folder
    let existing_plugins_folder = state.lock()
        .map_err(|e| e.to_string())?
        .app_config
        .plugins_folder
        .clone();

    let new_config = config::AppConfig {
        port,
        public_folder: abs_public_path.clone(),
        enable_upload,
        plugins_folder: existing_plugins_folder,
    };

    config::save_config_to_path(&new_config, &path).map_err(|e| e.to_string())?;

    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.app_config.port = port;
    state.app_config.public_folder = abs_public_path;
    state.app_config.enable_upload = enable_upload;

    let was_running = state.cancel_token.is_some();
    drop(state);

    if was_running {
        if let Err(e) = server::restart_server(&arc_state) {
            return Ok(format!("__RESTART_FAILED__{}", e));
        }
    }

    Ok("__OK__".to_string())
}

// ========== 服务控制 ==========

#[tauri::command]
pub fn start_server(state: tauri::State<'_, Arc<Mutex<server::ServerState>>>) -> Result<String, String> {
    let port = server::do_spawn_server(state.inner(), "")?;
    Ok(format!("HTTP 服务已启动，访问 http://127.0.0.1:{}", port))
}

#[tauri::command]
pub fn stop_server(state: tauri::State<Arc<Mutex<server::ServerState>>>) -> Result<String, String> {
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

/// 获取服务器运行状态
#[tauri::command]
pub fn get_server_status(state: tauri::State<'_, Arc<Mutex<server::ServerState>>>) -> Result<serde_json::Value, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    let is_running = state.cancel_token.is_some();
    let urls: Vec<String> = if is_running {
        server::get_local_ips()
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

// ========== 插件管理 ==========

#[tauri::command]
pub fn get_plugins_config(
    state: tauri::State<'_, Arc<Mutex<server::ServerState>>>,
) -> Result<plugins::PluginsConfig, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    Ok(guard.plugins_config.clone())
}

#[tauri::command]
pub fn set_plugin_browser_default(
    state: tauri::State<'_, Arc<Mutex<server::ServerState>>>,
    ext: String,
) -> Result<String, String> {
    let arc_state = Arc::clone(state.inner());
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard.plugins_config.set_browser_default(&ext)?;
    plugins::save_plugins_config(&guard.plugins_config)?;
    let was_running = guard.cancel_token.is_some();
    drop(guard);

    if was_running {
        if let Err(e) = server::restart_server(&arc_state) {
            return Ok(format!("__RESTART_FAILED__{}", e));
        }
    }
    Ok("__OK__".to_string())
}

#[tauri::command]
pub fn activate_plugin_handler(
    state: tauri::State<'_, Arc<Mutex<server::ServerState>>>,
    ext: String,
    handler_id: String,
) -> Result<String, String> {
    let arc_state = Arc::clone(state.inner());
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard.plugins_config.activate_handler(&ext, &handler_id)?;
    plugins::save_plugins_config(&guard.plugins_config)?;
    let was_running = guard.cancel_token.is_some();
    drop(guard);

    if was_running {
        if let Err(e) = server::restart_server(&arc_state) {
            return Ok(format!("__RESTART_FAILED__{}", e));
        }
    }
    Ok("__OK__".to_string())
}

#[tauri::command]
pub fn get_plugins_dir(
    state: tauri::State<'_, Arc<Mutex<server::ServerState>>>,
) -> Result<String, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    let dir = crate::paths::resolve_plugins_dir(guard.app_config.plugins_folder.as_deref())?;
    Ok(dir.to_string_lossy().replace('\\', "/"))
}

#[tauri::command]
pub fn add_custom_plugin(
    state: tauri::State<'_, Arc<Mutex<server::ServerState>>>,
    ext: String,
    folder_path: String,
) -> Result<String, String> {
    let arc_state = Arc::clone(state.inner());

    // 当前生效的插件根目录
    let plugins_root = {
        let guard = state.lock().map_err(|e| e.to_string())?;
        crate::paths::resolve_plugins_dir(guard.app_config.plugins_folder.as_deref())?
    };
    let plugins_root_canonical = plugins_root
        .canonicalize()
        .unwrap_or_else(|_| plugins_root.clone());

    let user_path = PathBuf::from(&folder_path);
    let user_path_canonical = user_path
        .canonicalize()
        .map_err(|_| format!("目录不存在: {}", folder_path))?;

    // ⚠️ 必须在 plugins_root 下，否则 /pfolder/ ServeDir 无法访问
    if !user_path_canonical.starts_with(&plugins_root_canonical) {
        return Err(format!(
            "插件目录必须位于插件根目录下。\n\n\
             插件根目录: {}\n\
             当前选择:   {}\n\n\
             请在【配置面板】修改插件根目录，或把插件文件夹移到插件根目录下。",
            plugins_root_canonical.display(),
            folder_path
        ));
    }

    let index_html = user_path_canonical.join("index.html");
    if !index_html.exists() {
        return Err(format!(
            "插件目录缺少 index.html: {}",
            index_html.display()
        ));
    }

    let folder_name = user_path_canonical
        .file_name()
        .ok_or_else(|| "无法提取目录名".to_string())?
        .to_string_lossy()
        .to_string();

    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard.plugins_config.add_custom_handler(&ext, &folder_name)?;
    plugins::save_plugins_config(&guard.plugins_config)?;
    let was_running = guard.cancel_token.is_some();
    drop(guard);

    if was_running {
        if let Err(e) = server::restart_server(&arc_state) {
            return Ok(format!("__RESTART_FAILED__{}", e));
        }
    }
    Ok("__OK__".to_string())
}

/// 设置插件根目录（写入 config.json + 更新运行时状态 + 重启服务）
#[tauri::command]
pub fn set_plugins_folder(
    state: tauri::State<'_, Arc<Mutex<server::ServerState>>>,
    folder_path: String,
) -> Result<String, String> {
    let arc_state = Arc::clone(state.inner());
    let path = config::get_default_config_path().map_err(|e| e.to_string())?;

    let pf = PathBuf::from(&folder_path);
    if !pf.exists() {
        std::fs::create_dir_all(&pf)
            .map_err(|e| format!("创建插件目录失败: {} ({})", folder_path, e))?;
    }

    let mut current_config = config::load_config(Some(path.to_string_lossy().as_ref()))?;
    current_config.plugins_folder = Some(pf.clone());
    config::save_config_to_path(&current_config, &path)?;

    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard.app_config.plugins_folder = Some(pf);
    let was_running = guard.cancel_token.is_some();
    drop(guard);

    if was_running {
        if let Err(e) = server::restart_server(&arc_state) {
            return Ok(format!("__RESTART_FAILED__{}", e));
        }
    }
    Ok("__OK__".to_string())
}

/// 重置插件根目录为默认（清除 config.json 中的 pluginsFolder 字段）
#[tauri::command]
pub fn reset_plugins_folder(
    state: tauri::State<'_, Arc<Mutex<server::ServerState>>>,
) -> Result<String, String> {
    let arc_state = Arc::clone(state.inner());
    let path = config::get_default_config_path().map_err(|e| e.to_string())?;

    let mut current_config = config::load_config(Some(path.to_string_lossy().as_ref()))?;
    current_config.plugins_folder = None;
    config::save_config_to_path(&current_config, &path)?;

    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard.app_config.plugins_folder = None;
    let was_running = guard.cancel_token.is_some();
    drop(guard);

    if was_running {
        if let Err(e) = server::restart_server(&arc_state) {
            return Ok(format!("__RESTART_FAILED__{}", e));
        }
    }
    Ok("__OK__".to_string())
}

/// 用系统文件管理器打开当前生效的插件目录
#[tauri::command]
pub fn open_plugins_folder(
    state: tauri::State<'_, Arc<Mutex<server::ServerState>>>,
) -> Result<(), String> {
    let dir = {
        let guard = state.lock().map_err(|e| e.to_string())?;
        crate::paths::resolve_plugins_dir(guard.app_config.plugins_folder.as_deref())?
    };
    open::that(&dir).map_err(|e| e.to_string())
}

// ========== 系统 / 工具 ==========

/// 获取应用版本号
#[tauri::command]
pub fn get_version() -> String {
    let version = env!("CARGO_PKG_VERSION");
    let tauri_version = tauri::VERSION;
    println!("🔖 版本查询: app={}, tauri={}", version, tauri_version);
    version.to_string()
}

/// 弹出「选择文件夹」对话框
#[tauri::command]
pub async fn choose_folder(
    app: tauri::AppHandle,
    initial_dir: Option<String>,
    title: Option<String>,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let (tx, rx) = tokio::sync::oneshot::channel::<Option<PathBuf>>();
    let tx_cell = RefCell::new(Some(tx));

    let mut builder = app.dialog().file();
    let dialog_title = title.unwrap_or_else(|| "选择文件夹".to_string());
    builder = builder.set_title(&dialog_title);

    // 🧭 处理 initial_dir：确保绝对路径 + 存在 + 规范化
    if let Some(dir_str) = initial_dir {
        let raw = PathBuf::from(&dir_str);
        let resolved = if raw.is_absolute() {
            raw.clone()
        } else {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(&raw)
        };
        let exists = resolved.exists();
        // ⚠️ Windows canonicalize 会返回 \\?\ UNC 前缀路径，dialog 的 set_directory 不认
        let canonical_raw = resolved.canonicalize().unwrap_or_else(|_| resolved.clone());
        let canonical = normalize_path_for_dialog(&canonical_raw);
        println!("📂 [choose_folder] title={}", dialog_title);
        println!("📂 [choose_folder] initial_dir 原始: {}", dir_str);
        println!("📂 [choose_folder] 解析后: {}", resolved.display());
        println!("📂 [choose_folder] 存在: {} | 规范化后: {}", exists, canonical.display());
        if exists {
            builder = builder.set_directory(&canonical);
            println!("📂 [choose_folder] ✅ 已设置初始目录");
        } else {
            println!("⚠️ [choose_folder] 初始目录不存在，跳过 set_directory");
        }
    } else {
        println!("📂 [choose_folder] 无 initial_dir，使用系统默认");
    }

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

#[tauri::command]
pub async fn open_url(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| e.to_string())
}

/// 清理 Windows canonicalize 产生的 \\?\ UNC 前缀
/// 某些原生 Windows API（如文件对话框）不认这种前缀
fn normalize_path_for_dialog(p: &std::path::Path) -> std::path::PathBuf {
    let s = p.to_string_lossy().to_string();
    let cleaned = if let Some(rest) = s.strip_prefix(r"\\?\") {
        rest.to_string()
    } else if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{}", rest)
    } else {
        s
    };
    std::path::PathBuf::from(cleaned)
}