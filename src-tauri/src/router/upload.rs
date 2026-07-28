// e:\dev\test-tauri\tauri-app\src-tauri\src\router\upload.rs
//! 文件上传 API 路由实现
//!
//! 提供 POST /api/upload 接口，接收 multipart/form-data 上传文件，
//! 保存到公共目录下的 upload/ 子目录中。
//!
//! 双重安全保障：
//!   ① 路由创建时：若 enable_upload = false，根本不挂载 /api/upload（返回 404）
//!   ② handler 内部：再次校验 state.enable_upload（防止万一）

use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::RouterState;

// ==========================================
// 🔵 响应数据结构
// ==========================================

#[derive(Debug, Serialize)]
pub struct UploadedFile {
    /// 保存后的文件名（为避免重名，自动加时间戳）
    pub filename: String,
    /// 相对公共目录的 URL 路径，可直接在浏览器访问，如 "/upload/abc_1234567.png"
    pub url: String,
    /// 文件字节大小
    pub size: u64,
    /// 原始文件名（用户上传时的名字）
    pub original_name: String,
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<UploadedFile>>,
}

// ==========================================
// 🟢 路由处理函数
// ==========================================

/// POST /api/upload - 多文件上传
///
/// Content-Type: multipart/form-data
/// 字段名：files（支持一次多个同名字段批量上传）
pub async fn handle_upload(
    State(state): State<RouterState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    // ──── 第二层安全检查：即便路由被挂载，也要再次读配置判断 ────
    if !state.enable_upload {
        return (
            StatusCode::FORBIDDEN,
            Json(UploadResponse {
                success: false,
                message: "❌ 文件上传功能已禁用。请在配置中开启 enableUpload 并重启服务。".to_string(),
                files: None,
            }),
        )
            .into_response();
    }

    // 上传目录：{root_path}/upload  自动创建
    let upload_dir = state.root_path.join("upload");
    if let Err(e) = tokio::fs::create_dir_all(&upload_dir).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(UploadResponse {
                success: false,
                message: format!("创建上传目录失败: {}", e),
                files: None,
            }),
        )
            .into_response();
    }

    let mut uploaded: Vec<UploadedFile> = Vec::new();

    // 遍历 multipart 每个字段
    while let Ok(Some(field)) = multipart.next_field().await {
        let original_name = field.file_name().unwrap_or("unnamed").to_string();
        let data = match field.bytes().await {
            Ok(b) => b,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(UploadResponse {
                        success: false,
                        message: format!("读取上传文件失败: {}", e),
                        files: None,
                    }),
                )
                    .into_response();
            }
        };

        let size = data.len() as u64;

        // 避免重名：文件名 = 原名(无扩展名) + _时间戳.扩展名
        let saved_filename = generate_unique_filename(&upload_dir, &original_name);
        let full_save_path = upload_dir.join(&saved_filename);

        if let Err(e) = tokio::fs::write(&full_save_path, &data).await {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UploadResponse {
                    success: false,
                    message: format!("写入文件 '{}' 失败: {}", saved_filename, e),
                    files: None,
                }),
            )
                .into_response();
        }

        // URL：可直接通过公共服务访问
        let url = format!("/upload/{}", saved_filename);

        uploaded.push(UploadedFile {
            filename: saved_filename,
            url,
            size,
            original_name,
        });
    }

    if uploaded.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(UploadResponse {
                success: false,
                message: "未接收到任何文件字段（请使用字段名 'files' 并选择至少 1 个文件）".to_string(),
                files: None,
            }),
        )
            .into_response();
    }

    let msg = format!("✅ 成功上传 {} 个文件 → {}", uploaded.len(), upload_dir.display());
    println!("{}", msg);

    (
        StatusCode::OK,
        Json(UploadResponse {
            success: true,
            message: msg,
            files: Some(uploaded),
        }),
    )
        .into_response()
}

// ==========================================
// 🟡 内部辅助函数
// ==========================================

/// 生成不冲突的文件名：
///   - 如果磁盘没有同名文件，直接原名
///   - 否则：原名_时间戳毫秒.扩展名
///
/// 例："photo.png" → "photo_1721960000123.png"
fn generate_unique_filename(dir: &Path, original: &str) -> String {
    let original_path = PathBuf::from(original);
    let stem = original_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let ext = original_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let first_try = if ext.is_empty() {
        stem.to_string()
    } else {
        format!("{}.{}", stem, ext)
    };

    // 第一次尝试：没冲突就直接用原名
    if !dir.join(&first_try).exists() {
        return first_try;
    }

    // 冲突 → 加时间戳
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);

    if ext.is_empty() {
        format!("{}_{}", stem, ts)
    } else {
        format!("{}_{}.{}", stem, ts, ext)
    }
}

// ==========================================
// 🔵 上传状态查询
// ==========================================

/// GET /api/upload-status - 查询上传功能状态（不依赖 enable_upload 开关，禁用时也能查）
///
/// 前端用途：
///   - 打开 upload.html 时先调此接口，决定显示上传表单还是「已禁用」提示
///   - 展示已上传文件数 / 总大小，做简易管理
pub async fn handle_upload_status(State(state): State<RouterState>) -> impl IntoResponse {

    let upload_dir = state.root_path.join("upload");
    let dir_exists = upload_dir.exists();

    // ── 遍历目录，统计文件数 + 总大小 ──
    let mut total_files: u64 = 0;
    let mut total_size: u64 = 0;

    if dir_exists {
        if let Ok(mut entries) = tokio::fs::read_dir(&upload_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if let Ok(meta) = tokio::fs::metadata(&path).await {
                    if meta.is_file() {
                        // 只统计根目录下的普通文件（不递归子目录，避免超大目录变慢）
                        total_files += 1;
                        total_size += meta.len();
                    }
                }
            }
        }
    }

    // ── 人类可读大小格式化：B / KB / MB / GB ──
    let size_human = human_readable_size(total_size);

    // ── 提示信息 ──
    let hint = if state.enable_upload {
        "POST 字段名使用 'files'，支持一次多文件（multipart/form-data）".to_string()
    } else {
        "上传功能未启用。请在 config.json 设置 \"enableUpload\": true，重启 HTTP 服务生效。".to_string()
    };

    let example_url = if state.enable_upload {
        "/upload/<filename>".to_string()
    } else {
        String::new()
    };

    let resp = serde_json::json!({
        "enabled": state.enable_upload,
        "uploadDir": upload_dir.to_string_lossy().to_string(),
        "uploadDirExists": dir_exists,
        "totalFiles": total_files,
        "totalSizeBytes": total_size,
        "totalSizeHuman": size_human,
        "exampleAccessUrl": example_url,
        "hint": hint,
    });

    (StatusCode::OK, Json(resp)).into_response()
}

// ==========================================
// 🟡 辅助：字节 → 人类可读
// ==========================================
fn human_readable_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut idx = 0usize;
    while size >= 1024.0 && idx < UNITS.len() - 1 {
        size /= 1024.0;
        idx += 1;
    }
    format!("{:.2} {}", size, UNITS[idx])
}