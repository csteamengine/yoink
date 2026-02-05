use crate::settings::SettingsManager;

#[cfg(target_os = "macos")]
pub fn get_frontmost_app() -> Option<String> {
    use std::process::Command;

    let output = Command::new("osascript")
        .args([
            "-e",
            r#"tell application "System Events" to get bundle identifier of first application process whose frontmost is true"#,
        ])
        .output()
        .ok()?;

    if output.status.success() {
        let bundle_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !bundle_id.is_empty() {
            return Some(bundle_id);
        }
    }

    None
}

#[cfg(target_os = "windows")]
pub fn get_frontmost_app() -> Option<String> {
    // On Windows, we'd use the Windows API to get the foreground window
    // For now, return None as a placeholder
    None
}

#[cfg(target_os = "linux")]
pub fn get_frontmost_app() -> Option<String> {
    use std::process::Command;

    // Try using xdotool to get active window
    let output = Command::new("xdotool")
        .args(["getactivewindow", "getwindowname"])
        .output()
        .ok()?;

    if output.status.success() {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !name.is_empty() {
            return Some(name);
        }
    }

    None
}

pub fn is_app_excluded(settings_manager: &SettingsManager) -> bool {
    let settings = settings_manager.get();

    if settings.excluded_apps.is_empty() {
        return false;
    }

    if let Some(app_id) = get_frontmost_app() {
        return settings.excluded_apps.iter().any(|excluded| {
            app_id.to_lowercase().contains(&excluded.to_lowercase())
        });
    }

    false
}

#[tauri::command]
pub async fn get_current_app() -> Result<Option<String>, String> {
    Ok(get_frontmost_app())
}

#[tauri::command]
pub async fn check_app_excluded(
    settings_manager: tauri::State<'_, SettingsManager>,
) -> Result<bool, String> {
    Ok(is_app_excluded(&settings_manager))
}
