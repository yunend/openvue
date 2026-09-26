//! 插件下载、校验、解压、安装、卸载核心功能

use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use sha2::{Sha256, Digest};
use uuid::Uuid;

use super::types::{PluginMeta, DownloadSource, PluginsState};

/// 下载进度信息（通过 Tauri Event 发给前端）
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub plugin_id: String,
    /// 0.0 ~ 100.0
    pub percentage: f64,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    /// 当前阶段：downloading / verifying / extracting / installing
    pub stage: String,
    pub message: String,
}

/// 进度回调
pub type ProgressCallback = Arc<dyn Fn(DownloadProgress) + Send + Sync + 'static>;

/// 无进度回调的空实现
pub fn noop_progress() -> ProgressCallback {
    Arc::new(|_| {})
}

// ========== 下载源选择 ==========

/// 根据用户偏好和 priority 选择下载 URL
/// 优先级：preferredDownloadSourceLabel → priority（数值小优先）
pub fn resolve_download_url(
    sources: &[DownloadSource],
    state: &PluginsState,
) -> Result<(String, String), String> {
    if sources.is_empty() {
        return Err("插件没有可用的下载源".to_string());
    }

    // 1. 如果用户有偏好标签，先用它
    if let Some(ref preferred) = state.preferred_download_source_label {
        if let Some(source) = sources.iter().find(|s| s.label == *preferred) {
            return Ok((source.label.clone(), source.url.clone()));
        }
    }

    // 2. 按 priority 排序（优先选 priority 最小的）
    let mut sorted: Vec<&DownloadSource> = sources.iter().collect();
    sorted.sort_by_key(|s| s.priority);

    let best = sorted[0];
    Ok((best.label.clone(), best.url.clone()))
}

// ========== 下载 ==========

/// 下载 ZIP 文件到本地临时路径，通过回调报告进度
pub async fn download_zip(
    url: &str,
    dest_path: &Path,
    plugin_id: &str,
    progress: ProgressCallback,
    cancel_flag: Arc<AtomicBool>,
) -> Result<(), String> {
    let client = reqwest::Client::new();

    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("下载请求失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("下载失败: HTTP {}", resp.status()));
    }

    let total_size = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    // 创建文件
    let mut file = std::fs::File::create(dest_path)
        .map_err(|e| format!("创建临时文件失败: {}", e))?;

    let mut stream = resp.bytes_stream();

    use futures_util::StreamExt;
    while let Some(chunk_result) = stream.next().await {
        if cancel_flag.load(Ordering::Relaxed) {
            let _ = std::fs::remove_file(dest_path);
            return Err("下载已取消".to_string());
        }

        let chunk = chunk_result.map_err(|e| format!("下载数据流错误: {}", e))?;
        let len = chunk.len() as u64;
        downloaded += len;

        // 写入文件
        use std::io::Write;
        file.write_all(&chunk)
            .map_err(|e| format!("写入文件失败: {}", e))?;

        // 报告进度
        let percentage = if total_size > 0 {
            (downloaded as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };

        progress(DownloadProgress {
            plugin_id: plugin_id.to_string(),
            percentage,
            downloaded_bytes: downloaded,
            total_bytes: total_size,
            stage: "downloading".to_string(),
            message: format!("下载中 {:.1}%", percentage),
        });
    }

    // 刷写到磁盘
    file.sync_all().map_err(|e| format!("刷写文件失败: {}", e))?;

    Ok(())
}

// ========== SHA256 校验 ==========

/// 计算文件的 SHA256 十六进制字符串
pub fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = std::fs::File::open(path)
        .map_err(|e| format!("打开文件失败: {}", e))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = file.read(&mut buffer)
            .map_err(|e| format!("读取文件失败: {}", e))?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

/// 验证文件的 SHA256 是否与预期一致
pub fn verify_sha256(file_path: &Path, expected_hex: &str) -> Result<(), String> {
    if expected_hex.is_empty() {
        return Err("sha256 为空，无法校验".to_string());
    }
    let actual = sha256_file(file_path)?;
    if actual.to_lowercase() != expected_hex.to_lowercase() {
        return Err(format!(
            "SHA256 不匹配！\n  预期: {}\n  实际: {}",
            expected_hex, actual
        ));
    }
    println!("✅ SHA256 校验通过: {} ({}...)", file_path.display(), &actual[..16]);
    Ok(())
}

// ========== ZIP 解压 ==========

/// 将 ZIP 文件解压到目标目录
pub fn extract_zip(zip_path: &Path, target_dir: &Path) -> Result<(), String> {
    let file = std::fs::File::open(zip_path)
        .map_err(|e| format!("打开 ZIP 文件失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("读取 ZIP 文件失败: {}", e))?;

    // 确保目标目录存在
    std::fs::create_dir_all(target_dir)
        .map_err(|e| format!("创建解压目录失败: {}", e))?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| format!("读取 ZIP 条目失败: {}", e))?;

        let entry_name = entry.name().to_string();
        // 安全防护：防止 zip slip 攻击
        let safe_name = entry_name
            .replace('\\', "/")
            .trim_start_matches('/')
            .to_string();
        if safe_name.is_empty() {
            continue;
        }

        let out_path = target_dir.join(&safe_name);

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)
                .map_err(|e| format!("创建目录失败 {}: {}", safe_name, e))?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("创建父目录失败 {}: {}", parent.display(), e))?;
            }
            let mut outfile = std::fs::File::create(&out_path)
                .map_err(|e| format!("创建文件失败 {}: {}", safe_name, e))?;
            std::io::copy(&mut entry, &mut outfile)
                .map_err(|e| format!("解压文件失败 {}: {}", safe_name, e))?;
        }
    }

    println!("✅ ZIP 解压完成: {} → {}", zip_path.display(), target_dir.display());
    Ok(())
}

