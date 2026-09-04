// e:\dev\test-tauri\tauri-app\src-tauri\src\router\mod.rs
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



// ==========================================
// 🔵 共享状态（通过 Axum State 传递给每个 handler）
// ==========================================

/// HTTP 路由共享状态（所有 handler 都可以通过 `State<RouterState>` 取到）
///
#[derive(Clone)]
pub struct RouterState {
    /// 指定文件根目录（同时也是目录浏览 API 的根）
    pub root_path: PathBuf,
    /// 是否启用文件上传功能（双重校验用）
    pub enable_upload: bool,
    /// ✅ App 版本号（来自 Cargo.toml 编译期嵌入，/api/about 接口返回）
    pub app_version: String,
    /// ✅ 当前配置的 HTTP 端口（/api/about 回显用）
    pub config_port: u16,
    /// ✅ 公共目录字符串（/api/about 友好展示，避免再次转 Path）
    pub config_public_folder: String,
    /// ✅ 插件配置（扩展名映射表，文件浏览器 /api/plugins 直接返回）
    pub plugins_config: PluginsConfig,
}

/// ==========================================
/// 🔵 对外公开 API（lib.rs 直接调用这个）
/// ==========================================

/// 创建完整的 HTTP 路由器
///
/// 这是本模块对外暴露的唯一入口。
/// Tauri 命令 start_server() 调用此函数获取 Router。
///
/// # 参数
/// - `root_path`    : 指定文件 / 目录浏览根目录
/// - `enable_upload`: 是否启用文件上传（config.json 中的 enableUpload 字段）
/// - `version`      : App 版本号（来自 Cargo.toml，编译期 env! 宏）
/// - `config_port`  : 当前配置的 HTTP 监听端口（/api/about 回显用）
///
/// # 返回
/// 包含所有 API 路由 + 指定文件服务的完整 Router
pub fn create_router(root_path: PathBuf, enable_upload: bool, version: String, config_port: u16, plugins_config: PluginsConfig) -> Router {
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
    if enable_upload {
        println!(
            "✅ 文件上传已启用，文件将保存到: {}",
            root_path.join("upload").display()
        );
    } else {
        println!("❌ 文件上传已禁用（config.enableUpload = false）");
    }
    let exe_path = std::env::current_exe().expect("无法获取可执行文件路径");
    let exe_dir = exe_path.parent().expect("无法获取可执行文件目录");

    // ---------- 资源目录查找：支持多平台 / 多安装布局 ----------
    // 构建候选资源目录列表，按优先级排序
    let mut resource_candidates: Vec<PathBuf> = Vec::new();

    // ① Windows / macOS / AppImage：exe 同级目录
    resource_candidates.push(exe_dir.to_path_buf());

    // ② 🐧 Linux 多发行版探测（deb/rpm/Arch/Flatpak）
    if cfg!(target_os = "linux") {
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
                resource_candidates.push(PathBuf::from(base).join(name));
            }
            resource_candidates.push(PathBuf::from(base).join("openvue"));
        }
    }

    // ③ 🍎 macOS App Bundle：Contents/Resources（exe 位于 Contents/MacOS）
    if cfg!(target_os = "macos") {
        if let Some(contents_dir) = exe_dir.parent() {
            let resources_dir = contents_dir.join("Resources");
            println!("🍎 [macOS App] 尝试资源目录: {}", resources_dir.display());
            resource_candidates.push(resources_dir);
        }
    }

    // ✅ 遍历候选目录，找到第一个存在 dist-web 或 public 的
    let mut found_base: Option<PathBuf> = None;
    for res_dir in &resource_candidates {
        let dist_web_dir = res_dir.join("dist-web");
        if dist_web_dir.exists() {
            println!("✅ 使用 Vite 构建产物: {:?}", dist_web_dir);
            found_base = Some(dist_web_dir);
            break;
        }
    }
    let base_dir = match found_base {
        Some(dir) => dir,
        None => {
            // 回退：找 public 目录
            let mut fallback = exe_dir.join("public");
            for res_dir in &resource_candidates {
                let public_dir = res_dir.join("public");
                if public_dir.exists() {
                    fallback = public_dir;
                    break;
                }
            }
            println!("⚠️ 未找到 dist-web 构建产物，使用目录: {:?}", fallback);
            fallback
        }
    };
    Router::new()
        .merge(api_routes)
        .with_state(state)
        .nest_service("/public", ServeDir::new(root_path))
        .nest_service(
            "/",
            ServeDir::new(&base_dir)
                .fallback(ServeFile::new(base_dir.join("404.html"))),
        )
        // ✅ CompressionLayer 放最外层（响应时最后进入、最先离开）
        //    charset 中间件放内层（更靠近 ServeDir），确保在压缩之前就把 charset 写进 Content-Type
        .layer(CompressionLayer::new())
        .layer(middleware::from_fn(add_security_headers))
        .layer(middleware::from_fn(add_text_charset_utf8))
        
}

// ==========================================
// 🟢 内部函数（仅本模块使用）
// ==========================================

// ----------------------------------------------------------------
// 🛡️ 中间件：为所有响应添加安全头
//  - Referrer-Policy: same-origin — 防止 Referer 泄露到第三方
//  - Content-Security-Policy     — 防止 XSS / 数据注入攻击
//  - X-Frame-Options: DENY        — 防止点击劫持（Clickjacking）
//  - X-Content-Type-Options: nosniff — 防止 MIME 类型嗅探
// ----------------------------------------------------------------
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
             connect-src 'self' ipc.localhost https://api.github.com; \
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

// ----------------------------------------------------------------
// 🛡️ 中间件：为文本类响应自动追加 charset=utf-8
//   原因：tower_http::ServeDir 默认只返回 "text/plain" "application/json" 等，不带编码。
//         中文 Windows 浏览器若未看到 charset，会默认用 GBK 解码 UTF-8 文件 → 乱码。
// ----------------------------------------------------------------
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

/// 注册所有 API 路由
///
/// # 参数
/// - `enable_upload`：true 时挂载 /api/upload，否则不挂载（第一层安全保障）
///
/// # 返回
/// 只包含 /api/* 路由的 Router（State 类型为 RouterState）
fn register_api_routes(enable_upload: bool) -> Router<RouterState> {
    let mut router = Router::new()
        // -------- 目录浏览 API --------
        .route("/api/dir", axum::routing::post(dir::handle_dir_list))
        // ================================================================
        //   GET /api/about（始终可用，不依赖任何开关）
        //    指定文件首页（index.html）的「关于」按钮 fetch 此接口后显示模态框
        //    返回内容：版本号 + 当前配置（config.json 核心字段）+ 帮助链接
        // ================================================================
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
        // ================================================================
        // /api/upload-status（必须无条件挂载！）
        //   即使enable_upload=false，前端也要能查到"已禁用"的状态
        // ================================================================
        .route(
            "/api/upload-status",
            axum::routing::get(upload::handle_upload_status),
        )
        // ==============================================================
        // ✅ GET /api/plugins（供 public/index.html 的 handleClick 查表用）
        //    返回完整 extensions 映射
        // ==============================================================
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
        

    // -------- 条件挂载：文件上传 API --------
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