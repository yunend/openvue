//! 统一的路径查找模块
//!
//! 🗂️ 两类目录必须区分开：
//!   1. **应用资源目录**（只读）—— 由 installer 放置，属 root/System，不可写
//!      放 dist-web/、默认配置模板等静态内容
//!   2. **用户配置目录**（可写）—— 用户个人空间，运行时读写 config.json / plugins.json
//!      Linux:   ~/.config/openvue/  (XDG Base Directory)
//!      macOS:   ~/Library/Application Support/openvue/
//!      Windows: %APPDATA%\openvue\
//!
//! 首次启动时，如果用户配置文件不存在，会从资源目录自动复制默认模板过去。

use std::path::PathBuf;

/// 找到应用的资源根目录
/// Windows:  直接返回 exe_dir（所有文件与 exe 同级）
/// macOS:    返回 Contents/Resources
/// Linux:    探测 /usr/lib*, /usr/share, /app/* 等发行版标准路径
///
/// 在返回的目录下应能找到 config.json、plugins.json、dist-web/ 等资源
pub fn find_app_root() -> Result<PathBuf, String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("无法获取当前可执行文件路径: {}", e))?;
    let exe_dir = exe_path.parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| "可执行文件没有父目录".to_string())?;

    // Windows / AppImage / 直接部署：exe 同级就是资源根
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        return Ok(exe_dir);
    }

    // macOS App Bundle：Contents/Resources
    #[cfg(target_os = "macos")]
    {
        if let Some(contents_dir) = exe_dir.parent() {
            let resources_dir = contents_dir.join("Resources");
            if resources_dir.exists() {
                println!("🍎 [macOS App] 资源根目录: {}", resources_dir.display());
                return Ok(resources_dir);
            }
        }
        // 非 App Bundle（开发模式 / 手动运行）
        return Ok(exe_dir);
    }

    // Linux 多发行版探测
    #[cfg(target_os = "linux")]
    {
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
                let candidate = PathBuf::from(base).join(name);
                if candidate.exists() {
                    println!("🐧 [Linux {}] 资源根目录: {}", base, candidate.display());
                    return Ok(candidate);
                }
            }
            let hardcoded = PathBuf::from(base).join("openvue");
            if hardcoded.exists() {
                println!("🐧 [Linux {}] 资源根目录: {}", base, hardcoded.display());
                return Ok(hardcoded);
            }
        }
        // 兜底：exe 同级
        println!("⚠️ [Linux] 未找到系统安装路径，使用 exe 同级: {}", exe_dir.display());
        Ok(exe_dir)
    }
}

// ============ 用户可写配置目录 ============

/// 返回用户配置目录（跨平台统一）：
///   Linux:   $XDG_CONFIG_HOME/openvue  (兜底 ~/.config/openvue)
///   macOS:   ~/Library/Application Support/openvue
///   Windows: %APPDATA%\openvue (兜底 %USERPROFILE%\AppData\Roaming\openvue)
/// 目录不存在时自动创建。
pub fn user_config_dir() -> Result<PathBuf, String> {
    let dir = do_find_user_config_dir();
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("创建用户配置目录失败 {}: {}", dir.display(), e))?;
    }
    println!("📂 用户配置目录: {}", dir.display());
    Ok(dir)
}

#[cfg(target_os = "linux")]
fn do_find_user_config_dir() -> PathBuf {
    // 优先尊重 XDG 环境变量
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join("openvue");
        }
    }
    // 兜底 ~/.config/openvue
    dirs_home().join(".config").join("openvue")
}

#[cfg(target_os = "macos")]
fn do_find_user_config_dir() -> PathBuf {
    dirs_home().join("Library").join("Application Support").join("openvue")
}

#[cfg(target_os = "windows")]
fn do_find_user_config_dir() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        if !appdata.is_empty() {
            return PathBuf::from(appdata).join("openvue");
        }
    }
    // 兜底 %USERPROFILE%\AppData\Roaming
    dirs_home().join("AppData").join("Roaming").join("openvue")
}

/// 获取用户主目录（跨平台）
fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            // 最后的最后，用当前目录
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        })
}

// ============ 便捷函数（区分「资源默认值」和「用户配置」） ============

/// 用户可写的 config.json 路径（运行时读写用这个）
pub fn config_path() -> Result<PathBuf, String> {
    Ok(user_config_dir()?.join("config.json"))
}

/// 应用资源目录里的 config.json 模板（只读，用于首次启动时复制到用户目录）
pub fn default_config_path() -> Result<PathBuf, String> {
    Ok(find_app_root()?.join("config.json"))
}

/// 用户可写的 plugins_state.json 路径（存储用户偏好：激活选择、下载源偏好）
pub fn plugins_state_path() -> Result<PathBuf, String> {
    Ok(user_config_dir()?.join("plugins_state.json"))
}

// dist-web 目录仍然在资源目录下（只读静态资源）
pub fn dist_web_dir() -> Result<PathBuf, String> {
    Ok(find_app_root()?.join("dist-web"))
}

/// 内置插件目录（资源目录/dist-web/plugins/，只读）
pub fn builtin_plugins_dir() -> Result<PathBuf, String> {
    Ok(dist_web_dir()?.join("plugins"))
}

/// 用户可写插件目录（用户配置目录/plugins/，默认放用户新建的插件）
pub fn user_plugins_dir() -> Result<PathBuf, String> {
    let dir = user_config_dir()?.join("plugins");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("创建用户插件目录失败 {}: {}", dir.display(), e))?;
    }
    Ok(dir)
}

/// 根据 AppConfig 的 plugins_folder 解析最终插件根目录
/// - Some(path) → 使用该路径（不存在则自动创建）
/// - None → 使用 user_plugins_dir()（用户配置目录/plugins/）
pub fn resolve_plugins_dir(config_plugins_folder: Option<&std::path::Path>) -> Result<PathBuf, String> {
    if let Some(pf) = config_plugins_folder {
        if !pf.exists() {
            std::fs::create_dir_all(pf)
                .map_err(|e| format!("创建插件目录失败 {}: {}", pf.display(), e))?;
        }
        Ok(pf.to_path_buf())
    } else {
        user_plugins_dir()
    }
}