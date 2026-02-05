use tauri::{AppHandle, Manager, Runtime, WebviewWindow};

pub struct WindowManager;

impl WindowManager {
    pub fn new() -> Self {
        Self
    }

    #[cfg(target_os = "macos")]
    pub fn setup_panel<R: Runtime>(&self, window: &WebviewWindow<R>) -> Result<(), String> {
        use tauri_nspanel::cocoa::appkit::NSWindowCollectionBehavior;
        use tauri_nspanel::WebviewWindowExt as NSPanelWindowExt;

        // Convert window to panel
        let panel = window.to_panel().map_err(|e| e.to_string())?;

        // Configure panel behavior
        panel.set_level(tauri_nspanel::cocoa::appkit::NSMainMenuWindowLevel + 1);

        panel.set_collection_behaviour(
            NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary,
        );

        // Set up delegate for auto-hide on blur
        // Note: This requires additional delegate setup which is complex
        // For now, we handle blur via frontend events

        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    pub fn setup_panel<R: Runtime>(&self, _window: &WebviewWindow<R>) -> Result<(), String> {
        // On non-macOS platforms, we use a standard transparent window
        Ok(())
    }
}

#[tauri::command]
pub async fn show_window<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        // Center on the current monitor
        if let Some(monitor) = window.current_monitor().map_err(|e| e.to_string())? {
            let monitor_size = monitor.size();
            let monitor_pos = monitor.position();
            let window_size = window.outer_size().map_err(|e| e.to_string())?;

            let x = monitor_pos.x + (monitor_size.width as i32 - window_size.width as i32) / 2;
            let y = monitor_pos.y + (monitor_size.height as i32 / 4); // Position in upper third

            window
                .set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }))
                .map_err(|e| e.to_string())?;
        }

        #[cfg(target_os = "macos")]
        {
            use tauri_nspanel::ManagerExt;
            if let Ok(panel) = app.get_webview_panel("main") {
                panel.show();
            } else {
                window.show().map_err(|e| e.to_string())?;
                window.set_focus().map_err(|e| e.to_string())?;
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn hide_window<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        #[cfg(target_os = "macos")]
        {
            use tauri_nspanel::ManagerExt;
            if let Ok(panel) = app.get_webview_panel("main") {
                panel.order_out(None);
            } else {
                window.hide().map_err(|e| e.to_string())?;
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            window.hide().map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn toggle_window<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let is_visible = window.is_visible().map_err(|e| e.to_string())?;

        if is_visible {
            hide_window(app).await
        } else {
            show_window(app).await
        }
    } else {
        Err("Main window not found".to_string())
    }
}

#[tauri::command]
pub async fn is_window_visible<R: Runtime>(app: AppHandle<R>) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("main") {
        window.is_visible().map_err(|e| e.to_string())
    } else {
        Ok(false)
    }
}
