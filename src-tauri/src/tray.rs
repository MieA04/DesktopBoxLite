use std::sync::Arc;

use tauri::{
    Emitter, Manager,
    menu::{Menu, MenuItem, CheckMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Runtime,
};

/// Builds and returns the system tray menu.
pub fn build_tray_menu<R: Runtime>(app: &AppHandle<R>) -> Result<tauri::tray::TrayIcon<R>, String> {
    let toggle = MenuItem::with_id(app, "toggle", "显示/隐藏", true, None::<&str>)
        .map_err(|e| format!("Failed to create toggle menu item: {}", e))?;
    let reload = MenuItem::with_id(app, "reload", "重载配置", true, None::<&str>)
        .map_err(|e| format!("Failed to create reload menu item: {}", e))?;
    let separator1 = tauri::menu::PredefinedMenuItem::separator(app)
        .map_err(|e| format!("Failed to create separator: {}", e))?;

    // Auto-start toggle — reads current config state for initial checked state
    let config = crate::config::load_config();
    let auto_start = Arc::new(
        CheckMenuItem::with_id(
            app,
            "auto_start",
            "开机自启",
            true,
            config.behavior.auto_start,
            None::<&str>,
        )
        .map_err(|e| format!("Failed to create auto_start menu item: {}", e))?,
    );

    let separator2 = tauri::menu::PredefinedMenuItem::separator(app)
        .map_err(|e| format!("Failed to create separator: {}", e))?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)
        .map_err(|e| format!("Failed to create quit menu item: {}", e))?;

    let menu = Menu::with_items(app, &[&toggle, &reload, &separator1, auto_start.as_ref(), &separator2, &quit])
        .map_err(|e| format!("Failed to create menu: {}", e))?;

    let auto_start_clone = auto_start.clone();

    // Load the Tauri framework logo for the tray icon
    let img = image::load_from_memory(include_bytes!("../icons/tauri-tray.png"))
        .expect("Failed to decode tray icon")
        .to_rgba8();
    let (w, h) = img.dimensions();
    let tray_icon = tauri::image::Image::new_owned(img.into_raw(), w, h);

    let tray = TrayIconBuilder::new()
        .icon(tray_icon)
        .menu(&menu)
        .tooltip("DesktopBox Lite")
        .on_menu_event(move |app, event| {
            match event.id().as_ref() {
                "toggle" => {
                    if let Some(window) = app.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = app.emit("animate-hide", ());
                            let _ = crate::config::save_display_visible(false);
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                            let _ = app.emit("animate-show", ());
                            let _ = crate::config::save_display_visible(true);
                        }
                    }
                }
                "reload" => {
                    log::info!("Reloading configuration");
                    let config = crate::config::load_config();
                    let _ = app.emit("config-reloaded", &config);
                }
                "auto_start" => {
                    let config = crate::config::load_config();
                    let new_state = !config.behavior.auto_start;
                    log::info!("Toggling auto-start: {}", new_state);
                    let _ = crate::desktop::set_auto_start(new_state);
                    let _ = crate::config::save_auto_start(new_state);
                    // Update the checkmark visually via the shared Arc reference
                    let _ = auto_start_clone.set_checked(new_state);
                }
                "quit" => {
                    log::info!("Quitting via tray menu");
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)
        .map_err(|e| format!("Failed to build tray icon: {}", e))?;

    Ok(tray)
}
