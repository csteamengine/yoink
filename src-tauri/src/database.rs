use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: String,
    pub content_type: String, // "text", "image", "file"
    pub content: String,
    pub preview: String,
    pub hash: String,
    pub is_pinned: bool,
    pub collection_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_data_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&app_data_dir).ok();
        let db_path = app_data_dir.join("yoink.db");
        let conn = Connection::open(db_path)?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS clipboard_items (
                id TEXT PRIMARY KEY,
                content_type TEXT NOT NULL,
                content TEXT NOT NULL,
                preview TEXT NOT NULL,
                hash TEXT NOT NULL,
                is_pinned INTEGER NOT NULL DEFAULT 0,
                collection_id TEXT,
                created_at TEXT NOT NULL,
                expires_at TEXT
            );

            CREATE TABLE IF NOT EXISTS collections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                color TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
            );

            CREATE TABLE IF NOT EXISTS item_tags (
                item_id TEXT NOT NULL,
                tag_id TEXT NOT NULL,
                PRIMARY KEY (item_id, tag_id),
                FOREIGN KEY (item_id) REFERENCES clipboard_items(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_items_created_at ON clipboard_items(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_items_hash ON clipboard_items(hash);
            CREATE INDEX IF NOT EXISTS idx_items_pinned ON clipboard_items(is_pinned);
            CREATE INDEX IF NOT EXISTS idx_items_collection ON clipboard_items(collection_id);
            "#,
        )?;

        Ok(())
    }

    pub fn insert_item(&self, item: &ClipboardItem) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            r#"
            INSERT INTO clipboard_items (id, content_type, content, preview, hash, is_pinned, collection_id, created_at, expires_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                item.id,
                item.content_type,
                item.content,
                item.preview,
                item.hash,
                item.is_pinned as i32,
                item.collection_id,
                item.created_at.to_rfc3339(),
                item.expires_at.map(|dt| dt.to_rfc3339()),
            ],
        )?;

        Ok(())
    }

    pub fn get_last_hash(&self) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();

        let result: Option<String> = conn
            .query_row(
                "SELECT hash FROM clipboard_items ORDER BY created_at DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .ok();

        Ok(result)
    }

    pub fn get_items(
        &self,
        limit: u32,
        offset: u32,
        search: Option<&str>,
        collection_id: Option<&str>,
    ) -> Result<Vec<ClipboardItem>> {
        let conn = self.conn.lock().unwrap();

        let mut query = String::from(
            r#"
            SELECT id, content_type, content, preview, hash, is_pinned, collection_id, created_at, expires_at
            FROM clipboard_items
            WHERE 1=1
            "#,
        );

        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(s) = search {
            query.push_str(" AND (content LIKE ?1 OR preview LIKE ?1)");
            params_vec.push(Box::new(format!("%{}%", s)));
        }

        if let Some(cid) = collection_id {
            let param_num = params_vec.len() + 1;
            query.push_str(&format!(" AND collection_id = ?{}", param_num));
            params_vec.push(Box::new(cid.to_string()));
        }

        query.push_str(" ORDER BY is_pinned DESC, created_at DESC");
        query.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        let mut stmt = conn.prepare(&query)?;

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

        let items = stmt
            .query_map(params_refs.as_slice(), |row| {
                let created_str: String = row.get(7)?;
                let expires_str: Option<String> = row.get(8)?;

                Ok(ClipboardItem {
                    id: row.get(0)?,
                    content_type: row.get(1)?,
                    content: row.get(2)?,
                    preview: row.get(3)?,
                    hash: row.get(4)?,
                    is_pinned: row.get::<_, i32>(5)? != 0,
                    collection_id: row.get(6)?,
                    created_at: DateTime::parse_from_rfc3339(&created_str)
                        .unwrap_or_else(|_| Utc::now().into())
                        .with_timezone(&Utc),
                    expires_at: expires_str.and_then(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .ok()
                            .map(|dt| dt.with_timezone(&Utc))
                    }),
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(items)
    }

    pub fn get_pinned_items(&self) -> Result<Vec<ClipboardItem>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT id, content_type, content, preview, hash, is_pinned, collection_id, created_at, expires_at
            FROM clipboard_items
            WHERE is_pinned = 1
            ORDER BY created_at DESC
            "#,
        )?;

        let items = stmt
            .query_map([], |row| {
                let created_str: String = row.get(7)?;
                let expires_str: Option<String> = row.get(8)?;

                Ok(ClipboardItem {
                    id: row.get(0)?,
                    content_type: row.get(1)?,
                    content: row.get(2)?,
                    preview: row.get(3)?,
                    hash: row.get(4)?,
                    is_pinned: row.get::<_, i32>(5)? != 0,
                    collection_id: row.get(6)?,
                    created_at: DateTime::parse_from_rfc3339(&created_str)
                        .unwrap_or_else(|_| Utc::now().into())
                        .with_timezone(&Utc),
                    expires_at: expires_str.and_then(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .ok()
                            .map(|dt| dt.with_timezone(&Utc))
                    }),
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(items)
    }

    pub fn delete_item(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM clipboard_items WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn pin_item(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE clipboard_items SET is_pinned = 1 WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn unpin_item(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE clipboard_items SET is_pinned = 0 WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn clear_history(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM clipboard_items WHERE is_pinned = 0", [])?;
        Ok(())
    }

    pub fn get_item(&self, id: &str) -> Result<Option<ClipboardItem>> {
        let conn = self.conn.lock().unwrap();

        let result = conn.query_row(
            r#"
            SELECT id, content_type, content, preview, hash, is_pinned, collection_id, created_at, expires_at
            FROM clipboard_items
            WHERE id = ?1
            "#,
            params![id],
            |row| {
                let created_str: String = row.get(7)?;
                let expires_str: Option<String> = row.get(8)?;

                Ok(ClipboardItem {
                    id: row.get(0)?,
                    content_type: row.get(1)?,
                    content: row.get(2)?,
                    preview: row.get(3)?,
                    hash: row.get(4)?,
                    is_pinned: row.get::<_, i32>(5)? != 0,
                    collection_id: row.get(6)?,
                    created_at: DateTime::parse_from_rfc3339(&created_str)
                        .unwrap_or_else(|_| Utc::now().into())
                        .with_timezone(&Utc),
                    expires_at: expires_str.and_then(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .ok()
                            .map(|dt| dt.with_timezone(&Utc))
                    }),
                })
            },
        );

        match result {
            Ok(item) => Ok(Some(item)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn enforce_limit(&self, limit: u32) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            r#"
            DELETE FROM clipboard_items
            WHERE id NOT IN (
                SELECT id FROM clipboard_items
                WHERE is_pinned = 1
                UNION ALL
                SELECT id FROM (
                    SELECT id FROM clipboard_items
                    WHERE is_pinned = 0
                    ORDER BY created_at DESC
                    LIMIT ?1
                )
            )
            "#,
            params![limit],
        )?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn cleanup_expired(&self) -> Result<u32> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();

        let deleted = conn.execute(
            "DELETE FROM clipboard_items WHERE expires_at IS NOT NULL AND expires_at < ?1",
            params![now],
        )?;

        Ok(deleted as u32)
    }

    // Collection methods
    pub fn create_collection(&self, collection: &Collection) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO collections (id, name, color, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![
                collection.id,
                collection.name,
                collection.color,
                collection.created_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    pub fn get_collections(&self) -> Result<Vec<Collection>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare("SELECT id, name, color, created_at FROM collections ORDER BY name")?;

        let collections = stmt
            .query_map([], |row| {
                let created_str: String = row.get(3)?;

                Ok(Collection {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                    created_at: DateTime::parse_from_rfc3339(&created_str)
                        .unwrap_or_else(|_| Utc::now().into())
                        .with_timezone(&Utc),
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(collections)
    }

    pub fn delete_collection(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Remove collection reference from items
        conn.execute(
            "UPDATE clipboard_items SET collection_id = NULL WHERE collection_id = ?1",
            params![id],
        )?;

        conn.execute("DELETE FROM collections WHERE id = ?1", params![id])?;

        Ok(())
    }

    pub fn update_collection(&self, id: &str, name: &str, color: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "UPDATE collections SET name = ?1, color = ?2 WHERE id = ?3",
            params![name, color, id],
        )?;

        Ok(())
    }

    pub fn move_item_to_collection(&self, item_id: &str, collection_id: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "UPDATE clipboard_items SET collection_id = ?1 WHERE id = ?2",
            params![collection_id, item_id],
        )?;

        Ok(())
    }

    pub fn set_item_expiration(&self, item_id: &str, expires_at: Option<DateTime<Utc>>) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "UPDATE clipboard_items SET expires_at = ?1 WHERE id = ?2",
            params![expires_at.map(|dt| dt.to_rfc3339()), item_id],
        )?;

        Ok(())
    }

    // Tag methods
    pub fn create_tag(&self, tag: &Tag) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT OR IGNORE INTO tags (id, name) VALUES (?1, ?2)",
            params![tag.id, tag.name],
        )?;

        Ok(())
    }

    pub fn get_tags(&self) -> Result<Vec<Tag>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare("SELECT id, name FROM tags ORDER BY name")?;

        let tags = stmt
            .query_map([], |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(tags)
    }

    pub fn add_tag_to_item(&self, item_id: &str, tag_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT OR IGNORE INTO item_tags (item_id, tag_id) VALUES (?1, ?2)",
            params![item_id, tag_id],
        )?;

        Ok(())
    }

    pub fn remove_tag_from_item(&self, item_id: &str, tag_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "DELETE FROM item_tags WHERE item_id = ?1 AND tag_id = ?2",
            params![item_id, tag_id],
        )?;

        Ok(())
    }

    pub fn get_item_tags(&self, item_id: &str) -> Result<Vec<Tag>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT t.id, t.name
            FROM tags t
            JOIN item_tags it ON t.id = it.tag_id
            WHERE it.item_id = ?1
            ORDER BY t.name
            "#,
        )?;

        let tags = stmt
            .query_map(params![item_id], |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(tags)
    }
}
