// e:\dev\test-tauri\tauri-app\src-tauri\src\router\mod.rs
//! Router 模块 - 所有 HTTP 路由的统一入口
//!
//! 模块结构：
//! - dir:    目录浏览 API
//! - upload: 文件上传 API

pub mod dir;
pub mod upload;

use axum::Router;
use std::path::PathBuf;
use tower_http::services::ServeDir;
use axum::extract::State;
use crate::plugins::PluginsConfig;

// ==========================================
// 🔵 共享状态（通过 Axum State 传递给每个 handler）
// ==========================================

/// HTTP 路由共享状态（所有 handler 都可以通过 `State<RouterState>` 取到）
///
#[derive(Clone)]
pub struct RouterState {
    /// 公共文件根目录（同时也是目录浏览 API 的根）
    pub root_path: PathBuf,
    /// 是否启用文件上传功能（双重校验用）
    pub enable_upload: bool,
    /// ✅ App 版本号（来自 Cargo.toml 编译期嵌入，/api/about 接口返回）
    pub app_version: String,
    /// ✅ 当前配置的 HTTP 端口（/api/about 回显用）
    pub config_port: u16,
    /// ✅ 公共目录字符串（/api/about 友好展示，避免再次转 Path）
    pub config_static_folder: String,
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
/// - `root_path`    : 公共文件 / 目录浏览根目录
/// - `enable_upload`: 是否启用文件上传（config.json 中的 enableUpload 字段）
/// - `version`      : App 版本号（来自 Cargo.toml，编译期 env! 宏）
/// - `config_port`  : 当前配置的 HTTP 监听端口（/api/about 回显用）
///
/// # 返回
/// 包含所有 API 路由 + 公共文件服务的完整 Router
pub fn create_router(root_path: PathBuf, enable_upload: bool, version: String, config_port: u16, plugins_config: PluginsConfig) -> Router {
    let state = RouterState {
        config_static_folder: root_path.to_string_lossy().replace('\\', "/"),
        root_path: root_path.clone(),
        enable_upload,
        app_version: version,
        config_port,
        plugins_config,
    };

    let api_routes = register_api_routes(enable_upload);

    println!("📂 公共文件目录: {}", root_path.display());
    if enable_upload {
        println!(
            "✅ 文件上传已启用，文件将保存到: {}",
            root_path.join("uploads").display()
        );
    } else {
        println!("❌ 文件上传已禁用（config.enableUpload = false）");
    }
    let exe_path = std::env::current_exe().expect("无法获取可执行文件路径");
    let exe_dir = exe_path.parent().expect("无法获取可执行文件目录");
    let static_dir = exe_dir.join("static");
    Router::new()
        .merge(api_routes)
        .with_state(state) 
        .nest_service("/public", ServeDir::new(root_path))
        .nest_service("/", ServeDir::new(&static_dir))
        
}

// ==========================================
// 🟢 内部函数（仅本模块使用）
// ==========================================

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
        //    公共文件首页（index.html）的「关于」按钮 fetch 此接口后显示模态框
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
                            "staticFolder": s.config_static_folder,
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
        // ✅ GET /api/plugins（供 static/index.html 的 handleClick 查表用）
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