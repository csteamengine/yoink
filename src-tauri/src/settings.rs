use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_hotkey")]
    pub hotkey: String,

    #[serde(default)]
    pub launch_at_startup: bool,

    #[serde(default = "default_history_limit")]
    pub history_limit: u32,

    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default = "default_accent")]
    pub accent_color: String,

    #[serde(default = "default_font_size")]
    pub font_size: u32,

    #[serde(default = "default_true")]
    pub show_timestamps: bool,

    #[serde(default)]
    pub excluded_apps: Vec<String>,

    #[serde(default)]
    pub queue_mode_enabled: bool,

    #[serde(default = "default_true")]
    pub auto_paste: bool,

    #[serde(default)]
    pub sticky_mode: bool,
}

fn default_hotkey() -> String {
    #[cfg(target_os = "macos")]
    return "Command+Shift+V".to_string();
    #[cfg(not(target_os = "macos"))]
    return "Ctrl+Shift+V".to_string();
}

fn default_history_limit() -> u32 {
    100
}

fn default_theme() -> String {
    "system".to_string()
}

fn default_accent() -> String {
    "blue".to_string()
}

fn default_font_size() -> u32 {
    14
}

fn default_true() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hotkey: default_hotkey(),
            launch_at_startup: false,
            history_limit: default_history_limit(),
            theme: default_theme(),
            accent_color: default_accent(),
            font_size: default_font_size(),
            show_timestamps: true,
            excluded_apps: Vec::new(),
            queue_mode_enabled: false,
            auto_paste: true,
            sticky_mode: false,
        }
    }
}

pub struct SettingsManager {
    settings: Mutex<Settings>,
    path: PathBuf,
}

impl SettingsManager {
    pub fn new(app_data_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&app_data_dir).ok();
        let path = app_data_dir.join("settings.json");

        let settings = if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => Settings::default(),
            }
        } else {
            Settings::default()
        };

        Self {
            settings: Mutex::new(settings),
            path,
        }
    }

    pub fn get(&self) -> Settings {
        self.settings.lock().unwrap().clone()
    }

    pub fn update(&self, new_settings: Settings) -> Result<(), String> {
        let mut settings = self.settings.lock().unwrap();
        *settings = new_settings;

        let json = serde_json::to_string_pretty(&*settings).map_err(|e| e.to_string())?;

        std::fs::write(&self.path, json).map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn update_field<F>(&self, updater: F) -> Result<Settings, String>
    where
        F: FnOnce(&mut Settings),
    {
        let mut settings = self.settings.lock().unwrap();
        updater(&mut settings);

        let json = serde_json::to_string_pretty(&*settings).map_err(|e| e.to_string())?;
        std::fs::write(&self.path, json).map_err(|e| e.to_string())?;

        Ok(settings.clone())
    }
}

// Tauri commands
#[tauri::command]
pub async fn get_settings(manager: tauri::State<'_, SettingsManager>) -> Result<Settings, String> {
    Ok(manager.get())
}

#[tauri::command]
pub async fn update_settings(
    manager: tauri::State<'_, SettingsManager>,
    settings: Settings,
) -> Result<(), String> {
    manager.update(settings)
}

#[tauri::command]
pub async fn set_hotkey(
    manager: tauri::State<'_, SettingsManager>,
    hotkey: String,
) -> Result<Settings, String> {
    manager.update_field(|s| s.hotkey = hotkey)
}

#[tauri::command]
pub async fn set_theme(
    manager: tauri::State<'_, SettingsManager>,
    theme: String,
) -> Result<Settings, String> {
    manager.update_field(|s| s.theme = theme)
}

#[tauri::command]
pub async fn set_accent_color(
    manager: tauri::State<'_, SettingsManager>,
    accent_color: String,
) -> Result<Settings, String> {
    manager.update_field(|s| s.accent_color = accent_color)
}

#[tauri::command]
pub async fn add_excluded_app(
    manager: tauri::State<'_, SettingsManager>,
    app_id: String,
) -> Result<Settings, String> {
    manager.update_field(|s| {
        if !s.excluded_apps.contains(&app_id) {
            s.excluded_apps.push(app_id);
        }
    })
}

#[tauri::command]
pub async fn remove_excluded_app(
    manager: tauri::State<'_, SettingsManager>,
    app_id: String,
) -> Result<Settings, String> {
    manager.update_field(|s| {
        s.excluded_apps.retain(|a| a != &app_id);
    })
}

#[tauri::command]
pub async fn toggle_queue_mode(
    manager: tauri::State<'_, SettingsManager>,
) -> Result<Settings, String> {
    manager.update_field(|s| s.queue_mode_enabled = !s.queue_mode_enabled)
}
