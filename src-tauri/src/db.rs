use crate::classifier::ContentType;
use crate::keychain;
use crate::search::split_camel_case;
use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

#[derive(Serialize, Clone)]
pub struct ClipMetadata {
    pub id: String,
    pub preview: String,
    pub content_type: ContentType,
    pub source_app: String,
    pub window_title: Option<String>,
    pub url: Option<String>,
    pub timestamp: i64,
}

pub fn init_db(app_handle: &AppHandle) -> rusqlite::Result<Connection> {
    let dir = app_handle
        .path()
        .app_data_dir()
        .expect("app data dir should be resolvable");
    std::fs::create_dir_all(&dir).expect("failed to create app data dir");

    let conn = Connection::open(dir.join("clips.db"))?;

    // AES-256 via SQLCipher. The key never touches disk in plaintext —
    // it lives in the macOS Keychain and is fetched fresh on every launch.
    let key = keychain::get_or_create_db_key();
    conn.pragma_update(None, "key", &key)?;

    // A wrong key doesn't error on `key` itself — SQLCipher only notices
    // once you actually touch the database. This query is what surfaces it.
    let integrity: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if integrity != "ok" {
        panic!("clips.db failed integrity check: {integrity}");
    }

    conn.execute(
        "CREATE TABLE IF NOT EXISTS clips (
            id TEXT PRIMARY KEY,
            content TEXT NOT NULL,
            content_type TEXT NOT NULL DEFAULT 'PlainText',
            source_app TEXT NOT NULL DEFAULT '',
            window_title TEXT,
            url TEXT,
            timestamp INTEGER NOT NULL
        )",
        (),
    )?;

    // Standalone FTS5 table (not "external content") — simpler to keep in
    // sync than the trigger-based external-content pattern, at the cost of
    // duplicating text already in `clips`. Fine at clipboard-history scale.
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS clips_fts USING fts5(
            id UNINDEXED,
            search_text,
            window_title,
            source_app
        )",
        (),
    )?;

    Ok(conn)
}

pub fn save_clip(
    conn: &Connection,
    content: &str,
    content_type: ContentType,
    source_app: &str,
) -> rusqlite::Result<ClipMetadata> {
    let id = Uuid::new_v4().to_string();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    conn.execute(
        "INSERT INTO clips (id, content, content_type, source_app, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
        (&id, content, content_type.as_str(), source_app, timestamp),
    )?;
    conn.execute(
        "INSERT INTO clips_fts (id, search_text, window_title, source_app) VALUES (?1, ?2, '', ?3)",
        (&id, split_camel_case(content), source_app),
    )?;

    Ok(ClipMetadata {
        id,
        preview: content.chars().take(200).collect(),
        content_type,
        source_app: source_app.to_string(),
        window_title: None,
        url: None,
        timestamp,
    })
}

// Called once the async, timeout-bounded Accessibility fetch resolves.
// If it never resolves in time, these columns just stay NULL — no retry.
pub fn update_clip_context(
    conn: &Connection,
    id: &str,
    window_title: Option<&str>,
    url: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE clips SET window_title = ?1, url = ?2 WHERE id = ?3",
        (window_title, url, id),
    )?;
    conn.execute(
        "UPDATE clips_fts SET window_title = ?1 WHERE id = ?2",
        (window_title.unwrap_or(""), id),
    )?;
    Ok(())
}

// The palette list only ever carries `preview` (max 200 chars). Full
// content is fetched by primary key, on demand, only when a clip is
// explicitly selected — never pre-fetched into the list.
pub fn get_clip_content(conn: &Connection, id: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row("SELECT content FROM clips WHERE id = ?1", [id], |row| row.get(0))
        .optional()
}

pub fn get_recent_clips(conn: &Connection, limit: i64) -> rusqlite::Result<Vec<ClipMetadata>> {
    let mut stmt = conn.prepare(
        "SELECT id, content, content_type, source_app, window_title, url, timestamp
         FROM clips ORDER BY timestamp DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map((limit,), |row| {
        let content: String = row.get(1)?;
        let content_type: String = row.get(2)?;
        Ok(ClipMetadata {
            id: row.get(0)?,
            preview: content.chars().take(200).collect(),
            content_type: ContentType::from_str(&content_type),
            source_app: row.get(3)?,
            window_title: row.get(4)?,
            url: row.get(5)?,
            timestamp: row.get(6)?,
        })
    })?;
    rows.collect()
}