// ========== 插件状态查询 ==========

/// 检查插件是否已安装（目录存在且 plugin.json 存在）
pub fn is_plugin_installed(plugins_root: &Path, plugin_id: &str) -> bool {
    let dir = plugins_root.join(plugin_id);
    if !dir.exists() || !dir.is_dir() {
        return false;
    }
    dir.join("plugin.json").exists()
}

/// 获取已安装插件的 plugin.json 路径
pub fn installed_plugin_json_path(plugins_root: &Path, plugin_id: &str) -> PathBuf {
    plugins_root.join(plugin_id).join("plugin.json")
}

/// 检查已安装的插件版本和 hash 是否匹配（用于判断是否有更新）
pub fn check_plugin_update(
    plugins_root: &Path,
    remote_meta: &PluginMeta,
) -> Result<bool, String> {
    let installed_path = installed_plugin_json_path(plugins_root, &remote_meta.id);
    if !installed_path.exists() {
        return Ok(true); // 未安装，需要下载
    }
    let content = std::fs::read_to_string(&installed_path)
        .map_err(|e| format!("读取已安装 plugin.json 失败: {}", e))?;
    let installed_meta: PluginMeta = serde_json::from_str(&content)
        .map_err(|e| format!("解析已安装 plugin.json 失败: {}", e))?;

    // 版本不同 → 需要更新
    if installed_meta.version != remote_meta.version {
        return Ok(true);
    }
    // 版本相同但 hash 不同 → 需要更新
    if installed_meta.sha256 != remote_meta.sha256 {
        return Ok(true);
    }
    // 完全一致 → 已是最新
    Ok(false)
}

// ========== 安装流程 ==========

