//! Router 模块 - 所有 HTTP 路由的统一入口
//!
//! 模块结构：
//! - dir:    目录浏览 API
//! - upload: 文件上传 API

pub mod dir;
pub mod upload;

use axum::Router;
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::extract::Request;
use axum::http::header::{CONTENT_TYPE, CONTENT_SECURITY_POLICY, REFERRER_POLICY, X_FRAME_OPTIONS};
use axum::http::HeaderValue;
use std::path::PathBuf;
use tower_http::services::ServeDir;
use tower_http::services::ServeFile;
use tower_http::compression::CompressionLayer;
use axum::extract::State;
use crate::plugins::PluginsConfig;

/// HTTP 路由共享状态
#[derive(Clone)]
pub struct RouterState {
    /// 指定文件根目录
    pub root_path: PathBuf,
    /// 是否启用文件上传
    pub enable_upload: bool,
    /// App 版本号（/api/about 返回）
    pub app_version: String,
    /// 当前 HTTP 端口（/api/about 回显）
    pub config_port: u16,
    /// 公共目录字符串
    pub config_public_folder: String,
    /// 插件配置
    pub plugins_config: PluginsConfig,
}

/// 创建完整的 HTTP 路由器（模块对外唯一入口）
pub fn create_router(
    root_path: PathBuf,
    enable_upload: bool,
    version: String,
    config_port: u16,
    plugins_config: PluginsConfig,
    plugins_folder: PathBuf,
) -> Router {
    let state = RouterState {
        config_public_folder: root_path.to_string_lossy().replace('\\', "/"),
        root_path: root_path.clone(),
        enable_upload,
        app_version: version,
        config_port,
        plugins_config,
    };

    let api_routes = register_api_routes(enable_upload);

    println!("📂 指定文件目录: {}", root_path.display());
    println!("📦 自定义插件目录: {}", plugins_folder.display());
    if enable_upload {
        println!(
            "✅ 文件上传已启用，文件将保存到: {}",
            root_path.join("upload").display()
        );
    } else {
        println!("❌ 文件上传已禁用（config.enableUpload = false）");
    }
    // 资源目录查找：委托给 paths::find_app_root()
    let app_root = crate::paths::find_app_root().expect("无法定位应用资源根目录");

    let dist_web_dir = app_root.join("dist-web");
    let base_dir = if dist_web_dir.exists() {
        println!("✅ 使用 Vite 构建产物: {}", dist_web_dir.display());
        dist_web_dir
    } else {
        let fallback = app_root.join("public");
        println!("⚠️ 未找到 dist-web 构建产物，使用目录: {}", fallback.display());
        fallback
    };
    // 🔀 路由注册顺序很重要：/pfolder /plugins /public 等前缀路由必须在根 / 之前
    let builtin_plugins_dir = crate::paths::builtin_plugins_dir()
        .unwrap_or_else(|_| base_dir.join("plugins"));
    Router::new()
        .merge(api_routes)
        .with_state(state)
        // 🆕 /pfolder —— 用户自定义可写插件目录（与 /plugins 内置前缀相异）
        .nest_service(
            "/pfolder",
            ServeDir::new(&plugins_folder)
                .fallback(ServeFile::new(base_dir.join("404.html"))),
        )
        // /plugins —— 内置只读插件目录（资源目录）
        .nest_service(
            "/plugins",
            ServeDir::new(&builtin_plugins_dir)
                .fallback(ServeFile::new(base_dir.join("404.html"))),
        )
        .nest_service(
            "/public",
            ServeDir::new(&root_path)
                .fallback(ServeFile::new(base_dir.join("404.html"))),
        )
        .nest_service(
            "/",
            ServeDir::new(&base_dir)
                .fallback(ServeFile::new(base_dir.join("404.html"))),
        )
        .layer(CompressionLayer::new())
        .layer(middleware::from_fn(add_security_headers))
        .layer(middleware::from_fn(add_text_charset_utf8))
}

