//! 服务控制命令

use std::sync::{Arc, Mutex};
use crate::server;

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