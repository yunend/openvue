use tauri::{tray::TrayIconBuilder, menu::Menu, AppHandle, Manager};

/// 🔧 Linux WebKitGTK 渲染修复（Ubuntu/GNOME 下莫名线条、色块、边框残缺）
/// 必须在 Tauri Builder 之前设置，WebKitGTK 初始化后就不能改了
pub fn apply_linux_rendering_fixes() {
    #[cfg(target_os = "linux")]
    {
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        std::env::set_var("GSK_RENDERER", "cairo");
        std::env::set_var("WEBKIT_FORCE_DISK_CACHE", "0");
        println!("🐧 [Linux 渲染修复] 已应用 WebKitGTK 渲染兼容性设置");
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = ();
    }
}

/// 初始化系统托盘
pub fn setup_system_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let show_menu_item = &tauri::menu::MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
    let hide_menu_item = &tauri::menu::MenuItem::with_id(app, "hide", "隐藏到托盘", true, None::<&str>)?;
    let separator = &tauri::menu::PredefinedMenuItem::separator(app)?;
    let quit_menu_item = &tauri::menu::MenuItem::with_id(app, "quit", "退出应用", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[show_menu_item, hide_menu_item, separator, quit_menu_item])?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("Tauri HTTP Server")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main_window(app),
            "hide" => hide_to_tray(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                button_state: tauri::tray::MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

/// 显示主窗口
pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        println!("🪟 [show_main_window] 尝试恢复窗口…");
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        println!("🪟 [show_main_window] 窗口恢复完成");
    }
}

/// 隐藏到托盘
pub fn hide_to_tray(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        println!("🪟 [hide_to_tray] 隐藏窗口到托盘…");
        #[cfg(target_os = "macos")]
        {
            let _ = window.minimize();
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = window.hide();
        }
    }
}