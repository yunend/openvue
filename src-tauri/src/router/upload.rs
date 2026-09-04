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

use super::RouterState;

#[derive(Debug, Serialize)]
pub struct UploadedFile {
    /// 保存后的文件名（与原始文件名相同，同名会被覆盖）
    pub filename: String,
    /// 相对公共目录的 URL 路径，如 "/upload/abc.png"
    pub url: String,
    /// 文件字节大小
    pub size: u64,
    /// 原始文件名
    pub original_name: String,
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<UploadedFile>>,
}

/// POST /api/upload 多文件上传（字段名：files）
pub async fn handle_upload(
    State(state): State<RouterState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    // 第二层安全检查：即使路由挂载也再次校验配置
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

        let saved_filename = original_name.clone();
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

        // URL：可通过公共服务访问
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

/// GET /api/upload-status 查询上传功能状态（禁用时也可查询）
pub async fn handle_upload_status(State(state): State<RouterState>) -> impl IntoResponse {

    let upload_dir = state.root_path.join("upload");
    let dir_exists = upload_dir.exists();

    // 遍历目录，统计文件数 + 总大小
    let mut total_files: u64 = 0;
    let mut total_size: u64 = 0;

    if dir_exists {
        if let Ok(mut entries) = tokio::fs::read_dir(&upload_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if let Ok(meta) = tokio::fs::metadata(&path).await {
                    if meta.is_file() {
                        total_files += 1;
                        total_size += meta.len();
                    }
                }
            }
        }
    }

    let size_human = human_readable_size(total_size);

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