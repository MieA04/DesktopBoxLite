use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a single custom hotkey command mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomCommand {
    pub keys: String,
    pub command: String,
    pub description: Option<String>,
}

/// Hotkey configuration section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub toggle_window: String,
    pub custom_commands: Vec<CustomCommand>,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            toggle_window: "Ctrl+Shift+D".to_string(),
            custom_commands: vec![
                CustomCommand {
                    keys: "Ctrl+Alt+C".to_string(),
                    command: "calc.exe".to_string(),
                    description: Some("打开计算器".to_string()),
                },
                CustomCommand {
                    keys: "Ctrl+Alt+N".to_string(),
                    command: "notepad.exe".to_string(),
                    description: Some("打开记事本".to_string()),
                },
                CustomCommand {
                    keys: "Ctrl+Alt+T".to_string(),
                    command: "cmd.exe /c echo Hello > C:\\test.txt".to_string(),
                    description: Some("执行系统指令".to_string()),
                },
                CustomCommand {
                    keys: "Ctrl+Shift+F".to_string(),
                    command: resolve_wt_path(),
                    description: Some("打开 Windows Terminal".to_string()),
                },
            ],
        }
    }
}

/// Searches for the Windows Terminal executable path.
/// Tries `%LOCALAPPDATA%\Microsoft\WindowsApps\wt.exe` first,
/// then falls back to `"wt.exe"` (relies on PATH / App Execution Alias).
fn resolve_wt_path() -> String {
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let wt_path = PathBuf::from(&local_app_data)
            .join("Microsoft")
            .join("WindowsApps")
            .join("wt.exe");
        if wt_path.exists() {
            log::info!("Windows Terminal found at: {:?}", wt_path);
            return wt_path.to_string_lossy().to_string();
        }
    }
    log::info!("Windows Terminal not found via LOCALAPPDATA, falling back to PATH lookup");
    "wt.exe".to_string()
}

/// Appearance configuration section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    pub css_path: Option<String>,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self { css_path: None }
    }
}

/// Behavior configuration section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    pub auto_start: bool,
    pub icon_refresh_interval_ms: u64,
    pub window_width: u32,
    pub window_height: u32,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            auto_start: false,
            icon_refresh_interval_ms: 500,
            window_width: 800,
            window_height: 600,
        }
    }
}

/// Root configuration structure matching config.json schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub hotkeys: HotkeyConfig,
    pub appearance: AppearanceConfig,
    pub behavior: BehaviorConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkeys: HotkeyConfig::default(),
            appearance: AppearanceConfig::default(),
            behavior: BehaviorConfig::default(),
        }
    }
}

/// Returns the path to the config.json file.
/// Located in the same directory as the executable.
pub fn config_path() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join("config.json")
}

/// Loads configuration from disk. Returns default config on failure.
pub fn load_config() -> Config {
    let path = config_path();
    if !path.exists() {
        log::info!("Config file not found at {:?}, using defaults", path);
        let default_config = Config::default();
        if let Err(e) = save_config(&default_config) {
            log::error!("Failed to save default config: {}", e);
        }
        return default_config;
    }

    let mut config = match std::fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<Config>(&content) {
            Ok(config) => {
                log::info!("Config loaded from {:?}", path);
                config
            }
            Err(e) => {
                log::error!(
                    "Failed to parse config file (falling back to defaults): {}",
                    e
                );
                return Config::default();
            }
        },
        Err(e) => {
            log::error!(
                "Failed to read config file (falling back to defaults): {}",
                e
            );
            return Config::default();
        }
    };

    // ── Migration: update terminal shortcut ───────────────────────────
    // Replace old Ctrl+Shift+R binding (pointing to a non-existent script)
    // with Ctrl+Shift+F → resolved wt.exe path.
    let mut migrated = false;
    let resolved_wt = resolve_wt_path();

    for cmd in &mut config.hotkeys.custom_commands {
        // Migrate: old script path → Windows Terminal
        if cmd.command.contains("my_script.bat")
            || cmd.command.contains("\\scripts\\")
            || (cmd.keys == "Ctrl+Shift+R" && cmd.command == "C:\\scripts\\my_script.bat")
        {
            cmd.keys = "Ctrl+Shift+F".to_string();
            cmd.command = resolved_wt.clone();
            cmd.description = Some("打开 Windows Terminal".to_string());
            migrated = true;
            log::info!("Migrated terminal shortcut: Ctrl+Shift+R → Ctrl+Shift+F");
            break;
        }

        // Migrate: existing wt.exe binding, ensure full path resolution
        if cmd.keys == "Ctrl+Shift+F"
            && (cmd.command == "wt.exe" || cmd.command == resolved_wt)
            && cmd.command != resolved_wt
        {
            cmd.command = resolved_wt.clone();
            cmd.description = Some("打开 Windows Terminal".to_string());
            migrated = true;
            log::info!("Resolved Windows Terminal path: {}", resolved_wt);
            break;
        }
    }

    // Save if migrated
    if migrated {
        if let Err(e) = save_config(&config) {
            log::warn!("Failed to save migrated config: {}", e);
        }
    }

    config
}

/// Saves configuration to disk.
pub fn save_config(config: &Config) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config dir: {}", e))?;
    }
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    std::fs::write(&path, content).map_err(|e| format!("Failed to write config: {}", e))?;
    log::info!("Config saved to {:?}", path);
    Ok(())
}

/// Saves window size to the config file.
pub fn save_window_size(width: u32, height: u32) -> Result<(), String> {
    let mut config = load_config();
    config.behavior.window_width = width;
    config.behavior.window_height = height;
    save_config(&config)
}
