use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

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
        let shortcut: Shortcut = hotkey.parse().map_err(|e: tauri_plugin_global_shortcut::Error| e.to_string())?;

        // Unregister existing shortcut if any
        self.unregister(app)?;

        let app_clone = app.clone();

        app.global_shortcut()
            .on_shortcut(shortcut.clone(), move |_app, _shortcut, _event| {
                // Toggle window visibility
                let app = app_clone.clone();
                tauri::async_runtime::spawn(async move {
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
