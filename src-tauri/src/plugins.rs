//! 插件配置管理模块
//!
//! 负责 plugins.json 的加载 / 保存，以及「扩展名 → 打开方式」的映射查询。
//! 🔧 注：get_default_plugins_path() 直接内置在本文件中，不依赖 config.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

// ==========================================
// 🔵 路径查找（从 config.rs 迁移而来，保持同样的 3 层回退策略）
// ==========================================

/// 查找可执行文件所在目录（开发环境 = target/debug，发布环境 = 打包后的 exe 目录）
fn get_exe_dir() -> Result<PathBuf, String> {
    let exe_path = env::current_exe()
        .map_err(|e| format!("无法获取当前可执行文件路径: {}", e))?;
    exe_path
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| "可执行文件没有父目录".to_string())
}

/// 获取 plugins.json 默认路径（与 config.json 同目录查找逻辑）
/// 查找顺序：
///   1. 可执行文件同级目录 -> plugins.json （开发: target/debug/plugins.json）
///   2. 可执行文件上一级目录 -> plugins.json （Tauri: src-tauri/plugins.json，因为 exe 在 target/debug）
///   3. 当前工作目录 -> plugins.json （兜底）
pub fn get_default_plugins_path() -> Result<PathBuf, String> {
    let exe_dir = get_exe_dir()?;

    // ① 直接在 exe_dir 下找
    let in_exe_dir = exe_dir.join("plugins.json");
    if in_exe_dir.exists()
        // 如果路径的父级是 target，说明是 Rust 开发模式，返回 exe_dir 下的路径
        || exe_dir
            .components()
            .any(|c| matches!(c, std::path::Component::Normal(p) if p == "target"))
    {
        // 🔁 特别处理：Tauri/Rust dev 模式（target/debug 或 target/release）
        // 尝试把 plugins.json 放在 src-tauri 根目录下（即 exe_dir 的 ../../plugins.json）
        if exe_dir
            .components()
            .any(|c| matches!(c, std::path::Component::Normal(p) if p == "target"))
        {
            let src_tauri_root = exe_dir
                .parent()          // target/debug -> target
                .and_then(|p| p.parent()) // target -> src-tauri
                .ok_or_else(|| "无法向上走到 src-tauri 目录".to_string())?;
            let path = src_tauri_root.join("plugins.json");
            println!("📁 [dev 模式] plugins.json 位于: {}", path.display());
            return Ok(path);
        }
        return Ok(in_exe_dir);
    }

    // ② 上一级目录
    if let Some(parent) = exe_dir.parent() {
        let in_parent = parent.join("plugins.json");
        if in_parent.exists() {
            return Ok(in_parent);
        }
    }

    // ③ 兜底：当前工作目录
    let cwd = env::current_dir()
        .map_err(|e| format!("获取当前工作目录失败: {}", e))?;
    let path = cwd.join("plugins.json");
    println!("⚠️ 使用兜底路径（当前工作目录）: {}", path.display());
    Ok(path)
}

// ==========================================
// 🔵 数据结构
// ==========================================

/// 单个扩展名的状态
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ExtensionStatus {
    BrowserDefault,
    Enabled,
    Disabled,
    Undeveloped,
}

/// 单个扩展名的完整配置条目
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionEntry {
    pub status: ExtensionStatus,
    #[serde(rename = "pluginId", alias = "plugin_id")]
    pub plugin_id: Option<String>,
    #[serde(rename = "urlTemplate", alias = "url_template")]
    pub url_template: Option<String>,
    pub description: String,
    pub name: String,
}

/// plugins.json 根结构
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PluginsConfig {
    pub extensions: HashMap<String, ExtensionEntry>,
}

// ==========================================
// 🟢 加载 / 保存函数
// ==========================================

/// 加载 plugins.json；若文件不存在则返回默认空配置 + 自动创建一个最小文件
pub fn load_plugins_config(plugins_path: Option<&str>) -> Result<PluginsConfig, String> {
    let path = match plugins_path {
        Some(p) => PathBuf::from(p),
        None => get_default_plugins_path()?,
    };

    if !path.exists() {
        println!("⚠️ plugins.json 不存在，使用默认空配置并创建骨架文件: {}", path.display());
        let default = PluginsConfig::default();
        save_plugins_config_to_path(&default, &path)?;
        return Ok(default);
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("读取 plugins.json 失败: {}", e))?;

    let cfg: PluginsConfig = serde_json::from_str(&content)
        .map_err(|e| format!("解析 plugins.json 失败: {}", e))?;

    println!("✅ 插件配置加载成功，共 {} 个扩展名条目", cfg.extensions.len());
    Ok(cfg)
}

/// 保存配置到指定路径
pub fn save_plugins_config_to_path(cfg: &PluginsConfig, path: &PathBuf) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建插件配置目录失败: {}", e))?;
        }
    }
    let json = serde_json::to_string_pretty(cfg)
        .map_err(|e| format!("序列化 plugins.json 失败: {}", e))?;
    std::fs::write(path, json)
        .map_err(|e| format!("写入 plugins.json 失败: {}", e))?;
    Ok(())
}

/// 保存到默认路径
pub fn save_plugins_config(cfg: &PluginsConfig) -> Result<(), String> {
    // ✅ 同样改为调用本文件自己的 get_default_plugins_path()
    let path = get_default_plugins_path()?;
    save_plugins_config_to_path(cfg, &path)
}

// ==========================================
// 🟡 业务辅助函数
// ==========================================

impl PluginsConfig {
   
    pub fn set_extension_status(
        &mut self,
        ext: &str,
        new_status: ExtensionStatus,
    ) -> Result<(), String> {
        let ext_key = ext.to_lowercase();
        let entry = self
            .extensions
            .entry(ext_key.clone())
            .or_insert_with(|| ExtensionEntry {
                status: new_status.clone(),
                plugin_id: None,
                url_template: None,
                description: format!("(.{}) 文件", ext_key),
                name: format!("{} 文件", ext_key.to_uppercase()),
            });
        entry.status = new_status;
        Ok(())
    }
    
}