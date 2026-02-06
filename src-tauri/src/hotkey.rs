<<<<<<< Updated upstream
use tauri::{AppHandle, Manager, Runtime};
=======
use tauri::{AppHandle, Emitter, Runtime};
>>>>>>> Stashed changes
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

#[cfg(target_os = "macos")]
use tauri_nspanel::ManagerExt;

pub struct HotkeyManager {
    current_shortcut: std::sync::Mutex<Option<Shortcut>>,
}

impl HotkeyManager {
    pub fn new() -> Self {
        Self {
            current_shortcut: std::sync::Mutex::new(None),
        }
    }

    pub fn register<R: Runtime>(&self, app: &AppHandle<R>, hotkey: &str) -> Result<(), String> {
        let shortcut: Shortcut = hotkey.parse().map_err(|e| format!("{:?}", e))?;

        // Unregister existing shortcut if any
        self.unregister(app)?;

        let app_clone = app.clone();

        app.global_shortcut()
            .on_shortcut(shortcut.clone(), move |_app, _shortcut, _event| {
                let app = app_clone.clone();
                tauri::async_runtime::spawn(async move {
<<<<<<< Updated upstream
                    // Check if window is already visible
                    let is_visible = crate::window::is_window_visible(app.clone())
                        .await
                        .unwrap_or(false);

                    if is_visible {
                        // Window is visible, just hide it
                        let _ = crate::window::hide_window(app).await;
                    } else {
                        // Window is not visible, show it and start quick-switch mode
                        let _ = crate::window::toggle_window_internal(app.clone(), true).await;

                        // Start quick-switch mode (monitors for V presses and modifier releases)
                        if let Some(input_monitor) =
                            app.try_state::<crate::input_monitor::InputMonitor>()
                        {
                            let app_for_monitor = app.clone();
                            input_monitor.start_quick_switch(app_for_monitor);
                        }
                    }
=======
                    // Check if window is currently hidden (opening mode)
                    let is_opening = {
                        #[cfg(target_os = "macos")]
                        {
                            if let Ok(panel) = app.get_webview_panel(crate::window::MAIN_WINDOW_LABEL) {
                                !panel.is_visible()
                            } else {
                                true
                            }
                        }
                        #[cfg(not(target_os = "macos"))]
                        {
                            true
                        }
                    };

                    // Emit hotkey-mode-started event BEFORE showing window
                    // Only emit when opening (not when closing)
                    if is_opening {
                        let _ = app.emit("hotkey-mode-started", ());
                    }

                    // Toggle window visibility
                    let _ = crate::window::toggle_window(app).await;
>>>>>>> Stashed changes
                });
            })
            .map_err(|e| e.to_string())?;

        *self.current_shortcut.lock().unwrap() = Some(shortcut);

        Ok(())
    }

    pub fn unregister<R: Runtime>(&self, app: &AppHandle<R>) -> Result<(), String> {
        let mut current = self.current_shortcut.lock().unwrap();

        if let Some(shortcut) = current.take() {
            app.global_shortcut()
                .unregister(shortcut)
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}

#[tauri::command]
pub async fn register_hotkey<R: Runtime>(
    app: AppHandle<R>,
    hotkey_manager: tauri::State<'_, HotkeyManager>,
    hotkey: String,
) -> Result<(), String> {
    hotkey_manager.register(&app, &hotkey)
}

#[tauri::command]
pub async fn validate_hotkey(hotkey: String) -> Result<bool, String> {
    // Validate the hotkey format
    let result: Result<Shortcut, _> = hotkey.parse();
    Ok(result.is_ok())
}
