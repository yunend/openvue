// e:\dev\test-tauri\tauri-app\src-tauri\src\config.rs
//! 配置管理模块
//!
//! 从 config.json 加载应用配置（端口号、指定文件路径、上传开关等）

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ==========================================
// 🔵 配置数据结构
// ==========================================

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    /// HTTP 服务器端口号
    pub port: u16,

    /// 指定文件目录路径
    /// config.json 中写 "publicFolder"，Rust 中读为 public_folder
    pub public_folder: PathBuf,

    /// ✅ 是否启用文件上传功能
    /// config.json 中写 "enableUpload"，Rust 中读为 enable_upload
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

// ==========================================
// 🟢 配置加载函数
// ==========================================

/// 从 config.json 文件加载配置
///
/// # 参数
/// - `config_path`: 配置文件的路径（可选）
///
/// # 返回
/// - 成功：返回 `AppConfig`
/// - 失败：返回错误信息字符串
pub fn load_config(config_path: Option<&str>) -> Result<AppConfig, String> {
    let path = match config_path {
        Some(p) => PathBuf::from(p),
        None => get_default_config_path()?,
    };

    if !path.exists() {
        return Err(format!("配置文件不存在: {}", path.display()));
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    let mut config: AppConfig = serde_json::from_str(&content)
        .map_err(|e| format!("解析 JSON 失败: {}", e))?;

    // ====== 解析相对路径 ======
    if !config.public_folder.is_absolute() {
        let config_dir = path.parent()
            .ok_or_else(|| "无法获取配置文件目录".to_string())?;
        let raw = config_dir.join(&config.public_folder);
        // ✅ 先尝试 canonicalize（存在就取真实路径），失败就直接 normalize 软规范化
        config.public_folder = match raw.canonicalize() {
            Ok(p) => normalize_path(p),
            Err(_) => normalize_path(raw),
        };
    } else {
        // 绝对路径也处理一下 \\?\
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
///
/// 查找顺序：
/// 1. exe 同级目录下的 config.json   （生产环境 / 打包后）
/// 2. 工作目录下的 src-tauri/config.json （开发环境）
/// 3. 工作目录下的 config.json
pub fn get_default_config_path() -> Result<PathBuf, String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("获取可执行文件路径失败: {}", e))?;

    let exe_dir = exe_path.parent()
        .ok_or_else(|| "无法获取可执行文件目录".to_string())?;

    // ---- 1. 生产环境：exe 同级 config.json（打包后）----
    let config_in_exe_dir = exe_dir.join("config.json");
    if config_in_exe_dir.exists() {
        return Ok(config_in_exe_dir);
    }

    // ---- 2. 开发环境：向上查找含 Cargo.toml 的目录 ----
    let mut probe_dir: Option<&std::path::Path> = Some(exe_dir);
    while let Some(dir) = probe_dir {
        let candidate = dir.join("config.json");
        let has_cargo = dir.join("Cargo.toml").exists();
        if candidate.exists() && has_cargo {
            return Ok(candidate.canonicalize().unwrap_or(candidate));
        }
        probe_dir = dir.parent();
    }

    // ---- 3. 当前工作目录下查找 ----
    if let Ok(cwd) = std::env::current_dir() {
        let in_src_tauri = cwd.join("config.json");
        if in_src_tauri.exists() {
            return Ok(in_src_tauri.canonicalize().unwrap_or(in_src_tauri));
        }
        let in_parent = cwd.join("src-tauri/config.json");
        if in_parent.exists() {
            return Ok(in_parent.canonicalize().unwrap_or(in_parent));
        }
    }

    Err(format!(
        "未找到配置文件。请确保存在 config.json：\n   \
         - [打包后] 安装目录 (exe 同级)\n   \
         - [开发时]  src-tauri/config.json\n   \
         当前 exe 目录: {}", exe_dir.display()
    ))
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


/// 保存配置到指定路径
pub fn save_config_to_path(config: &AppConfig, path: &PathBuf) -> Result<(), String> {
    // 获取 config 所在目录（用于处理 public_folder 相对路径写入）
    // 写入时：如果 public_folder 恰好位于 config_dir 下，写回相对路径
    let config_dir = path.parent()
        .ok_or_else(|| "无法获取配置文件目录".to_string())?;

    let public_folder_write = if config.public_folder.is_absolute() {
        // 尝试转为相对于 config_dir 的相对路径（更友好）
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

