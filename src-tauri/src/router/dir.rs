// e:\dev\test-tauri\tauri-app\src-tauri\src\router\dir.rs
//! 目录浏览 API 路由实现
//!
//! 提供 POST /api/dir 接口，用于浏览文件系统目录。

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

use super::RouterState;

// ==========================================
// 🔵 请求/响应数据结构
// ==========================================

#[derive(Debug, Deserialize)]
pub struct DirRequest {
    /// 路径数组，如 ["folder1", "subfolder2"]
    /// 空数组或 None 表示根目录
    pub path: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct FileInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub file_type: String,
    pub path: String,
    pub mtime: String,
}

// ==========================================
// 🟢 路由处理函数
// ==========================================

pub async fn handle_dir_list(
    State(state): State<RouterState>, // ✅ 原来的 State<PathBuf> 替换成这个
    Json(payload): Json<DirRequest>,
) -> impl IntoResponse {
    let root_path = &state.root_path; // ✅ 取实际目录

    let path_array = match payload.path {
        Some(arr) if !arr.is_empty() => arr,
        _ => vec![],
    };

    let requested_path = if !path_array.is_empty() {
        format!("/{}", path_array.join("/"))
    } else {
        "/".to_string()
    };

    let full_path = root_path.join(requested_path.trim_start_matches('/'));

    match tokio::fs::metadata(&full_path).await {
        Ok(metadata) => {
            if metadata.is_dir() {
                match read_directory(&full_path, &requested_path).await {
                    Ok(files) => (StatusCode::OK, Json(files)).into_response(),
                    Err(e) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": format!("无法读取目录: {}", e) })),
                    )
                        .into_response(),
                }
            } else {
                let file_info = FileInfo {
                    name: full_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned(),
                    file_type: "file".to_string(),
                    path: requested_path,
                    mtime: metadata
                        .modified()
                        .ok()
                        .map(format_iso_time)
                        .unwrap_or_default(),
                };
                (StatusCode::OK, Json(file_info)).into_response()
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "文件或目录不存在" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("系统错误: {}", e) })),
        )
            .into_response(),
    }
}

// ==========================================
// 🟡 内部辅助函数
// ==========================================

async fn read_directory(
    dir_path: &PathBuf,
    base_path: &str,
) -> Result<Vec<FileInfo>, std::io::Error> {
    let mut entries = tokio::fs::read_dir(dir_path).await?;
    let mut files = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let metadata = tokio::fs::metadata(&path).await?;

        let file_name = entry.file_name().to_string_lossy().into_owned();

        let relative_path = if base_path == "/" {
            format!("/{}", file_name)
        } else {
            format!("{}/{}", base_path, file_name)
        };

        let mtime_str = metadata
            .modified()
            .ok()
            .map(format_iso_time)
            .unwrap_or_default();

        files.push(FileInfo {
            name: file_name,
            file_type: if metadata.is_dir() {
                "directory".to_string()
            } else {
                "file".to_string()
            },
            path: relative_path,
            mtime: mtime_str,
        });
    }

    Ok(files)
}

fn format_iso_time(time: SystemTime) -> String {
    match time.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => {
            let total_secs = duration.as_secs();
            let millis = duration.subsec_millis();
            let days = total_secs / 86400;
            let secs_of_day = total_secs % 86400;
            let hours = secs_of_day / 3600;
            let minutes = (secs_of_day % 3600) / 60;
            let seconds = secs_of_day % 60;

            let mut year: i32 = 1970;
            let mut remaining_days = days as i64;
            loop {
                let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
                let year_days = if is_leap { 366 } else { 365 };
                if remaining_days >= year_days as i64 {
                    remaining_days -= year_days as i64;
                    year += 1;
                } else {
                    break;
                }
            }

            let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
            let days_in_month = [
                31u8,
                28 + if is_leap { 1 } else { 0 },
                31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
            ];

            let mut month: u8 = 1;
            for &mdays in &days_in_month {
                if remaining_days >= mdays as i64 {
                    remaining_days -= mdays as i64;
                    month += 1;
                } else {
                    break;
                }
            }

            let day = (remaining_days + 1) as u8;
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
                year, month, day, hours, minutes, seconds, millis
            )
        }
        Err(_) => String::from("1970-01-01T00:00:00.000Z"),
    }
}