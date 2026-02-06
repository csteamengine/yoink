use std::sync::Mutex;

/// Stores the previously focused application for paste-back functionality
pub struct PreviousAppState {
    bundle_id: Mutex<Option<String>>,
}

impl PreviousAppState {
    pub fn new() -> Self {
        Self {
            bundle_id: Mutex::new(None),
        }
    }

    /// Save the currently focused app (call before showing Yoink window)
    pub fn save_previous_app(&self) {
        if let Some(app_id) = get_frontmost_app() {
            // Don't save Yoink itself as the previous app
            if !app_id.contains("yoink") {
                log::info!("Saved previous app: {}", app_id);
                *self.bundle_id.lock().unwrap() = Some(app_id);
            }
        }
    }

    /// Get the saved previous app bundle ID
    pub fn get_previous_app(&self) -> Option<String> {
        self.bundle_id.lock().unwrap().clone()
    }

    /// Clear the saved previous app
    pub fn clear(&self) {
        *self.bundle_id.lock().unwrap() = None;
    }
}

/// Get the bundle identifier of the frontmost application
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

#[cfg(not(target_os = "macos"))]
pub fn get_frontmost_app() -> Option<String> {
    None
}

/// Activate an application by its bundle identifier
#[cfg(target_os = "macos")]
pub fn activate_app(bundle_id: &str) -> Result<(), String> {
    use std::process::Command;

    let script = format!(r#"tell application id "{}" to activate"#, bundle_id);

    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        log::info!("Activated app: {}", bundle_id);
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr).to_string();
        log::warn!("Failed to activate app {}: {}", bundle_id, error);
        Err(error)
    }
}

#[cfg(not(target_os = "macos"))]
pub fn activate_app(_bundle_id: &str) -> Result<(), String> {
    // Not supported on other platforms yet
    Ok(())
}

/// Simulate a Cmd+V keystroke to paste
#[cfg(target_os = "macos")]
pub fn simulate_paste() -> Result<(), String> {
    use std::process::Command;

    let script = r#"tell application "System Events" to keystroke "v" using command down"#;

    let output = Command::new("osascript")
        .args(["-e", script])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        log::info!("Simulated Cmd+V paste");
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr).to_string();
        log::warn!("Failed to simulate paste: {}", error);
        Err(error)
    }
}

#[cfg(not(target_os = "macos"))]
pub fn simulate_paste() -> Result<(), String> {
    // Not supported on other platforms yet
    Ok(())
}

/// Perform the full paste-back operation: activate previous app and simulate Cmd+V
#[cfg(target_os = "macos")]
pub async fn paste_to_previous_app(previous_app: &PreviousAppState) -> Result<(), String> {
    if let Some(bundle_id) = previous_app.get_previous_app() {
        // Activate the previous app
        activate_app(&bundle_id)?;

        // Small delay to ensure the app is focused
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Simulate Cmd+V
        simulate_paste()?;

        Ok(())
    } else {
        log::warn!("No previous app saved, skipping paste-back");
        Ok(())
    }
}

#[cfg(not(target_os = "macos"))]
pub async fn paste_to_previous_app(_previous_app: &PreviousAppState) -> Result<(), String> {
    Ok(())
}
