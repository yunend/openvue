//! 插件管理 + 插件市场 + 下载源偏好命令

use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use tauri::Emitter;
use crate::{server, config, plugins};

// ========== 内部辅助 ==========

/// 重新扫描插件目录并重建运行时配置
fn rescan_and_rebuild(
    guard: &mut server::ServerState,
    state: &plugins::PluginsState,
) -> Result<(), String> {
    let builtin_dir = crate::paths::builtin_plugins_dir()?;
    let user_dir = crate::paths::resolve_plugins_dir(guard.app_config.plugins_folder.as_deref())?;
    guard.plugins_config = plugins::scan_and_build_config(&builtin_dir, &user_dir, state);
    Ok(())
}

// ========== 插件配置查询 ==========

#[tauri::command]
pub fn get_plugins_config(
    state: tauri::State<'_, Arc<Mutex<server::ServerState>>>,
) -> Result<plugins::PluginsConfig, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    Ok(guard.plugins_config.clone())
}

#[tauri::command]
pub fn get_plugins_dir(
    state: tauri::State<'_, Arc<Mutex<server::ServerState>>>,
) -> Result<String, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    let dir = crate::paths::resolve_plugins_dir(guard.app_config.plugins_folder.as_deref())?;
    Ok(dir.to_string_lossy().replace('\\', "/"))
}

// ========== 激活 / 浏览器默认切换 ==========

#[tauri::command]
pub fn set_plugin_browser_default(
    state: tauri::State<'_, Arc<Mutex<server::ServerState>>>,
    ext: String,
) -> Result<String, String> {
    let arc_state = Arc::clone(state.inner());
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard.plugins_config.set_browser_default(&ext)?;
    let mut plugins_state = plugins::load_plugins_state().unwrap_or_default();
    plugins_state.clear_active(&ext);
    plugins::save_plugins_state(&plugins_state)?;
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
    let mut plugins_state = plugins::load_plugins_state().unwrap_or_default();
    plugins_state.set_active(&ext, &handler_id);
    plugins::save_plugins_state(&plugins_state)?;
    let was_running = guard.cancel_token.is_some();
    drop(guard);
    if was_running {
        if let Err(e) = server::restart_server(&arc_state) {
            return Ok(format!("__RESTART_FAILED__{}", e));
        }
    }
    Ok("__OK__".to_string())
}

// ========== 自定义插件 ==========

#[tauri::command]
pub fn add_custom_plugin(
    state: tauri::State<'_, Arc<Mutex<server::ServerState>>>,
    ext: String,
    folder_path: String,
) -> Result<String, String> {
    let arc_state = Arc::clone(state.inner());

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
        return Err(format!("插件目录缺少 index.html: {}", index_html.display()));
    }

    let folder_name = user_path_canonical
        .file_name()
        .ok_or_else(|| "无法提取目录名".to_string())?
        .to_string_lossy()
        .to_string();

    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard.plugins_config.add_custom_handler(&ext, &folder_name)?;

    let mut plugins_state = plugins::load_plugins_state().unwrap_or_default();
    plugins_state.set_active(&ext, &folder_name);
    plugins::save_plugins_state(&plugins_state)?;

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
pub fn remove_custom_plugin(
    state: tauri::State<'_, Arc<Mutex<server::ServerState>>>,
    ext: String,
    folder_path: String,
) -> Result<String, String> {
    let arc_state = Arc::clone(state.inner());
    let user_path = PathBuf::from(&folder_path);
    let folder_name = user_path
        .file_name()
        .ok_or_else(|| "无法从路径中提取目录名".to_string())?
        .to_string_lossy()
        .to_string();

    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard.plugins_config.remove_custom_handler(&ext, &folder_name)?;

    let mut plugins_state = plugins::load_plugins_state().unwrap_or_default();
    plugins_state.clear_active(&ext);
    plugins::save_plugins_state(&plugins_state)?;

    let was_running = guard.cancel_token.is_some();
    drop(guard);
    if was_running {
        if let Err(e) = server::restart_server(&arc_state) {
            return Ok(format!("__RESTART_FAILED__{}", e));
        }
    }
    Ok("__OK__".to_string())
}

