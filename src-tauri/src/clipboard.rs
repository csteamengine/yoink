use crate::database::{ClipboardItem, Database};
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Runtime};
use tauri_plugin_clipboard_manager::ClipboardExt;
use uuid::Uuid;

pub struct ClipboardMonitor {
    last_hash: Mutex<Option<String>>,
}

impl ClipboardMonitor {
    pub fn new() -> Self {
        Self {
            last_hash: Mutex::new(None),
        }
    }

    pub fn init_last_hash(&self, db: &Database) {
        if let Ok(hash) = db.get_last_hash() {
            *self.last_hash.lock().unwrap() = hash;
        }
    }
}

// Called from frontend via polling
#[tauri::command]
pub async fn check_clipboard<R: Runtime>(
    app: AppHandle<R>,
    db: tauri::State<'_, Database>,
    monitor: tauri::State<'_, ClipboardMonitor>,
) -> Result<Option<ClipboardItem>, String> {
    let clipboard = app.clipboard();

    // Try to read text content
    if let Ok(text) = clipboard.read_text() {
        if !text.is_empty() {
            let hash = compute_hash(&text);

            // Skip if same as last item
            {
                let last_hash = monitor.last_hash.lock().unwrap();
                if last_hash.as_ref() == Some(&hash) {
                    return Ok(None);
                }
            }

            // Create clipboard item
            let preview = create_text_preview(&text);
            let item = ClipboardItem {
                id: Uuid::new_v4().to_string(),
                content_type: detect_content_type(&text),
                content: text,
                preview,
                hash: hash.clone(),
                is_pinned: false,
                collection_id: None,
                created_at: Utc::now(),
                expires_at: None,
            };

            // Store in database
            db.insert_item(&item).map_err(|e| e.to_string())?;
            db.enforce_limit(100).map_err(|e| e.to_string())?;

            *monitor.last_hash.lock().unwrap() = Some(hash);

            // Emit event to frontend
            let _ = app.emit("clipboard-changed", &item);

            return Ok(Some(item));
        }
    }

    // Try to read image content
    if let Ok(image) = clipboard.read_image() {
        let rgba = image.rgba();
        if !rgba.is_empty() {
            let hash = compute_hash_bytes(&rgba);

            {
                let last_hash = monitor.last_hash.lock().unwrap();
                if last_hash.as_ref() == Some(&hash) {
                    return Ok(None);
                }
            }

            let base64_content = STANDARD.encode(&rgba);

            let item = ClipboardItem {
                id: Uuid::new_v4().to_string(),
                content_type: "image".to_string(),
                content: base64_content,
                preview: format!("Image ({}x{})", image.width(), image.height()),
                hash: hash.clone(),
                is_pinned: false,
                collection_id: None,
                created_at: Utc::now(),
                expires_at: None,
            };

            db.insert_item(&item).map_err(|e| e.to_string())?;
            db.enforce_limit(100).map_err(|e| e.to_string())?;

            *monitor.last_hash.lock().unwrap() = Some(hash);
            let _ = app.emit("clipboard-changed", &item);

            return Ok(Some(item));
        }
    }

    Ok(None)
}

fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn compute_hash_bytes(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

fn create_text_preview(text: &str) -> String {
    let preview: String = text
        .chars()
        .take(500)
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .collect();

    if text.len() > 500 {
        format!("{}...", preview)
    } else {
        preview
    }
}

fn detect_content_type(text: &str) -> String {
    let trimmed = text.trim();

    // Check if it's a file path (Unix or Windows)
    if trimmed.starts_with('/') || (trimmed.len() > 2 && &trimmed[1..3] == ":\\") {
        // Check for multiple paths (newline separated)
        if trimmed.contains('\n') {
            return "files".to_string();
        }
        return "file".to_string();
    }

    // Check if it's a URL
    if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("ftp://")
    {
        return "url".to_string();
    }

    // Check if it looks like code
    if looks_like_code(trimmed) {
        return "code".to_string();
    }

    "text".to_string()
}

fn looks_like_code(text: &str) -> bool {
    let code_indicators = [
        "function ",
        "const ",
        "let ",
        "var ",
        "import ",
        "export ",
        "class ",
        "def ",
        "fn ",
        "pub ",
        "async ",
        "await ",
        "return ",
        "if (",
        "for (",
        "while (",
        "=>",
        "->",
        "{}",
        "();",
    ];

    let text_lower = text.to_lowercase();
    let indicator_count = code_indicators
        .iter()
        .filter(|&indicator| text_lower.contains(&indicator.to_lowercase()))
        .count();

    // If multiple code indicators found, likely code
    indicator_count >= 2
}

// Tauri commands
#[tauri::command]
pub async fn get_clipboard_items(
    db: tauri::State<'_, Database>,
    limit: u32,
    offset: u32,
    search: Option<String>,
    collection_id: Option<String>,
) -> Result<Vec<ClipboardItem>, String> {
    db.get_items(
        limit,
        offset,
        search.as_deref(),
        collection_id.as_deref(),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_pinned_items(
    db: tauri::State<'_, Database>,
) -> Result<Vec<ClipboardItem>, String> {
    db.get_pinned_items().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_clipboard_item(
    db: tauri::State<'_, Database>,
    id: String,
) -> Result<(), String> {
    db.delete_item(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pin_item(db: tauri::State<'_, Database>, id: String) -> Result<(), String> {
    db.pin_item(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn unpin_item(db: tauri::State<'_, Database>, id: String) -> Result<(), String> {
    db.unpin_item(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_history(db: tauri::State<'_, Database>) -> Result<(), String> {
    db.clear_history().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn paste_item<R: Runtime>(
    app: AppHandle<R>,
    db: tauri::State<'_, Database>,
    previous_app_state: tauri::State<'_, crate::paste_helper::PreviousAppState>,
    settings_manager: tauri::State<'_, crate::settings::SettingsManager>,
    id: String,
) -> Result<(), String> {
    let item = db.get_item(&id).map_err(|e| e.to_string())?;
    let settings = settings_manager.get();

    if let Some(item) = item {
        let clipboard = app.clipboard();

        match item.content_type.as_str() {
            "image" => {
                // Decode base64 and write as image
                if let Ok(_bytes) = STANDARD.decode(&item.content) {
                    // For now, write as text since image writing needs raw image data
                    // TODO: Properly handle image pasting
                    clipboard
                        .write_text(&item.preview)
                        .map_err(|e| e.to_string())?;
                }
            }
            _ => {
                clipboard
                    .write_text(&item.content)
                    .map_err(|e| e.to_string())?;
            }
        }

        // Auto-paste to previous app if enabled
        if settings.auto_paste {
            // Hide the window first
            crate::window::hide_window(app.clone()).await?;

            // Paste to previous app
            if let Err(e) = crate::paste_helper::paste_to_previous_app(&previous_app_state).await {
                log::warn!("Failed to auto-paste: {}", e);
                // Don't fail the whole operation, clipboard is already updated
            }
        } else {
            // Even if auto-paste is disabled, hide the window
            crate::window::hide_window(app.clone()).await?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn move_to_collection(
    db: tauri::State<'_, Database>,
    item_id: String,
    collection_id: Option<String>,
) -> Result<(), String> {
    db.move_item_to_collection(&item_id, collection_id.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_expiration(
    db: tauri::State<'_, Database>,
    item_id: String,
    expires_at: Option<String>,
) -> Result<(), String> {
    let expires = expires_at.and_then(|s| {
        chrono::DateTime::parse_from_rfc3339(&s)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    });

    db.set_item_expiration(&item_id, expires)
        .map_err(|e| e.to_string())
}
