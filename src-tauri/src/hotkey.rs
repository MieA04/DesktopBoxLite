use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

/// Parsed hotkey components.
pub struct ParsedHotkey {
    pub modifiers: Modifiers,
    pub code: Code,
}

/// Parses a hotkey string like "Ctrl+Shift+D" into modifier bits + key code.
pub fn parse_hotkey(hotkey_str: &str) -> Result<ParsedHotkey, String> {
    let parts: Vec<&str> = hotkey_str.split('+').collect();
    if parts.is_empty() {
        return Err("Empty hotkey string".to_string());
    }

    let mut modifiers = Modifiers::empty();
    let key_part = parts[parts.len() - 1];

    for m in &parts[..parts.len() - 1] {
        match m.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "alt" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            "win" | "super" | "command" | "meta" => modifiers |= Modifiers::SUPER,
            _ => return Err(format!("Unknown modifier: {}", m)),
        }
    }

    let code = parse_key_code(key_part)?;

    Ok(ParsedHotkey { modifiers, code })
}

/// Parses a key string into a Code enum.
fn parse_key_code(key: &str) -> Result<Code, String> {
    match key.to_uppercase().as_str() {
        "A" => Ok(Code::KeyA),
        "B" => Ok(Code::KeyB),
        "C" => Ok(Code::KeyC),
        "D" => Ok(Code::KeyD),
        "E" => Ok(Code::KeyE),
        "F" => Ok(Code::KeyF),
        "G" => Ok(Code::KeyG),
        "H" => Ok(Code::KeyH),
        "I" => Ok(Code::KeyI),
        "J" => Ok(Code::KeyJ),
        "K" => Ok(Code::KeyK),
        "L" => Ok(Code::KeyL),
        "M" => Ok(Code::KeyM),
        "N" => Ok(Code::KeyN),
        "O" => Ok(Code::KeyO),
        "P" => Ok(Code::KeyP),
        "Q" => Ok(Code::KeyQ),
        "R" => Ok(Code::KeyR),
        "S" => Ok(Code::KeyS),
        "T" => Ok(Code::KeyT),
        "U" => Ok(Code::KeyU),
        "V" => Ok(Code::KeyV),
        "W" => Ok(Code::KeyW),
        "X" => Ok(Code::KeyX),
        "Y" => Ok(Code::KeyY),
        "Z" => Ok(Code::KeyZ),
        "0" => Ok(Code::Digit0),
        "1" => Ok(Code::Digit1),
        "2" => Ok(Code::Digit2),
        "3" => Ok(Code::Digit3),
        "4" => Ok(Code::Digit4),
        "5" => Ok(Code::Digit5),
        "6" => Ok(Code::Digit6),
        "7" => Ok(Code::Digit7),
        "8" => Ok(Code::Digit8),
        "9" => Ok(Code::Digit9),
        "F1" => Ok(Code::F1),
        "F2" => Ok(Code::F2),
        "F3" => Ok(Code::F3),
        "F4" => Ok(Code::F4),
        "F5" => Ok(Code::F5),
        "F6" => Ok(Code::F6),
        "F7" => Ok(Code::F7),
        "F8" => Ok(Code::F8),
        "F9" => Ok(Code::F9),
        "F10" => Ok(Code::F10),
        "F11" => Ok(Code::F11),
        "F12" => Ok(Code::F12),
        _ => Err(format!("Unsupported key: {}", key)),
    }
}

/// Registers a global shortcut handler using `on_shortcut`.
pub fn register_shortcut<F>(app: &tauri::AppHandle, hotkey_str: &str, handler: F) -> Result<(), String>
where
    F: Fn(&tauri::AppHandle, &Shortcut, tauri_plugin_global_shortcut::ShortcutEvent) + Send + Sync + 'static,
{
    let parsed = parse_hotkey(hotkey_str)?;
    let shortcut = Shortcut::new(Some(parsed.modifiers), parsed.code);

    app.global_shortcut()
        .on_shortcut(shortcut, handler)
        .map_err(|e| format!("Failed to register shortcut '{}': {}", hotkey_str, e))?;

    log::info!("Registered global shortcut: {}", hotkey_str);
    Ok(())
}

/// Unregisters all global shortcuts for this app.
pub fn unregister_all(app: &tauri::AppHandle) {
    if let Err(e) = app.global_shortcut().unregister_all() {
        log::error!("Failed to unregister shortcuts: {}", e);
    } else {
        log::info!("All global shortcuts unregistered");
    }
}
