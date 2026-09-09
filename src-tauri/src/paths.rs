//! 统一的路径查找模块
//!
//! Windows:  所有资源（config.json / plugins.json / dist-web / public）都在 exe 同级
//! macOS:    App Bundle → Contents/Resources
//! Linux:    多发行版探测（deb/rpm/Arch/Flatpak）

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

// ============ 便捷函数 ============

pub fn config_path() -> Result<PathBuf, String> {
    Ok(find_app_root()?.join("config.json"))
}

pub fn plugins_path() -> Result<PathBuf, String> {
    Ok(find_app_root()?.join("plugins.json"))
}

pub fn dist_web_dir() -> Result<PathBuf, String> {
    Ok(find_app_root()?.join("dist-web"))
}

pub fn plugins_dir() -> Result<PathBuf, String> {
    Ok(dist_web_dir()?.join("plugins"))
}