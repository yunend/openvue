//! 插件配置管理模块（plugins.json 加载 / 保存 + 扩展名映射查询）

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

/// 查找可执行文件所在目录
fn get_exe_dir() -> Result<PathBuf, String> {
    let exe_path = env::current_exe()
        .map_err(|e| format!("无法获取当前可执行文件路径: {}", e))?;
    exe_path
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| "可执行文件没有父目录".to_string())
}

/// plugins.json 默认路径（与 config.json 同目录查找逻辑）
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
        // Tauri/Rust dev 模式（target/debug 或 release）→ src-tauri/plugins.json
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

    // Linux 系统安装包资源目录探测
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

    // macOS App Bundle：Contents/Resources
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

/// 单个扩展名的状态（支持 kebab-case 和 PascalCase 两种 JSON 写法）
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

/// 单个扩展名的一个处理器
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

/// 单个扩展名配置：多个备选处理器 + 当前激活的 handler_id
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionConfig {
    pub handlers: Vec<ExtensionHandler>,
/// 当前激活的 handler_id；None 表示走 browser-default
    #[serde(default)]
    pub active_handler_id: Option<String>,
}

/// plugins.json 根结构
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PluginsConfig {
    pub extensions: HashMap<String, ExtensionConfig>,
}

/// 加载 plugins.json；不存在则返回空配置并创建骨架文件
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
    let path = get_default_plugins_path()?;
    save_plugins_config_to_path(cfg, &path)
}

/// 获取 dist-web/plugins 目录的绝对路径
/// 查找策略与 router 中资源目录解析一致
pub fn get_plugins_dir() -> Result<PathBuf, String> {
    let exe_dir = get_exe_dir()?;

    // 开发模式：target/debug 或 target/release 下 → 向上走到 src-tauri/dist-web/plugins
    if exe_dir
        .components()
        .any(|c| matches!(c, std::path::Component::Normal(p) if p == "target"))
    {
        let src_tauri_root = exe_dir
            .parent()
            .and_then(|p| p.parent())
            .ok_or_else(|| "无法找到 src-tauri 目录".to_string())?;
        let path = src_tauri_root.join("dist-web").join("plugins");
        println!("📁 [dev 模式] plugins 目录: {}", path.display());
        return Ok(path);
    }

    // 生产模式：exe 同级目录下的 dist-web/plugins
    let path = exe_dir.join("dist-web").join("plugins");
    println!("📁 [生产模式] plugins 目录: {}", path.display());
    Ok(path)
}

impl PluginsConfig {
    /// 激活指定 handler：同扩展名其他 Enabled→Disabled，目标 Disabled→Enabled
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

        // 校验 handler 是否存在
        if !config.handlers.iter().any(|h| h.handler_id == handler_id) {
            return Err(format!(
                "扩展名 .{} 下找不到处理器 id={}",
                ext_key, handler_id
            ));
        }

        // 遍历 handlers：目标 Disabled→Enabled；其他 Enabled→Disabled
        for handler in config.handlers.iter_mut() {
            if handler.handler_id == handler_id {
                if matches!(handler.status, ExtensionStatus::Disabled) {
                    handler.status = ExtensionStatus::Enabled;
                }
            } else if matches!(handler.status, ExtensionStatus::Enabled) {
                handler.status = ExtensionStatus::Disabled;
            }
        }

        // 只有 Enabled 才写入 active_handler_id，否则 None
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

    /// 设置指定 handler 的状态（Enabled 时维持互斥）
    pub fn set_handler_status(
        &mut self,
        ext: &str,
        handler_id: &str,
        new_status: ExtensionStatus,
    ) -> Result<(), String> {
        let ext_key = ext.to_lowercase();

        // 扩展名不存在且设 default 处理器 → 兜底创建
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

        // handler_id 不存在 → 自动追加
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

        // 互斥：新状态为 Enabled 时，其他 Enabled→Disabled
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

        // 同步 active_handler_id 到第一个 Enabled 处理器
        config.active_handler_id = config
            .handlers
            .iter()
            .find(|h| matches!(h.status, ExtensionStatus::Enabled))
            .map(|h| h.handler_id.clone());

        Ok(())
    }

    /// 添加自定义插件处理器（folder_name 作为 handlerId/pluginId）
    pub fn add_custom_handler(
        &mut self,
        ext: &str,
        folder_name: &str,
    ) -> Result<(), String> {
        let ext_key = ext.to_lowercase();
        let handler_id = folder_name.to_string();
        let plugin_id = folder_name.to_string();
        let url_template = "/plugins/{pluginId}/?path={publicPath}".to_string();
        let name = format!("自定义{}", ext_key);

        let new_handler = ExtensionHandler {
            handler_id: handler_id.clone(),
            status: ExtensionStatus::Enabled,
            plugin_id: Some(plugin_id),
            url_template: Some(url_template),
            description: String::new(),
            name,
        };

        let config = self.extensions.entry(ext_key.clone()).or_insert_with(|| ExtensionConfig {
            handlers: vec![],
            active_handler_id: None,
        });

        // 同扩展名下其他 Enabled 处理器 → Disabled（互斥）
        for handler in config.handlers.iter_mut() {
            if matches!(handler.status, ExtensionStatus::Enabled) {
                handler.status = ExtensionStatus::Disabled;
            }
        }

        config.handlers.push(new_handler);
        config.active_handler_id = Some(handler_id);

        Ok(())
    }

    /// 兼容旧前端：把扩展名首个 handler 设为指定状态
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