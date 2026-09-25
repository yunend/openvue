//! 配置管理命令

use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use crate::{server, config};

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