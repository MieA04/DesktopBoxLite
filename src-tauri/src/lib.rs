mod config;
mod desktop;
mod executor;
mod hotkey;
mod icon_cache;
mod icon_state;
mod icons;
mod logging;
mod tray;

use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::ShortcutState;

/// Information about a desktop icon sent to the frontend.
#[derive(Debug, Clone, serde::Serialize)]
pub struct IconInfo {
    pub name: String,
    pub path: String,
    pub icon_data: String,
    pub is_shortcut: bool,
    pub click_count: u64,
}

// ── Tauri Commands ──────────────────────────────────────────

/// Lightweight change-detection: compares current desktop state
/// against the stored fingerprint. No icon extraction involved.
#[tauri::command]
fn check_icons_changed(state: tauri::State<'_, icon_state::IconState>) -> Result<bool, String> {
    let current = icons::compute_fingerprint()?;
    let known = state.fingerprint.lock().map_err(|e| e.to_string())?;
    Ok(&current != &*known)
}

/// Triggers a background icon refresh. Returns immediately.
/// The result arrives via the `"icons-ready"` event.
#[tauri::command]
fn refresh_icons(
    app: tauri::AppHandle,
    state: tauri::State<'_, icon_state::IconState>,
) -> Result<(), String> {
    // Update fingerprint first so subsequent polls don't re-trigger
    let fp = icons::compute_fingerprint()?;
    if let Ok(mut f) = state.fingerprint.lock() {
        *f = fp;
    }
    state.request_refresh(app);
    Ok(())
}

/// Returns the cached icon list from the last successful extraction.
/// If no extraction has completed yet, returns an empty list.
#[tauri::command]
fn get_icons() -> Result<Vec<icons::DesktopIcon>, String> {
    // Legacy shim — triggers initial load if state is empty.
    // The real data flow now goes through refresh_icons + "icons-ready" event.
    // This is kept for backward compatibility during transition.
    Ok(Vec::new())
}

#[tauri::command]
fn open_file(path: String) -> Result<(), String> {
    executor::open_with_system_handler(&path)
}

#[tauri::command]
fn get_config() -> config::Config {
    config::load_config()
}

#[tauri::command]
fn save_window_size(width: u32, height: u32) -> Result<(), String> {
    config::save_window_size(width, height)
}

#[tauri::command]
fn execute_custom_command(command: String) -> Result<(), String> {
    executor::execute_command(&command)
}

#[tauri::command]
fn set_auto_start(enabled: bool) -> Result<(), String> {
    desktop::set_auto_start(enabled)?;
    config::save_auto_start(enabled)
}

/// Persists the current window visibility state to config.
/// Called on every show/hide toggle so the next launch restores the same state.
#[tauri::command]
fn set_display_visible(visible: bool) -> Result<(), String> {
    config::save_display_visible(visible)
}

/// Increments the click count for a desktop icon and returns the new count.
/// Called by the frontend whenever a user clicks an icon.
#[tauri::command]
fn increment_click_count(
    path: String,
    state: tauri::State<'_, icon_state::IconState>,
) -> Result<u64, String> {
    log::info!("Incrementing click count for: {}", path);
    state.cache.increment_click_count(std::path::Path::new(&path))
}

/// Called by the frontend after the hide animation completes.
#[tauri::command]
fn finish_hide(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window
            .hide()
            .map_err(|e| format!("Failed to hide window: {}", e))?;
        log::info!("Window hidden after animation");
    }
    Ok(())
}

// ── App Entry Point ─────────────────────────────────────────

pub fn run() {
    logging::init_logging();

    log::info!("Starting DesktopBox Lite");

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // Load configuration
            let config = config::load_config();
            log::info!("Configuration loaded");

            // Register Tauri managed state (icon cache + extraction queue)
            let icon_state = icon_state::IconState::new(&config::config_path());
            app.manage(icon_state);

            // Apply auto-start setting based on config
            if config.behavior.auto_start {
                if let Err(e) = desktop::set_auto_start(true) {
                    log::error!("Failed to enable auto-start: {}", e);
                }
            }

            // Restore window visibility from saved config
            if !config.display.visible {
                if let Some(window) = app.get_webview_window("main") {
                    if let Err(e) = window.hide() {
                        log::error!("Failed to hide window on startup: {}", e);
                    } else {
                        log::info!("Window started hidden (restoring saved state)");
                    }
                }
            }

            // Register toggle window shortcut — filter to Pressed only
            let toggle_hotkey = config.hotkeys.toggle_window.clone();

            if let Err(e) = hotkey::register_shortcut(
                app.handle(),
                &toggle_hotkey,
                move |app, _shortcut, event| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }

                    if let Some(window) = app.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = app.emit("animate-hide", ());
                            let _ = config::save_display_visible(false);
                            log::info!("Hide animation triggered");
                        } else {
                            if let Err(e) = window.show() {
                                log::error!("Failed to show window: {}", e);
                                return;
                            }
                            if let Err(e) = window.set_focus() {
                                log::error!("Failed to focus window: {}", e);
                            }
                            let _ = app.emit("animate-show", ());
                            let _ = config::save_display_visible(true);
                            log::info!("Show animation triggered");
                        }
                    }
                },
            ) {
                log::error!("Failed to register toggle shortcut: {}", e);
            }

            // Register all custom command shortcuts
            for cmd in &config.hotkeys.custom_commands {
                let command = cmd.command.clone();
                let keys_str = cmd.keys.clone();
                let keys_for_closure = keys_str.clone();

                if let Err(e) = hotkey::register_shortcut(
                    app.handle(),
                    &keys_str,
                    move |_app, _shortcut, event| {
                        if event.state != ShortcutState::Pressed {
                            return;
                        }
                        log::info!(
                            "Custom shortcut triggered: {} -> {}",
                            keys_for_closure,
                            command
                        );
                        if let Err(e) = executor::execute_command(&command) {
                            log::error!("Failed to execute command '{}': {}", command, e);
                        }
                    },
                ) {
                    log::warn!(
                        "Failed to register custom shortcut '{}': {}",
                        keys_str,
                        e
                    );
                }
            }

            // Create system tray icon
            match tray::build_tray_menu(app.handle()) {
                Ok(_tray) => log::info!("System tray icon created"),
                Err(e) => log::error!("Failed to create tray icon: {}", e),
            }

            log::info!("DesktopBox Lite initialized successfully");
            Ok(())
        })
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                log::info!("Window close requested, cleaning up");
                let _ = hotkey::unregister_all(&_window.app_handle());
            }
        })
        .invoke_handler(tauri::generate_handler![
            check_icons_changed,
            refresh_icons,
            get_icons,
            open_file,
            get_config,
            save_window_size,
            execute_custom_command,
            set_auto_start,
            finish_hide,
            increment_click_count,
            set_display_visible,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
