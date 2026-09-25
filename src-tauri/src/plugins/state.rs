//! plugins_state.json 读写

use std::path::Path;

use super::types::PluginsState;

/// 加载 plugins_state.json（不存在时返回默认空状态）
pub fn load_plugins_state() -> Result<PluginsState, String> {
    let path = crate::paths::plugins_state_path()?;
    if !path.exists() {
        return Ok(PluginsState::default());
    }
    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("读取 plugins_state.json 失败: {}", e))?;
    let state: PluginsState = serde_json::from_str(&content)
        .map_err(|e| format!("解析 plugins_state.json 失败: {}", e))?;
    Ok(state)
}

/// 保存 plugins_state.json 到默认路径
pub fn save_plugins_state(state: &PluginsState) -> Result<(), String> {
    let path = crate::paths::plugins_state_path()?;
    save_plugins_state_to_path(state, &path)
}

/// 保存 plugins_state.json 到指定路径
pub fn save_plugins_state_to_path(state: &PluginsState, path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建 plugins_state 目录失败: {}", e))?;
        }
    }
    let json = serde_json::to_string_pretty(state)
        .map_err(|e| format!("序列化 plugins_state.json 失败: {}", e))?;
    std::fs::write(path, json).map_err(|e| format!("写入 plugins_state.json 失败: {}", e))?;
    println!(
        "💾 plugins_state.json 已保存（{} 个激活选择）",
        state.active_handlers.len()
    );
    Ok(())
}