/// 中间件：为所有响应添加安全头（CSP / X-Frame-Options / nosniff / Referrer-Policy）
async fn add_security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        REFERRER_POLICY,
        HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline' 'unsafe-eval' data: blob:; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data: blob:; \
             font-src 'self' data:; \
             connect-src *; \
             frame-src 'self' https://*.officeapps.live.com https://*.wopi.net; \
             worker-src 'self' data: blob:; \
             frame-ancestors 'none';"
        ),
    );
    headers.insert(
        X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    );
    headers.insert(
        axum::http::header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    response
}

/// 中间件：为文本类响应自动追加 charset=utf-8（避免中文 Windows 浏览器 GBK 乱码）
async fn add_text_charset_utf8(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    if let Some(content_type) = response.headers().get(CONTENT_TYPE).cloned() {
        if let Ok(ct_str) = content_type.to_str() {
            let lower = ct_str.to_ascii_lowercase();
            // 尚未设置 charset 时才追加（避免重复）
            if !lower.contains("charset") {
                let needs_utf8 = lower.starts_with("text/")
                    || lower.starts_with("application/json")
                    || lower.starts_with("application/javascript")
                    || lower.starts_with("application/ecmascript")
                    || lower.starts_with("application/xml")
                    || lower.starts_with("application/xhtml")
                    || lower.starts_with("application/csv")
                    || lower.starts_with("image/svg"); // SVG 是 XML，需要 UTF-8
                if needs_utf8 {
                    if let Ok(new_ct) = HeaderValue::from_str(&format!("{}; charset=utf-8", ct_str)) {
                        response.headers_mut().insert(CONTENT_TYPE, new_ct);
                    }
                }
            }
        }
    }
    response
}

/// 注册所有 /api/* 路由（enable_upload=false 时 /upload 返回 403）
fn register_api_routes(enable_upload: bool) -> Router<RouterState> {
    let mut router = Router::new()
        // 目录浏览 API
        .route("/api/dir", axum::routing::post(dir::handle_dir_list))
        // /api/about（始终可用）
        .route(
            "/api/about",
            axum::routing::get(|State(s): State<RouterState>| async move {
                use axum::Json;
                (
                    axum::http::StatusCode::OK,
                    Json(serde_json::json!({
                        "version": s.app_version,
                        "buildStack": "Tauri 2.x + Axum (Rust) + Vue 3 + Tailwind CSS",
                        "config": {
                            "port": s.config_port,
                            "publicFolder": s.config_public_folder,
                            "enableUpload": s.enable_upload,
                        },
                        "helpLinks": [
                            { "label": "🌐 Tauri 官方网站",  "url": "https://tauri.app" },
                            { "label": "📘 Tauri 中文文档",  "url": "https://tauri.app/zh-cn/" },
                            { "label": "🦀 Axum 官方文档",  "url": "https://docs.rs/axum" },
                            { "label": "💚 Vue 3 中文文档", "url": "https://cn.vuejs.org" },
                            { "label": "🎨 Tailwind 中文站", "url": "https://www.tailwindcss.cn" },
                            { "label": "🐙 GitHub 代码仓库", "url": "https://github.com/yunend/openvue" },
                        ]
                    })),
                )
            }),
        )
        // /api/upload-status（无条件挂载，禁用时也能查询）
        .route(
            "/api/upload-status",
            axum::routing::get(upload::handle_upload_status),
        )
        // /api/plugins（返回完整 extensions 映射）
        .route(
            "/api/plugins",
            axum::routing::get(|State(s): State<RouterState>| async move {
                use axum::Json;
                (
                    axum::http::StatusCode::OK,
                    Json(serde_json::json!({ "extensions": s.plugins_config.extensions })),
                )
            }),
        );

    // 条件挂载文件上传 API
    if enable_upload {
        router = router.route("/upload", axum::routing::post(upload::handle_upload));
    } else {
        router = router.route(
            "/upload",
            axum::routing::post(|| async {
                (
                    axum::http::StatusCode::FORBIDDEN,
                    axum::Json(serde_json::json!({
                        "success": false,
                        "message": "❌ 文件上传功能未启用。请在 config.json 中设置 \"enableUpload\": true 并重启 HTTP 服务。"
                    })),
                )
            }),
        );
    }

    router
}