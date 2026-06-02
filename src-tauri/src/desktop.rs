/// Windows system integration (registry auto-start only).
///
/// Note: Desktop icon hide/show has been removed per PM re-evaluation.
/// DesktopBox Lite does not interfere with native desktop icons.

/// Enables or disables auto-start via Windows registry.
#[cfg(target_os = "windows")]
pub fn set_auto_start(enable: bool) -> Result<(), String> {
    use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
    use windows_sys::Win32::System::Registry::*;

    let app_name = "DesktopBox Lite";
    let app_path =
        std::env::current_exe().map_err(|e| format!("Failed to get executable path: {}", e))?;
    let app_path_str = app_path.to_str().ok_or("Invalid executable path")?;

    let key: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Run\0"
        .encode_utf16()
        .collect();
    let value_name: Vec<u16> = app_name.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        let mut hkey: HKEY = std::ptr::null_mut();
        let result =
            RegOpenKeyExW(HKEY_CURRENT_USER, key.as_ptr(), 0, KEY_SET_VALUE, &mut hkey);

        if result != ERROR_SUCCESS {
            return Err(format!("Failed to open registry key: {}", result));
        }

        if enable {
            let value: Vec<u16> = app_path_str.encode_utf16().chain(std::iter::once(0)).collect();
            let result = RegSetValueExW(
                hkey,
                value_name.as_ptr(),
                0,
                REG_SZ,
                value.as_ptr() as *const u8,
                (value.len() * 2) as u32,
            );
            if result != ERROR_SUCCESS {
                let _ = RegCloseKey(hkey);
                return Err(format!("Failed to set registry value: {}", result));
            }
        } else {
            let result = RegDeleteValueW(hkey, value_name.as_ptr());
            if result != ERROR_SUCCESS && result != ERROR_FILE_NOT_FOUND as u32 {
                let _ = RegCloseKey(hkey);
                return Err(format!("Failed to delete registry value: {}", result));
            }
        }

        RegCloseKey(hkey);
    }

    log::info!(
        "Auto-start {} for DesktopBox Lite",
        if enable { "enabled" } else { "disabled" }
    );
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn set_auto_start(_enable: bool) -> Result<(), String> {
    log::warn!("Auto-start is only supported on Windows");
    Ok(())
}