// ========== 插件根目录 ==========

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
    let plugins_state = plugins::load_plugins_state().unwrap_or_default();
    let _ = rescan_and_rebuild(&mut guard, &plugins_state);
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
    let plugins_state = plugins::load_plugins_state().unwrap_or_default();
    let _ = rescan_and_rebuild(&mut guard, &plugins_state);
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
pub fn open_plugins_folder(
    state: tauri::State<'_, Arc<Mutex<server::ServerState>>>,
) -> Result<(), String> {
    let dir = {
        let guard = state.lock().map_err(|e| e.to_string())?;
        crate::paths::resolve_plugins_dir(guard.app_config.plugins_folder.as_deref())?
    };
    open::that(&dir).map_err(|e| e.to_string())
}

// ========== 插件市场 ==========

/// 从插件市场安装一个插件（只生成 plugin.json，不下载实际文件）
#[tauri::command]
pub fn install_plugin_from_market(
    state: tauri::State<'_, Arc<Mutex<server::ServerState>>>,
    plugin_json: serde_json::Value,
) -> Result<String, String> {
    let arc_state = Arc::clone(state.inner());

    let meta: plugins::PluginMeta = serde_json::from_value(plugin_json)
        .map_err(|e| format!("解析插件元信息失败: {}", e))?;

    let plugins_root = {
        let guard = state.lock().map_err(|e| e.to_string())?;
        crate::paths::resolve_plugins_dir(guard.app_config.plugins_folder.as_deref())?
    };

    let plugin_dir = plugins_root.join(&meta.id);
    if !plugin_dir.exists() {
        std::fs::create_dir_all(&plugin_dir)
            .map_err(|e| format!("创建插件目录失败: {}", e))?;
    }

    let plugin_json_path = plugin_dir.join("plugin.json");
    let json_str = serde_json::to_string_pretty(&meta)
        .map_err(|e| format!("序列化 plugin.json 失败: {}", e))?;
    std::fs::write(&plugin_json_path, &json_str)
        .map_err(|e| format!("写入 plugin.json 失败: {}", e))?;

    println!("📦 插件市场安装 [{}]: plugin.json 已生成 -> {}",
        meta.id, plugin_json_path.display());

    let mut guard = state.lock().map_err(|e| e.to_string())?;
    let plugins_state = plugins::load_plugins_state().unwrap_or_default();
    let _ = rescan_and_rebuild(&mut guard, &plugins_state);
    let was_running = guard.cancel_token.is_some();
    drop(guard);
    if was_running {
        if let Err(e) = server::restart_server(&arc_state) {
            return Ok(format!("__RESTART_FAILED__{}", e));
        }
    }
    Ok("__OK__".to_string())
}

// ========== 插件市场 ==========

/// 从远程 URL 获取插件市场索引（后端请求，无 CORS 限制）
#[tauri::command]
pub async fn fetch_plugins_index(url: String) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let resp = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("解析 JSON 失败: {}", e))?;

    Ok(json)
}

// ========== 下载安装 ==========

use std::sync::atomic::AtomicBool;
use std::sync::LazyLock;
use std::collections::HashMap;
use crate::plugins::download::DownloadProgress;

