mod config;
mod paths;
mod plugins;
mod router;
mod server;
mod tray;
mod commands;

use std::sync::{Arc, Mutex};
use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // ⚠️ 务必放在所有 Tauri/WebKit 初始化之前！
    tray::apply_linux_rendering_fixes();

    let app_config = config::load_config(None).expect("加载配置失败");
    config::validate_config(&app_config).expect("配置验证失败");

    let builtin_dir = paths::builtin_plugins_dir().expect("获取内置插件目录失败");
    let user_dir = paths::resolve_plugins_dir(app_config.plugins_folder.as_deref())
        .expect("获取用户插件目录失败");
    let plugins_state = plugins::load_plugins_state().unwrap_or_default();
    let plugins_config = plugins::scan_and_build_config(&builtin_dir, &user_dir, &plugins_state);

    let server_state = Arc::new(Mutex::new(server::ServerState {
        cancel_token: None,
        app_config: app_config.clone(),
        plugins_config: plugins_config.clone(),
    }));

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--hide-to-tray"]),
        ))
        .plugin(tauri_plugin_dialog::init())
        .manage(server_state.clone())
        .setup(|app| {
            tray::setup_system_tray(app)?;

            // Rust 后端自动启动 HTTP 服务（不依赖前端 WebView，开机自启时可靠）
            let state = app.state::<Arc<Mutex<server::ServerState>>>();
            server::auto_start_server_if_needed(&state);

            let args: Vec<String> = std::env::args().collect();
            if args.contains(&"--hide-to-tray".to_string()) {
                // macOS 用 minimize() 代替 hide()，确保窗口可恢复
                tray::hide_to_tray(app.handle());
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_server,
            commands::stop_server,
            commands::show_window,
            commands::hide_window,
            commands::quit_app,
            commands::get_config,
            commands::save_config,
            commands::get_server_status,
            commands::get_version,
            commands::choose_folder,
            commands::get_plugins_config,
            commands::activate_plugin_handler,
            commands::set_plugin_browser_default,
            commands::add_custom_plugin,
            commands::remove_custom_plugin,
            commands::get_plugins_dir,
            commands::open_url,
            commands::set_plugins_folder,
            commands::reset_plugins_folder,
            commands::open_plugins_folder,
            commands::get_preferred_download_source,
            commands::set_preferred_download_source,
            commands::install_plugin_from_market,
            commands::fetch_plugins_index,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}