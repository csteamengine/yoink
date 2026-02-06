use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[cfg(target_os = "macos")]
use tauri_nspanel::ManagerExt;

use crate::window::HotkeyModeState;

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
            .on_shortcut(shortcut.clone(), move |_app, _shortcut, event| {
                // Only handle key press, not key release
                if event.state != ShortcutState::Pressed {
                    return;
                }

                let app = app_clone.clone();
                tauri::async_runtime::spawn(async move {
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

                    // Enter hotkey mode and emit event BEFORE showing window
                    // Only when opening (not when closing)
                    if is_opening {
                        // Enter backend hotkey mode to prevent auto-hide while modifiers held
                        if let Some(hotkey_state) = app.try_state::<HotkeyModeState>() {
                            hotkey_state.enter();
                        }
                        let _ = app.emit("hotkey-mode-started", ());
                    }

                    // Toggle window visibility
                    let _ = crate::window::toggle_window(app).await;
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