// 全局取消标志表
static DOWNLOAD_CANCEL_FLAGS: LazyLock<Mutex<HashMap<String, Arc<AtomicBool>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// 下载并安装插件（完整流程：下载→校验→解压→安装）
#[tauri::command]
pub async fn download_and_install_plugin(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<Mutex<server::ServerState>>>,
    plugin_json: serde_json::Value,
) -> Result<String, String> {
    let meta: plugins::PluginMeta = serde_json::from_value(plugin_json)
        .map_err(|e| format!("解析插件元信息失败: {}", e))?;

    let pid = meta.id.clone();

    // 获取插件根目录
    let plugins_root = {
        let guard = state.lock().map_err(|e| e.to_string())?;
        crate::paths::resolve_plugins_dir(guard.app_config.plugins_folder.as_deref())?
    };

    // 创建取消标志
    let cancel_flag = Arc::new(AtomicBool::new(false));
    {
        let mut flags = DOWNLOAD_CANCEL_FLAGS.lock().map_err(|e| e.to_string())?;
        flags.insert(pid.clone(), cancel_flag.clone());
    }

    // 构建进度回调（通过 Tauri Event 发送到前端）
    let app_clone = app.clone();
    let progress: plugins::ProgressCallback = Arc::new(move |dp: DownloadProgress| {
        let _ = app_clone.emit("plugin-download-progress", &dp);
    });

    // 获取用户偏好状态
    let plugins_state = plugins::load_plugins_state().unwrap_or_default();

    // 执行下载安装
    let result = plugins::install_plugin(
        &plugins_root,
        &meta,
        &plugins_state,
        progress.clone(),
        cancel_flag.clone(),
    )
    .await;

    // 清除取消标志
    {
        let mut flags = DOWNLOAD_CANCEL_FLAGS.lock().map_err(|e| e.to_string())?;
        flags.remove(&pid);
    }

    // 如果成功，重新扫描配置
    if let Err(e) = &result {
        // 发送失败事件
        let _ = app.emit("plugin-download-error", serde_json::json!({
            "pluginId": pid,
            "error": e,
        }));
        return Err(e.clone());
    }

    // 安装后验证：打印目标目录内容
    let verify_dir = plugins_root.join(&pid);
    eprintln!("🔎 [download_and_install_plugin] 安装后验证目录: {:?}", verify_dir);
    eprintln!("   📁 目录存在: {}", verify_dir.exists());
    eprintln!("   📄 plugin.json 存在: {}", verify_dir.join("plugin.json").exists());
    if verify_dir.exists() {
        match std::fs::read_dir(&verify_dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    eprintln!("      - {}", entry.path().display());
                }
            }
            Err(e) => eprintln!("   ❌ 读取目录失败: {}", e),
        }
    }

    // 成功：重新扫描插件目录
    let arc_state = Arc::clone(state.inner());
    {
        let mut guard = state.lock().map_err(|e| e.to_string())?;
        let ps = plugins::load_plugins_state().unwrap_or_default();
        if let Err(e) = rescan_and_rebuild(&mut guard, &ps) {
            eprintln!("⚠️ rescan_and_rebuild 失败: {}", e);
        }
        let was_running = guard.cancel_token.is_some();
        drop(guard);
        if was_running {
            if let Err(e) = server::restart_server(&arc_state) {
                eprintln!("⚠️ server::restart_server 失败: {}", e);
            }
        }
    }

    Ok("__OK__".to_string())
}

/// 取消正在进行的插件下载
#[tauri::command]
pub fn cancel_plugin_download(plugin_id: String) -> Result<(), String> {
    let flags = DOWNLOAD_CANCEL_FLAGS.lock().map_err(|e| e.to_string())?;
    if let Some(flag) = flags.get(&plugin_id) {
        flag.store(true, std::sync::atomic::Ordering::Relaxed);
        println!("🛑 已请求取消插件 [{}] 的下载", plugin_id);
    }
    Ok(())
}

/// 卸载插件
#[tauri::command]
pub fn uninstall_plugin_command(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<Mutex<server::ServerState>>>,
    plugin_id: String,
) -> Result<String, String> {
    let arc_state = Arc::clone(state.inner());
    let plugins_root = {
        let guard = state.lock().map_err(|e| e.to_string())?;
        crate::paths::resolve_plugins_dir(guard.app_config.plugins_folder.as_deref())?
    };

    plugins::uninstall_plugin(&plugins_root, &plugin_id)?;

    // 重新扫描
    {
        let mut guard = state.lock().map_err(|e| e.to_string())?;
        let ps = plugins::load_plugins_state().unwrap_or_default();
        let _ = rescan_and_rebuild(&mut guard, &ps);
        let was_running = guard.cancel_token.is_some();
        drop(guard);
        if was_running {
            let _ = server::restart_server(&arc_state);
        }
    }

    let _ = app.emit("plugin-uninstalled", serde_json::json!({
        "pluginId": plugin_id,
    }));

    Ok("__OK__".to_string())
}

