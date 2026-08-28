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
///   1. 可执行文件同级目录 -> plugins.json （Windows/macOS/AppImage）
///   1.5 Linux 多路径探测：/usr/lib,/usr/lib64,/usr/share,/app/lib,/app/share
///   2. 可执行文件上一级目录 -> plugins.json （Tauri: src-tauri/plugins.json）
///   3. 当前工作目录 -> plugins.json （兜底）
pub fn get_default_plugins_path() -> Result<PathBuf, String> {
    let exe_dir = get_exe_dir()?;
    let exe_path = env::current_exe()
        .map_err(|e| format!("无法获取当前可执行文件路径: {}", e))?;

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

    // ①.5 🐧 Linux 系统安装包：资源目录探测（deb/rpm/Arch/Flatpak 等）
    if cfg!(target_os = "linux") {
        let exe_name = exe_path.file_stem().map(|s| s.to_string_lossy().to_string());
        let linux_base_dirs: [&str; 5] = [
            "/usr/lib",      // Debian/Ubuntu deb
            "/usr/lib64",    // Fedora/RHEL rpm (64-bit)
            "/usr/share",    // Arch/FHS 标准（架构无关资源）
            "/app/lib",      // Flatpak 运行时
            "/app/share",    // Flatpak 运行时（架构无关）
        ];
        for base in linux_base_dirs {
            if let Some(ref name) = exe_name {
                let candidate = PathBuf::from(base).join(name).join("plugins.json");
                if candidate.exists() {
                    println!("🐧 [Linux {}] plugins.json 位于: {}", base, candidate.display());
                    return Ok(candidate);
                }
            }
            let hardcoded = PathBuf::from(base).join("openvue").join("plugins.json");
            if hardcoded.exists() {
                println!("🐧 [Linux {}] plugins.json 位于: {}", base, hardcoded.display());
                return Ok(hardcoded);
            }
        }
    }

    // ①.6 🍎 macOS App Bundle：资源在 Contents/Resources（exe 位于 Contents/MacOS）
    if cfg!(target_os = "macos") {
        if let Some(contents_dir) = exe_dir.parent() {
            let resources_dir = contents_dir.join("Resources");
            let candidate = resources_dir.join("plugins.json");
            if candidate.exists() {
                println!("🍎 [macOS App] plugins.json 位于: {}", candidate.display());
                return Ok(candidate);
            }
        }
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
/// 同时支持两种 JSON 写法：kebab-case（browser-default / enabled / disabled / undeveloped）和 PascalCase（BrowserDefault / Enabled / Disabled / Undeveloped）
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ExtensionStatus {
    #[serde(alias = "BrowserDefault")]
    BrowserDefault,
    #[serde(alias = "Enabled")]
    Enabled,
    #[serde(alias = "Disabled")]
    Disabled,
    #[serde(alias = "Undeveloped")]
    Undeveloped,
}

/// 单个扩展名下的一个「处理器」——一个扩展名可以有多个备选处理器，但只有一个 active
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionHandler {
    /// 处理器唯一 ID（同一扩展名内不重复），如 "pdf-native" / "pdf-onlyoffice"
    pub handler_id: String,
    pub status: ExtensionStatus,
    #[serde(rename = "pluginId", alias = "plugin_id", default)]
    pub plugin_id: Option<String>,
    #[serde(rename = "urlTemplate", alias = "url_template", default)]
    pub url_template: Option<String>,
    pub description: String,
    pub name: String,
}

/// 单个扩展名的完整配置：多个备选处理器 + 一个当前激活的处理器 ID
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionConfig {
    pub handlers: Vec<ExtensionHandler>,
    /// 当前激活的处理器 handler_id；None 表示没有激活项（走 browser-default）
    #[serde(default)]
    pub active_handler_id: Option<String>,
}

/// plugins.json 根结构（仅支持新格式：每个扩展名 = handlers[] + active_handler_id）
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PluginsConfig {
    pub extensions: HashMap<String, ExtensionConfig>,
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
    /// 指定扩展名 + 指定 handler_id，把那个处理器设为【当前激活项】
    /// - 同一扩展名下所有其他处理器：如果是 Enabled → 自动降级为 Disabled（保证互斥）
    /// - 目标处理器：若为 Disabled → 自动升级为 Enabled；其余状态保留
    /// 返回该处理器的新状态引用
    pub fn activate_handler(
        &mut self,
        ext: &str,
        handler_id: &str,
    ) -> Result<(), String> {
        let ext_key = ext.to_lowercase();
        let config = self
            .extensions
            .get_mut(&ext_key)
            .ok_or_else(|| format!("扩展名 .{} 不存在", ext_key))?;

        // 1. 校验 handler 是否存在
        if !config.handlers.iter().any(|h| h.handler_id == handler_id) {
            return Err(format!(
                "扩展名 .{} 下找不到处理器 id={}",
                ext_key, handler_id
            ));
        }

        // 2. 遍历所有 handlers：除目标外，Enabled → Disabled；目标本身 Disabled → Enabled
        for handler in config.handlers.iter_mut() {
            if handler.handler_id == handler_id {
                if matches!(handler.status, ExtensionStatus::Disabled) {
                    handler.status = ExtensionStatus::Enabled;
                }
            } else if matches!(handler.status, ExtensionStatus::Enabled) {
                handler.status = ExtensionStatus::Disabled;
            }
        }

        // 3. 只有目标处理器状态为 Enabled 时才写入 active_handler_id；否则置 None
        let target = config
            .handlers
            .iter()
            .find(|h| h.handler_id == handler_id)
            .unwrap();
        config.active_handler_id = if matches!(target.status, ExtensionStatus::Enabled) {
            Some(handler_id.to_string())
        } else {
            None
        };

        Ok(())
    }

    /// 直接改变指定处理器的状态（不强制互斥，只改那一个条目），
    /// 若目标设为 Enabled 且其他处理器也有 Enabled，将自动降级其他，以维持互斥语义
    pub fn set_handler_status(
        &mut self,
        ext: &str,
        handler_id: &str,
        new_status: ExtensionStatus,
    ) -> Result<(), String> {
        let ext_key = ext.to_lowercase();

        // 若扩展名不存在，且用户想创建一条 default 处理器 → 允许兜底创建
        let config = self.extensions.entry(ext_key.clone()).or_insert_with(|| ExtensionConfig {
            handlers: vec![ExtensionHandler {
                handler_id: "default".to_string(),
                status: ExtensionStatus::Undeveloped,
                plugin_id: None,
                url_template: None,
                description: format!("(.{}) 文件", ext_key),
                name: format!("{} 文件", ext_key.to_uppercase()),
            }],
            active_handler_id: None,
        });

        // 如果要设置的 handler_id 在当前 handlers 中不存在 → 自动加一条（兜底）
        if !config.handlers.iter().any(|h| h.handler_id == handler_id) {
            config.handlers.push(ExtensionHandler {
                handler_id: handler_id.to_string(),
                status: new_status.clone(),
                plugin_id: None,
                url_template: None,
                description: format!("(.{}) 处理器 {}", ext_key, handler_id),
                name: format!("{} {}", ext_key.to_uppercase(), handler_id),
            });
        }

        // 状态互斥：如果新状态是 Enabled，其他 handler 的 Enabled → Disabled
        if matches!(new_status, ExtensionStatus::Enabled) {
            for handler in config.handlers.iter_mut() {
                if handler.handler_id != handler_id
                    && matches!(handler.status, ExtensionStatus::Enabled)
                {
                    handler.status = ExtensionStatus::Disabled;
                }
            }
        }

        // 应用目标 handler 状态
        for handler in config.handlers.iter_mut() {
            if handler.handler_id == handler_id {
                handler.status = new_status.clone();
                break;
            }
        }

        // 同步 active_handler_id：指向第一个状态为 Enabled 的处理器；若没有则为 None
        config.active_handler_id = config
            .handlers
            .iter()
            .find(|h| matches!(h.status, ExtensionStatus::Enabled))
            .map(|h| h.handler_id.clone());

        Ok(())
    }

    /// 保留旧 API 签名：把整个扩展名统一设为某状态（只对【首个】handler 生效，兼容旧前端调用）
    pub fn set_extension_status(
        &mut self,
        ext: &str,
        new_status: ExtensionStatus,
    ) -> Result<(), String> {
        let ext_key = ext.to_lowercase();
        let first_handler_id = self
            .extensions
            .get(&ext_key)
            .and_then(|cfg| cfg.handlers.first().map(|h| h.handler_id.clone()))
            .unwrap_or_else(|| "default".to_string());
        self.set_handler_status(ext, &first_handler_id, new_status)
    }
}