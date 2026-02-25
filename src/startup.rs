//! Task Scheduler integration for "Run on startup" functionality.

use std::os::windows::process::CommandExt;
use std::process::Command;

/// Task name in Windows Task Scheduler.
const TASK_NAME: &str = "LangLock";

/// Flag to hide console window.
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Checks if the startup task exists in Task Scheduler.
///
/// # Returns
/// `true` if the task exists, `false` otherwise.
pub fn is_startup_enabled() -> bool {
    let output = Command::new("schtasks")
        .args(["/query", "/tn", TASK_NAME])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    match output {
        Ok(result) => result.status.success(),
        Err(_) => false,
    }
}

/// Enables the "Run on startup" feature by creating a scheduled task.
/// This will trigger a UAC prompt since creating tasks with highest privileges requires admin.
///
/// # Returns
/// `Ok(())` on success, or an error message.
pub fn enable_startup() -> Result<(), String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;

    let exe_path_str = exe_path
        .to_str()
        .ok_or("Executable path contains invalid UTF-8")?;

    // Use PowerShell Start-Process with -Verb RunAs to trigger UAC
    let status = Command::new("powershell")
        .args([
            "-WindowStyle", "Hidden",
            "-Command",
            &format!(
                "Start-Process -FilePath 'schtasks' -ArgumentList '/create /tn \"{}\" /tr \"\\\"{}\\\"\" /sc onlogon /rl highest /f' -Verb RunAs -Wait -WindowStyle Hidden",
                TASK_NAME, exe_path_str
            ),
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("Failed to execute PowerShell: {}", e))?;

    // Give it a moment to complete and check if task was created
    std::thread::sleep(std::time::Duration::from_millis(500));

    if is_startup_enabled() {
        Ok(())
    } else {
        Err("Failed to create scheduled task (UAC cancelled or error)".to_string())
    }
}

/// Disables the "Run on startup" feature by deleting the scheduled task.
/// This will trigger a UAC prompt.
///
/// # Returns
/// `Ok(())` on success, or an error message.
pub fn disable_startup() -> Result<(), String> {
    // Use PowerShell Start-Process with -Verb RunAs to trigger UAC
    let status = Command::new("powershell")
        .args([
            "-WindowStyle", "Hidden",
            "-Command",
            &format!(
                "Start-Process -FilePath 'schtasks' -ArgumentList '/delete /tn \"{}\" /f' -Verb RunAs -Wait -WindowStyle Hidden",
                TASK_NAME
            ),
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("Failed to execute PowerShell: {}", e))?;

    // Give it a moment to complete and check if task was deleted
    std::thread::sleep(std::time::Duration::from_millis(500));

    if !is_startup_enabled() {
        Ok(())
    } else {
        Err("Failed to delete scheduled task (UAC cancelled or error)".to_string())
    }
}
