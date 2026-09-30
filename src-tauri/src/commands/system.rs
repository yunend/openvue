//! 系统 / 工具命令

use std::path::{Path, PathBuf};
use std::cell::RefCell;

#[tauri::command]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
pub async fn choose_folder(
    app: tauri::AppHandle,
    initial_dir: Option<String>,
    title: Option<String>,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let (tx, rx) = tokio::sync::oneshot::channel::<Option<PathBuf>>();
    let tx_cell = RefCell::new(Some(tx));

    let mut builder = app.dialog().file();
    let dialog_title = title.unwrap_or_else(|| "选择文件夹".to_string());
    builder = builder.set_title(&dialog_title);

    if let Some(dir_str) = initial_dir {
        let raw = PathBuf::from(&dir_str);
        let resolved = if raw.is_absolute() {
            raw.clone()
        } else {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(&raw)
        };
        let exists = resolved.exists();
        let canonical_raw = resolved.canonicalize().unwrap_or_else(|_| resolved.clone());
        let canonical = normalize_path_for_dialog(&canonical_raw);
        if exists {
            builder = builder.set_directory(&canonical);
        }
    }

    builder.pick_folder(move |fp_opt| {
        let path_opt: Option<PathBuf> = fp_opt.and_then(|fp| {
            fp.as_path().map(|p: &Path| p.to_path_buf())
        });
        if let Some(tx) = tx_cell.borrow_mut().take() {
            let _ = tx.send(path_opt);
        }
    });

    let result = rx.await.map_err(|e| format!("等待对话框失败: {}", e))?;

    match result {
        Some(pb) => Ok(Some(pb.to_string_lossy().replace('\\', "/"))),
        None => Ok(None)
    }
}

#[tauri::command]
pub async fn open_url(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| e.to_string())
}

/// 清理 Windows canonicalize 产生的 \\?\ UNC 前缀
fn normalize_path_for_dialog(p: &Path) -> PathBuf {
    let s = p.to_string_lossy().to_string();
    let cleaned = if let Some(rest) = s.strip_prefix(r"\\?\") {
        rest.to_string()
    } else if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{}", rest)
    } else {
        s
    };
    PathBuf::from(cleaned)
}