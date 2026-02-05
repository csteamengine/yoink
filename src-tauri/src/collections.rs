use crate::database::{Collection, Database, Tag};
use chrono::Utc;
use uuid::Uuid;

#[tauri::command]
pub async fn create_collection(
    db: tauri::State<'_, Database>,
    name: String,
    color: String,
) -> Result<Collection, String> {
    let collection = Collection {
        id: Uuid::new_v4().to_string(),
        name,
        color,
        created_at: Utc::now(),
    };

    db.create_collection(&collection).map_err(|e| e.to_string())?;

    Ok(collection)
}

#[tauri::command]
pub async fn get_collections(db: tauri::State<'_, Database>) -> Result<Vec<Collection>, String> {
    db.get_collections().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_collection(db: tauri::State<'_, Database>, id: String) -> Result<(), String> {
    db.delete_collection(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_collection(
    db: tauri::State<'_, Database>,
    id: String,
    name: String,
    color: String,
) -> Result<(), String> {
    db.update_collection(&id, &name, &color)
        .map_err(|e| e.to_string())
}

// Tag commands
#[tauri::command]
pub async fn create_tag(db: tauri::State<'_, Database>, name: String) -> Result<Tag, String> {
    let tag = Tag {
        id: Uuid::new_v4().to_string(),
        name,
    };

    db.create_tag(&tag).map_err(|e| e.to_string())?;

    Ok(tag)
}

#[tauri::command]
pub async fn get_tags(db: tauri::State<'_, Database>) -> Result<Vec<Tag>, String> {
    db.get_tags().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_tag_to_item(
    db: tauri::State<'_, Database>,
    item_id: String,
    tag_id: String,
) -> Result<(), String> {
    db.add_tag_to_item(&item_id, &tag_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_tag_from_item(
    db: tauri::State<'_, Database>,
    item_id: String,
    tag_id: String,
) -> Result<(), String> {
    db.remove_tag_from_item(&item_id, &tag_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_item_tags(
    db: tauri::State<'_, Database>,
    item_id: String,
) -> Result<Vec<Tag>, String> {
    db.get_item_tags(&item_id).map_err(|e| e.to_string())
}