/// 下载安装插件的完整流程
/// 1. 在用户插件目录下创建临时目录（带 UUID）
/// 2. 在临时目录生成 plugin.json
/// 3. 下载 ZIP 到临时目录
/// 4. 校验 SHA256
/// 5. 解压 ZIP
/// 6. 移动到目标插件目录（覆盖已有文件）
/// 7. 将 plugin.json 复制回插件目录
/// 8. 返回成功
pub async fn install_plugin(
    plugins_root: &Path,
    meta: &PluginMeta,
    state: &PluginsState,
    progress: ProgressCallback,
    cancel_flag: Arc<AtomicBool>,
) -> Result<(), String> {
    let plugin_id = &meta.id;
    let pid = plugin_id.to_string();

    // ---- 1. 选择下载源 ----
    progress(DownloadProgress {
        plugin_id: pid.clone(),
        percentage: 0.0,
        downloaded_bytes: 0,
        total_bytes: 0,
        stage: "resolving".to_string(),
        message: "选择下载源...".to_string(),
    });

    let (_label, download_url) = resolve_download_url(&meta.download_sources, state)?;

    // ---- 2. 创建临时目录 ----
    let temp_id = Uuid::new_v4().to_string();
    let temp_dir = plugins_root.join(format!(".tmp_{}_{}", plugin_id, temp_id));
    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir)
            .map_err(|e| format!("清除旧临时目录失败: {}", e))?;
    }
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("创建临时目录失败: {}", e))?;

    // 确保最终目标路径
    let target_dir = plugins_root.join(plugin_id);

    let cleanup = || -> Result<(), String> {
        if temp_dir.exists() {
            std::fs::remove_dir_all(&temp_dir).ok();
        }
        // 下载中途取消时，如果目标目录已经部分写入，不要删
        Ok(())
    };

    // ---- 3. 在临时目录生成 plugin.json ----
    let temp_plugin_json = temp_dir.join("plugin.json");
    let meta_json = serde_json::to_string_pretty(meta)
        .map_err(|e| format!("序列化 plugin.json 失败: {}", e))?;
    std::fs::write(&temp_plugin_json, &meta_json)
        .map_err(|e| format!("写入临时 plugin.json 失败: {}", e))?;

    // ---- 4. 下载 ZIP 到临时目录 ----
    let zip_path = temp_dir.join(format!("{}.zip", plugin_id));
    progress(DownloadProgress {
        plugin_id: pid.clone(),
        percentage: 0.0,
        downloaded_bytes: 0,
        total_bytes: 0,
        stage: "downloading".to_string(),
        message: "开始下载...".to_string(),
    });

    download_zip(&download_url, &zip_path, plugin_id, progress.clone(), cancel_flag.clone()).await?;

    if cancel_flag.load(Ordering::Relaxed) {
        cleanup()?;
        return Err("下载已取消".to_string());
    }

    // ---- 5. 校验 SHA256 ----
    progress(DownloadProgress {
        plugin_id: pid.clone(),
        percentage: 100.0,
        downloaded_bytes: 0,
        total_bytes: 0,
        stage: "verifying".to_string(),
        message: "校验文件完整性...".to_string(),
    });

    verify_sha256(&zip_path, &meta.sha256)?;

    if cancel_flag.load(Ordering::Relaxed) {
        cleanup()?;
        return Err("校验完成后取消".to_string());
    }

    // ---- 6. 解压 ZIP ----
    progress(DownloadProgress {
        plugin_id: pid.clone(),
        percentage: 100.0,
        downloaded_bytes: 0,
        total_bytes: 0,
        stage: "extracting".to_string(),
        message: "解压文件中...".to_string(),
    });

    let extract_dir = temp_dir.join("extracted");
    extract_zip(&zip_path, &extract_dir)?;

    // ---- 7. 移动到目标插件目录 ----
    progress(DownloadProgress {
        plugin_id: pid.clone(),
        percentage: 100.0,
        downloaded_bytes: 0,
        total_bytes: 0,
        stage: "installing".to_string(),
        message: "安装中...".to_string(),
    });

    // 如果目标目录已存在，先删除（避免 rename 跨文件系统失败）
    if target_dir.exists() {
        std::fs::remove_dir_all(&target_dir)
            .map_err(|e| format!("删除旧插件目录失败: {}", e))?;
    }

    // 检查解压内容：如果 ZIP 根只有一个目录，使用该目录的内容
    let mut source_for_move = extract_dir.clone();
    let extract_entries: Vec<_> = std::fs::read_dir(&extract_dir)
        .map_err(|e| format!("读取解压目录失败: {}", e))?
        .filter_map(|e| e.ok())
        .collect();

    // 如果解压后只有一个子目录，且该子目录名不是 plugin.json，则用该子目录的内容
    let use_inner =
        extract_entries.len() == 1 && extract_entries[0].path().is_dir();
    if use_inner {
        source_for_move = extract_entries[0].path();
    }

    // 使用 copy 替代 rename（兼容跨文件系统）
    fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), String> {
        std::fs::create_dir_all(dst)
            .map_err(|e| format!("创建目标目录失败 {}: {}", dst.display(), e))?;
        for entry in std::fs::read_dir(src).map_err(|e| format!("读取目录失败 {}: {}", src.display(), e))? {
            let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
            let entry_type = entry.file_type().map_err(|e| format!("获取文件类型失败: {}", e))?;
            let src_path = entry.path();
            let file_name = src_path.file_name().unwrap_or_default();
            let dst_path = dst.join(file_name);

            if entry_type.is_dir() {
                copy_dir_all(&src_path, &dst_path)?;
            } else {
                std::fs::copy(&src_path, &dst_path)
                    .map_err(|e| format!("复制文件失败 {}: {}", src_path.display(), e))?;
            }
        }
        Ok(())
    }

    copy_dir_all(&source_for_move, &target_dir)?;

    // ---- 8. 将 plugin.json 复制到最终位置（确保存在） ----
    let final_plugin_json = target_dir.join("plugin.json");
    std::fs::copy(&temp_plugin_json, &final_plugin_json)
        .map_err(|e| format!("复制 plugin.json 到目标目录失败: {}", e))?;

    // 强制刷写文件系统缓存，确保后续查询立即可见
    if let Ok(f) = std::fs::File::open(&final_plugin_json) {
        let _ = f.sync_all();
    }
    // 刷写目标目录（Windows 目录句柄也可刷写）
    if let Ok(f) = std::fs::File::open(&target_dir) {
        let _ = f.sync_all();
    }

    // 打印目标目录内容用于调试
    eprintln!("📋 [install_plugin] 安装后验证 - 目标目录: {:?}", target_dir);
    eprintln!("   📁 目录存在: {}", target_dir.exists());
    eprintln!("   📄 plugin.json 存在: {}", final_plugin_json.exists());
    if target_dir.exists() {
        match std::fs::read_dir(&target_dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    eprintln!("      - {}", entry.path().display());
                }
            }
            Err(e) => eprintln!("   ❌ 读取目录失败: {}", e),
        }
    }

    // ---- 9. 清理临时目录 ----
    std::fs::remove_dir_all(&temp_dir)
        .map_err(|e| format!("清理临时目录失败: {}", e))?;

    // ---- 10. 成功 ----
    progress(DownloadProgress {
        plugin_id: pid.clone(),
        percentage: 100.0,
        downloaded_bytes: 0,
        total_bytes: 0,
        stage: "done".to_string(),
        message: "安装完成！".to_string(),
    });

    println!(
        "✅ 插件 [{}] 安装完成 -> {}",
        plugin_id,
        target_dir.display()
    );

    Ok(())
}

// ========== 卸载 ==========

/// 卸载插件：删除 plugin.json，再删除整个目录
pub fn uninstall_plugin(plugins_root: &Path, plugin_id: &str) -> Result<(), String> {
    let plugin_dir = plugins_root.join(plugin_id);
    if !plugin_dir.exists() {
        return Err(format!("插件目录不存在: {}", plugin_dir.display()));
    }

    // 先删除 plugin.json
    let plugin_json = plugin_dir.join("plugin.json");
    if plugin_json.exists() {
        std::fs::remove_file(&plugin_json)
            .map_err(|e| format!("删除 plugin.json 失败: {}", e))?;
        println!("🗑️ 已删除 plugin.json: {}", plugin_json.display());
    }

    // 再删除整个目录
    std::fs::remove_dir_all(&plugin_dir)
        .map_err(|e| format!("删除插件目录失败 {}: {}", plugin_dir.display(), e))?;

    println!("🗑️ 插件 [{}] 已卸载，目录已删除: {}", plugin_id, plugin_dir.display());
    Ok(())
}