/// 检查插件是否有更新
#[tauri::command]
pub fn check_plugin_update_command(
    state: tauri::State<'_, Arc<Mutex<server::ServerState>>>,
    plugin_json: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let meta: plugins::PluginMeta = serde_json::from_value(plugin_json)
        .map_err(|e| format!("解析插件元信息失败: {}", e))?;

    let plugins_root = {
        let guard = state.lock().map_err(|e| e.to_string())?;
        crate::paths::resolve_plugins_dir(guard.app_config.plugins_folder.as_deref())?
    };

    let need_update = plugins::check_plugin_update(&plugins_root, &meta)?;
    let installed = plugins::is_plugin_installed(&plugins_root, &meta.id);

    Ok(serde_json::json!({
        "pluginId": meta.id,
        "needUpdate": need_update,
        "installed": installed,
        "remoteVersion": meta.version,
    }))
}

/// 批量检查多个插件的安装状态
#[tauri::command]
pub fn get_plugins_install_status(
    state: tauri::State<'_, Arc<Mutex<server::ServerState>>>,
    plugin_ids: Vec<String>,
) -> Result<serde_json::Value, String> {
    let plugins_root = {
        let guard = state.lock().map_err(|e| e.to_string())?;
        crate::paths::resolve_plugins_dir(guard.app_config.plugins_folder.as_deref())?
    };
    let builtin_root = crate::paths::builtin_plugins_dir().ok();

    eprintln!("🔍 [get_plugins_install_status] 插件根目录: {:?}", plugins_root);
    if let Some(ref b) = builtin_root {
        eprintln!("   📁 内置目录: {:?}", b);
    }

    let mut statuses = serde_json::Map::new();
    for pid in &plugin_ids {
        // 同时检查 用户目录 和 内置目录
        let installed_user = plugins::is_plugin_installed(&plugins_root, pid);
        let installed_builtin = builtin_root
            .as_ref()
            .map(|r| plugins::is_plugin_installed(r, pid))
            .unwrap_or(false);
        let installed = installed_user || installed_builtin;

        // 调试输出来诊断文件系统状态
        let user_dir = plugins_root.join(pid);
        let user_json = user_dir.join("plugin.json");
        eprintln!(
            "   🧪 [{}] 用户目录存在:{} plugin.json存在:{} | is_installed={}",
            pid,
            user_dir.exists(),
            user_json.exists(),
            installed
        );

        // 优先级：用户目录 > 内置目录
        let check_dir = if installed_user {
            plugins_root.clone()
        } else if installed_builtin {
            builtin_root.clone().unwrap()
        } else {
            plugins_root.clone()
        };

        let version = if installed {
            let json_path = plugins::installed_plugin_json_path(&check_dir, pid);
            std::fs::read_to_string(&json_path)
                .ok()
                .and_then(|s| serde_json::from_str::<plugins::PluginMeta>(&s).ok())
                .map(|m| m.version)
                .unwrap_or_default()
        } else {
            String::new()
        };

        statuses.insert(pid.clone(), serde_json::json!({
            "installed": installed,
            "version": version,
        }));
    }

    Ok(serde_json::Value::Object(statuses))
}

// ========== 下载源偏好 ==========

#[tauri::command]
pub fn get_preferred_download_source(
    _state: tauri::State<'_, Arc<Mutex<server::ServerState>>>,
) -> Result<Option<String>, String> {
    let ps = plugins::load_plugins_state()?;
    Ok(ps.preferred_download_source_label)
}

#[tauri::command]
pub fn set_preferred_download_source(
    _state: tauri::State<'_, Arc<Mutex<server::ServerState>>>,
    label: String,
) -> Result<String, String> {
    let mut ps = plugins::load_plugins_state()?;
    ps.preferred_download_source_label = if label.is_empty() {
        None
    } else {
        Some(label)
    };
    plugins::save_plugins_state(&ps)?;
    Ok("__OK__".to_string())
}