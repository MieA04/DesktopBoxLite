use std::process::Command;

/// Executes a shell command without showing a console window.
///
/// Supports:
/// - `.exe` executables
/// - `.bat`, `.cmd`, `.ps1` scripts
/// - System commands with arguments (via `cmd /c`)
///
/// Uses `CREATE_NO_WINDOW` flag on Windows to prevent console windows.
/// On failure, logs the error without showing UI popups.
pub fn execute_command(command: &str) -> Result<(), String> {
    if command.is_empty() {
        return Err("Empty command".to_string());
    }

    log::info!("Executing command: {}", command);

    let result = run_shell_command(command);

    match &result {
        Ok(_) => log::info!("Command executed successfully: {}", command),
        Err(e) => log::error!("Command failed '{}': {}", command, e),
    }

    result
}

/// Opens a file, folder, or URL with the system default handler.
/// Uses the `opener` crate which handles this more reliably than shell commands.
pub fn open_with_system_handler(path: &str) -> Result<(), String> {
    log::info!("Opening with system handler: {}", path);
    opener::open(path).map_err(|e| format!("Failed to open '{}': {}", path, e))
}

/// Runs a shell command on Windows using `cmd /c`, creating no window.
#[cfg(target_os = "windows")]
fn run_shell_command(command: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let child = Command::new("cmd")
        .args(["/c", command])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| format!("Failed to spawn process: {}", e))?;

    // Detach — return immediately without waiting for completion
    let _ = child;

    Ok(())
}

/// Runs a shell command on non-Windows platforms.
#[cfg(not(target_os = "windows"))]
fn run_shell_command(command: &str) -> Result<(), String> {
    Command::new("sh")
        .args(["-c", command])
        .spawn()
        .map_err(|e| format!("Failed to spawn process: {}", e))?;

    Ok(())
}
