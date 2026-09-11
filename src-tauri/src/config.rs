//! 配置管理模块
//!
//! 从 config.json 加载应用配置（端口号、指定文件路径、上传开关等）

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    /// HTTP 服务器端口号
    pub port: u16,

    /// 指定文件目录路径（JSON 中为 publicFolder）
    pub public_folder: PathBuf,

    /// 是否启用文件上传（JSON 中为 enableUpload）
    pub enable_upload: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            port: 8005,
            public_folder: PathBuf::from("public"),
            enable_upload: false, // 默认关闭上传，更安全
        }
    }
}

/// 从 config.json 加载配置
/// config_path: 配置文件路径（None 则用用户目录下的可写路径）
///
/// ⚡ 首次启动时，如果用户目录下还没有 config.json，
///    会自动从应用资源目录（只读）复制默认模板过去，再加载。
pub fn load_config(config_path: Option<&str>) -> Result<AppConfig, String> {
    let path = match config_path {
        Some(p) => PathBuf::from(p),
        None => get_default_config_path()?,
    };

    // 首次启动：从资源目录复制默认模板到用户可写目录
    if !path.exists() {
        let template = crate::paths::default_config_path()?;
        if template.exists() {
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("创建配置目录失败: {}", e))?;
                }
            }
            std::fs::copy(&template, &path).map_err(|e| {
                format!(
                    "从模板复制 config.json 失败: {} → {} ({})",
                    template.display(),
                    path.display(),
                    e
                )
            })?;
            println!("📋 config.json 首次启动，已从资源模板复制: {}", path.display());
        } else {
            // 资源目录也没有模板，就直接用 Default 构造一份写过去
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent).ok();
                }
            }
            let default_cfg = AppConfig::default();
            save_config_to_path(&default_cfg, &path)?;
            println!("📋 config.json 首次启动（无模板），已写入默认配置: {}", path.display());
        }
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    let mut config: AppConfig = serde_json::from_str(&content)
        .map_err(|e| format!("解析 JSON 失败: {}", e))?;

    // 解析相对路径
    if !config.public_folder.is_absolute() {
        let config_dir = path.parent()
            .ok_or_else(|| "无法获取配置文件目录".to_string())?;
        let raw = config_dir.join(&config.public_folder);

        // 🔧 关键修复：如果用户目录下找不到，就回退到资源目录查找
        // （首次启动时 config.json 在 ~/.config/openvue/，但 public/ 还在 /usr/lib/openvue/）
        let resolved = if raw.exists() {
            raw
        } else {
            match crate::paths::find_app_root() {
                Ok(app_root) => {
                    let fallback = app_root.join(&config.public_folder);
                    if fallback.exists() {
                        println!("⚠️ 用户目录下未找到 {}，回退到资源目录: {}", 
                                 config.public_folder.display(), fallback.display());
                        fallback
                    } else {
                        raw // 两边都找不到，保持原样让 validate_config 报错
                    }
                }
                Err(_) => raw, // 拿不到资源目录，保持原样
            }
        };

        config.public_folder = match resolved.canonicalize() {
            Ok(p) => normalize_path(p),
            Err(_) => normalize_path(resolved),
        };
    } else {
        config.public_folder = normalize_path(config.public_folder.clone());
    }

    println!("✅ 配置加载成功:");
    println!("   配置文件: {}", path.display());
    println!("   端口: {}", config.port);
    println!("   指定文件目录: {}", config.public_folder.display());
    println!("   文件上传: {}", if config.enable_upload { "✅ 启用" } else { "❌ 禁用" });

    Ok(config)
}

/// 获取默认配置文件路径
/// 委托给 paths::config_path()，自动处理 Windows / macOS / Linux 差异
pub fn get_default_config_path() -> Result<PathBuf, String> {
    crate::paths::config_path()
}

/// 验证配置有效性
pub fn validate_config(config: &AppConfig) -> Result<(), String> {
    if config.port == 0 {
        return Err("端口号不能为 0".to_string());
    }

    if !config.public_folder.exists() {
        return Err(format!(
            "指定文件目录不存在: {}",
            config.public_folder.display()
        ));
    }

    Ok(())
}


/// 保存配置到指定路径（public_folder 自动写回相对路径）
pub fn save_config_to_path(config: &AppConfig, path: &PathBuf) -> Result<(), String> {
    let config_dir = path.parent()
        .ok_or_else(|| "无法获取配置文件目录".to_string())?;

    let public_folder_write = if config.public_folder.is_absolute() {
        // 转为相对 config_dir 的路径（更友好）
        match config.public_folder.strip_prefix(config_dir) {
            Ok(rel) => rel.to_string_lossy().to_string(),
            Err(_) => config.public_folder.to_string_lossy().to_string(),
        }
    } else {
        config.public_folder.to_string_lossy().to_string()
    };

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct ConfigFile {
        port: u16,
        public_folder: String,
        enable_upload: bool,
    }

    let file_data = ConfigFile {
        port: config.port,
        public_folder: public_folder_write,
        enable_upload: config.enable_upload,
    };

    let json = serde_json::to_string_pretty(&file_data)
        .map_err(|e| format!("序列化配置失败: {}", e))?;

    std::fs::write(path, json)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(())
}

fn normalize_path(p: PathBuf) -> PathBuf {
    let s = p.to_string_lossy().to_string();
    let cleaned = if let Some(rest) = s.strip_prefix(r"\\?\") {
        rest.to_string()
    } else if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
        // UNC 路径特例：\\?\UNC\server\share → \\server\share
        format!(r"\\{}", rest)
    } else {
        s
    };
    PathBuf::from(cleaned)
}