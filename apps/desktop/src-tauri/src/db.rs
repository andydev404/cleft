use crate::classifier::ContentType;
use crate::keychain;
use crate::search::split_camel_case;
use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

// History is a rolling FIFO queue, not a hard wall — the app never stops
// saving, it just drops the oldest *unpinned* clip to make room. Pinned
// favorites are exempt and never age out.
pub const FREE_TIER_LIMIT: i64 = 500;
pub const DEFAULT_WORKSPACE: &str = "Personal";
pub const MAX_WORKSPACES: i64 = 5;

#[derive(Serialize, Clone)]
pub struct ClipMetadata {
    pub id: String,
    pub preview: String,
    pub content_type: ContentType,
    pub source_app: String,
    pub window_title: Option<String>,
    pub url: Option<String>,
    pub timestamp: i64,
    pub is_favorite: bool,
    pub workspace: String,
    pub collection: Option<String>,
    pub tags: Vec<String>,
    pub copy_count: i64,
    pub expires_at: Option<i64>,
}

#[derive(Serialize)]
pub struct Workspace {
    pub name: String,
    pub is_current: bool,
}

#[derive(Serialize)]
pub struct CollectionSummary {
    pub name: String,
    pub count: i64,
    pub samples: Vec<String>,
}

fn parse_tags(raw: &str) -> Vec<String> {
    if raw.is_empty() {
        Vec::new()
    } else {
        raw.split(',').map(str::to_string).collect()
    }
}

// SQLite's `ALTER TABLE ... ADD COLUMN` errors on a column that already
// exists — that error is exactly the "nothing to do" case here, so it's
// swallowed. Needed because `CREATE TABLE IF NOT EXISTS` is a no-op against
// a database created by an older version of this app, which would
// otherwise leave newer columns permanently missing.
fn add_column_if_missing(conn: &Connection, column_def: &str) {
    let _ = conn.execute(&format!("ALTER TABLE clips ADD COLUMN {column_def}"), ());
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

    // WAL mode: writes go to a separate -wal file instead of touching the
    // main database file directly, so a crash mid-write leaves the actual
    // clips.db untouched rather than half-modified. Non-fatal if it doesn't
    // stick (e.g. an unusual filesystem) — this is a reliability
    // improvement, not a correctness requirement, so it shouldn't block
    // startup the way a failed integrity check does.
    match conn.query_row("PRAGMA journal_mode=WAL", [], |row| row.get::<_, String>(0)) {
        Ok(mode) if mode.eq_ignore_ascii_case("wal") => {}
        Ok(mode) => eprintln!("warning: expected WAL journal mode, got {mode}"),
        Err(e) => eprintln!("warning: failed to enable WAL mode: {e}"),
    }

    conn.execute(
        "CREATE TABLE IF NOT EXISTS clips (
            id TEXT PRIMARY KEY,
            content TEXT NOT NULL,
            content_type TEXT NOT NULL DEFAULT 'PlainText',
            source_app TEXT NOT NULL DEFAULT '',
            window_title TEXT,
            url TEXT,
            timestamp INTEGER NOT NULL,
            is_favorite INTEGER NOT NULL DEFAULT 0,
            workspace TEXT NOT NULL DEFAULT 'Personal',
            collection TEXT,
            tags TEXT NOT NULL DEFAULT '',
            copy_count INTEGER NOT NULL DEFAULT 0,
            expires_at INTEGER
        )",
        (),
    )?;
    for column_def in [
        "is_favorite INTEGER NOT NULL DEFAULT 0",
        "workspace TEXT NOT NULL DEFAULT 'Personal'",
        "collection TEXT",
        "tags TEXT NOT NULL DEFAULT ''",
        "copy_count INTEGER NOT NULL DEFAULT 0",
        "expires_at INTEGER",
    ] {
        add_column_if_missing(&conn, column_def);
    }

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

    conn.execute(
        "CREATE TABLE IF NOT EXISTS workspaces (
            name TEXT PRIMARY KEY,
            is_current INTEGER NOT NULL DEFAULT 0
        )",
        (),
    )?;
    // Two default workspaces; Personal starts active.
    let workspace_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM workspaces", [], |r| r.get(0))?;
    if workspace_count == 0 {
        conn.execute(
            "INSERT INTO workspaces (name, is_current) VALUES ('Personal', 1)",
            (),
        )?;
        conn.execute(
            "INSERT INTO workspaces (name, is_current) VALUES ('Work', 0)",
            (),
        )?;
    }

    Ok(conn)
}

// Deletes the oldest unpinned clips until the total is back at `limit`.
// Called from save_clip on every insert. Returns the ids evicted so callers
// can emit an event —
// the frontend's in-memory list needs to drop them too, or a click on an
// evicted row would silently fail against get_clip_content. Applies across
// all workspaces combined — the free-tier cap is a single account-wide
// budget, not one per workspace.
fn enforce_free_tier_limit(conn: &Connection, limit: i64) -> rusqlite::Result<Vec<String>> {
    let total: i64 = conn.query_row("SELECT COUNT(*) FROM clips", [], |row| row.get(0))?;
    let overflow = total - limit;
    if overflow <= 0 {
        return Ok(Vec::new());
    }

    let ids: Vec<String> = {
        let mut stmt = conn.prepare(
            "SELECT id FROM clips WHERE is_favorite = 0 ORDER BY timestamp ASC LIMIT ?1",
        )?;
        let rows = stmt.query_map([overflow], |row| row.get::<_, String>(0))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };
    // If most clips are pinned there may be fewer unpinned ones than the
    // overflow — that's fine, pinned clips simply keep growing the total
    // past `limit`.
    delete_clips(conn, &ids)?;
    Ok(ids)
}

pub fn get_current_workspace(conn: &Connection) -> rusqlite::Result<String> {
    conn.query_row(
        "SELECT name FROM workspaces WHERE is_current = 1",
        [],
        |r| r.get(0),
    )
    .optional()
    .map(|w| w.unwrap_or_else(|| DEFAULT_WORKSPACE.to_string()))
}

pub fn list_workspaces(conn: &Connection) -> rusqlite::Result<Vec<Workspace>> {
    let mut stmt = conn.prepare("SELECT name, is_current FROM workspaces ORDER BY rowid")?;
    let rows = stmt.query_map([], |r| {
        Ok(Workspace {
            name: r.get(0)?,
            is_current: r.get(1)?,
        })
    })?;
    rows.collect()
}

pub fn create_workspace(conn: &Connection, name: &str) -> rusqlite::Result<Result<(), String>> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM workspaces", [], |r| r.get(0))?;
    if count >= MAX_WORKSPACES {
        return Ok(Err(format!(
            "Free tier allows up to {MAX_WORKSPACES} workspaces"
        )));
    }
    match conn.execute(
        "INSERT INTO workspaces (name, is_current) VALUES (?1, 0)",
        [name],
    ) {
        Ok(_) => Ok(Ok(())),
        Err(rusqlite::Error::SqliteFailure(e, _))
            if e.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            Ok(Err(format!("A workspace named \"{name}\" already exists")))
        }
        Err(e) => Err(e),
    }
}

pub fn switch_workspace(conn: &Connection, name: &str) -> rusqlite::Result<()> {
    conn.execute("UPDATE workspaces SET is_current = (name = ?1)", [name])?;
    Ok(())
}

// A workspace's clips don't exist anywhere else (workspaces don't share
// history), so deleting one takes its clips with it — reusing
// delete_clips keeps the FTS index in sync the same way single/bulk clip
// delete already does. Always leaves at least one workspace behind, and
// hands "current" to another one if the deleted workspace was it.
pub fn delete_workspace(conn: &Connection, name: &str) -> rusqlite::Result<Result<(), String>> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM workspaces", [], |r| r.get(0))?;
    if count <= 1 {
        return Ok(Err("Can't delete your only workspace".to_string()));
    }

    let clip_ids: Vec<String> = {
        let mut stmt = conn.prepare("SELECT id FROM clips WHERE workspace = ?1")?;
        let rows = stmt.query_map([name], |r| r.get::<_, String>(0))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };
    delete_clips(conn, &clip_ids)?;

    let was_current: bool = conn
        .query_row(
            "SELECT is_current FROM workspaces WHERE name = ?1",
            [name],
            |r| r.get(0),
        )
        .unwrap_or(false);
    conn.execute("DELETE FROM workspaces WHERE name = ?1", [name])?;

    if was_current {
        let fallback: String = conn.query_row(
            "SELECT name FROM workspaces ORDER BY rowid LIMIT 1",
            [],
            |r| r.get(0),
        )?;
        conn.execute(
            "UPDATE workspaces SET is_current = 1 WHERE name = ?1",
            [&fallback],
        )?;
    }
    Ok(Ok(()))
}

pub fn assign_collection(
    conn: &Connection,
    id: &str,
    collection: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE clips SET collection = ?1 WHERE id = ?2",
        (collection, id),
    )?;
    Ok(())
}

pub fn list_collections(conn: &Connection) -> rusqlite::Result<Vec<CollectionSummary>> {
    let mut stmt = conn.prepare(
        "SELECT collection, COUNT(*) FROM clips WHERE collection IS NOT NULL
         GROUP BY collection ORDER BY collection",
    )?;
    let names: Vec<(String, i64)> = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?
        .collect::<rusqlite::Result<_>>()?;

    let mut summaries = Vec::with_capacity(names.len());
    for (name, count) in names {
        let mut sample_stmt = conn.prepare(
            "SELECT content FROM clips WHERE collection = ?1 ORDER BY timestamp DESC LIMIT 2",
        )?;
        let samples: Vec<String> = sample_stmt
            .query_map([&name], |r| {
                Ok(r.get::<_, String>(0)?.chars().take(80).collect())
            })?
            .collect::<rusqlite::Result<_>>()?;
        summaries.push(CollectionSummary {
            name,
            count,
            samples,
        });
    }
    Ok(summaries)
}

pub fn add_tag(conn: &Connection, id: &str, tag: &str) -> rusqlite::Result<()> {
    let existing: String =
        conn.query_row("SELECT tags FROM clips WHERE id = ?1", [id], |r| r.get(0))?;
    let mut tags = parse_tags(&existing);
    if !tags.iter().any(|t| t == tag) {
        tags.push(tag.to_string());
    }
    conn.execute(
        "UPDATE clips SET tags = ?1 WHERE id = ?2",
        (tags.join(","), id),
    )?;
    Ok(())
}

pub fn remove_tag(conn: &Connection, id: &str, tag: &str) -> rusqlite::Result<()> {
    let existing: String =
        conn.query_row("SELECT tags FROM clips WHERE id = ?1", [id], |r| r.get(0))?;
    let tags: Vec<String> = parse_tags(&existing)
        .into_iter()
        .filter(|t| t != tag)
        .collect();
    conn.execute(
        "UPDATE clips SET tags = ?1 WHERE id = ?2",
        (tags.join(","), id),
    )?;
    Ok(())
}

pub fn save_clip(
    conn: &Connection,
    content: &str,
    content_type: ContentType,
    source_app: &str,
    workspace: &str,
) -> rusqlite::Result<(ClipMetadata, Vec<String>)> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Duplicate detection: re-copying content that's already in this
    // workspace bumps the existing clip to the top (fresh timestamp)
    // instead of inserting a twin. Its tags/collection/pin/copy-count
    // survive — that's the point. Plain equality scan is fine at the
    // 500-clip cap. The original capture context (source app, window,
    // URL) is kept; only the time moves.
    let existing: Option<String> = conn
        .query_row(
            "SELECT id FROM clips WHERE workspace = ?1 AND content = ?2",
            (workspace, content),
            |r| r.get(0),
        )
        .optional()?;
    if let Some(id) = existing {
        conn.execute(
            "UPDATE clips SET timestamp = ?1 WHERE id = ?2",
            (timestamp, &id),
        )?;
        let sql = format!("SELECT {CLIP_COLUMNS} FROM clips WHERE id = ?1");
        let metadata = conn.query_row(&sql, [&id], row_to_metadata)?;
        return Ok((metadata, Vec::new()));
    }

    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO clips (id, content, content_type, source_app, timestamp, workspace) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (&id, content, content_type.as_str(), source_app, timestamp, workspace),
    )?;
    conn.execute(
        "INSERT INTO clips_fts (id, search_text, window_title, source_app) VALUES (?1, ?2, '', ?3)",
        (&id, split_camel_case(content), source_app),
    )?;

    let evicted = enforce_free_tier_limit(conn, FREE_TIER_LIMIT)?;

    Ok((
        ClipMetadata {
            id,
            preview: content.chars().take(200).collect(),
            content_type,
            source_app: source_app.to_string(),
            window_title: None,
            url: None,
            timestamp,
            is_favorite: false,
            workspace: workspace.to_string(),
            collection: None,
            tags: Vec::new(),
            copy_count: 0,
            expires_at: None,
        },
        evicted,
    ))
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

pub fn set_favorite(conn: &Connection, id: &str, favorite: bool) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE clips SET is_favorite = ?1 WHERE id = ?2",
        (favorite, id),
    )?;
    Ok(())
}

// Counts copies *out of Cleft* (the Copy button, Enter, ⌘1-9) — not
// captures. Over time the most-reached-for clips carry the highest counts.
pub fn increment_copy_count(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE clips SET copy_count = copy_count + 1 WHERE id = ?1",
        [id],
    )?;
    Ok(())
}

// NULL = never expires. The monitor loop sweeps via purge_expired.
pub fn set_expiry(conn: &Connection, id: &str, expires_at: Option<i64>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE clips SET expires_at = ?1 WHERE id = ?2",
        (expires_at, id),
    )?;
    Ok(())
}

// Deletes every clip whose expiry has passed and returns their ids so the
// caller can emit clips-evicted (the frontend drops them from memory the
// same way FIFO evictions are handled). Reuses delete_clips so the FTS
// index stays in sync.
pub fn purge_expired(conn: &Connection) -> rusqlite::Result<Vec<String>> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let ids: Vec<String> = {
        let mut stmt =
            conn.prepare("SELECT id FROM clips WHERE expires_at IS NOT NULL AND expires_at <= ?1")?;
        let rows = stmt.query_map([now], |row| row.get::<_, String>(0))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };
    delete_clips(conn, &ids)?;
    Ok(ids)
}

// The palette list only ever carries `preview` (max 200 chars). Full
// content is fetched by primary key, on demand, only when a clip is
// explicitly selected — never pre-fetched into the list.
pub fn get_clip_content(conn: &Connection, id: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row("SELECT content FROM clips WHERE id = ?1", [id], |row| {
        row.get(0)
    })
    .optional()
}

pub fn delete_clips(conn: &Connection, ids: &[String]) -> rusqlite::Result<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let params: Vec<&dyn rusqlite::types::ToSql> = ids
        .iter()
        .map(|id| id as &dyn rusqlite::types::ToSql)
        .collect();

    conn.execute(
        &format!("DELETE FROM clips WHERE id IN ({placeholders})"),
        params.as_slice(),
    )?;
    conn.execute(
        &format!("DELETE FROM clips_fts WHERE id IN ({placeholders})"),
        params.as_slice(),
    )?;
    Ok(())
}

pub fn row_to_metadata(row: &rusqlite::Row) -> rusqlite::Result<ClipMetadata> {
    let content: String = row.get(1)?;
    let content_type: String = row.get(2)?;
    let tags: String = row.get(9)?;
    Ok(ClipMetadata {
        id: row.get(0)?,
        preview: content.chars().take(200).collect(),
        content_type: ContentType::from_str(&content_type),
        source_app: row.get(3)?,
        window_title: row.get(4)?,
        url: row.get(5)?,
        timestamp: row.get(6)?,
        is_favorite: row.get(7)?,
        workspace: row.get(8)?,
        collection: row.get(10)?,
        tags: parse_tags(&tags),
        copy_count: row.get(11)?,
        expires_at: row.get(12)?,
    })
}

pub const CLIP_COLUMNS: &str =
    "id, content, content_type, source_app, window_title, url, timestamp, is_favorite, workspace, tags, collection, copy_count, expires_at";

pub fn get_recent_clips(
    conn: &Connection,
    workspace: &str,
    limit: i64,
) -> rusqlite::Result<Vec<ClipMetadata>> {
    let sql = format!(
        "SELECT {CLIP_COLUMNS} FROM clips WHERE workspace = ?1 ORDER BY timestamp DESC LIMIT ?2"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map((workspace, limit), row_to_metadata)?;
    rows.collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE clips (
                id TEXT PRIMARY KEY, content TEXT NOT NULL,
                content_type TEXT NOT NULL DEFAULT 'PlainText',
                source_app TEXT NOT NULL DEFAULT '', window_title TEXT, url TEXT,
                timestamp INTEGER NOT NULL, is_favorite INTEGER NOT NULL DEFAULT 0,
                workspace TEXT NOT NULL DEFAULT 'Personal', collection TEXT,
                tags TEXT NOT NULL DEFAULT '',
                copy_count INTEGER NOT NULL DEFAULT 0, expires_at INTEGER
            )",
            (),
        )
        .unwrap();
        conn.execute(
            "CREATE VIRTUAL TABLE clips_fts USING fts5(id UNINDEXED, search_text, window_title, source_app)",
            (),
        )
        .unwrap();
        conn.execute(
            "CREATE TABLE workspaces (name TEXT PRIMARY KEY, is_current INTEGER NOT NULL DEFAULT 0)",
            (),
        )
        .unwrap();
        conn.execute(
            "INSERT INTO workspaces (name, is_current) VALUES ('Personal', 1)",
            (),
        )
        .unwrap();
        conn.execute(
            "INSERT INTO workspaces (name, is_current) VALUES ('Work', 0)",
            (),
        )
        .unwrap();
        conn
    }

    fn save(conn: &Connection, content: &str) -> ClipMetadata {
        save_clip(
            conn,
            content,
            ContentType::PlainText,
            "app",
            DEFAULT_WORKSPACE,
        )
        .unwrap()
        .0
    }

    fn recent(conn: &Connection) -> Vec<ClipMetadata> {
        get_recent_clips(conn, DEFAULT_WORKSPACE, 1000).unwrap()
    }

    #[test]
    fn duplicate_content_bumps_existing_clip_instead_of_inserting() {
        let conn = test_conn();
        let first = save(&conn, "same thing");
        save(&conn, "something else");

        // Pin + tag the original so we can prove they survive the re-copy.
        set_favorite(&conn, &first.id, true).unwrap();
        add_tag(&conn, &first.id, "keep").unwrap();
        // Backdate it so the bump is observable.
        conn.execute(
            "UPDATE clips SET timestamp = timestamp - 100 WHERE id = ?1",
            [&first.id],
        )
        .unwrap();

        let (bumped, evicted) = save_clip(
            &conn,
            "same thing",
            ContentType::PlainText,
            "another-app",
            DEFAULT_WORKSPACE,
        )
        .unwrap();

        assert_eq!(bumped.id, first.id);
        assert!(bumped.is_favorite);
        assert_eq!(bumped.tags, vec!["keep"]);
        assert!(evicted.is_empty());

        let clips = recent(&conn);
        assert_eq!(clips.len(), 2, "no twin row was inserted");
        assert_eq!(clips[0].id, first.id, "bumped clip is back on top");
        // Original capture context is kept — only the timestamp moved.
        assert_eq!(clips[0].source_app, "app");
    }

    #[test]
    fn duplicate_detection_is_per_workspace() {
        let conn = test_conn();
        save(&conn, "shared text");
        let (other, _) =
            save_clip(&conn, "shared text", ContentType::PlainText, "app", "Work").unwrap();
        assert_eq!(other.workspace, "Work");
        let total: i64 = conn
            .query_row("SELECT COUNT(*) FROM clips", [], |r| r.get(0))
            .unwrap();
        assert_eq!(total, 2, "same content in another workspace is a new clip");
    }

    #[test]
    fn copy_count_increments() {
        let conn = test_conn();
        let clip = save(&conn, "reused snippet");
        assert_eq!(clip.copy_count, 0);
        increment_copy_count(&conn, &clip.id).unwrap();
        increment_copy_count(&conn, &clip.id).unwrap();
        assert_eq!(recent(&conn)[0].copy_count, 2);
    }

    #[test]
    fn purge_expired_deletes_only_past_expiries() {
        let conn = test_conn();
        let gone = save(&conn, "temp token");
        let stays = save(&conn, "keep me");
        let later = save(&conn, "expires later");

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        set_expiry(&conn, &gone.id, Some(now - 5)).unwrap();
        set_expiry(&conn, &later.id, Some(now + 3600)).unwrap();

        let purged = purge_expired(&conn).unwrap();
        assert_eq!(purged, vec![gone.id]);

        let ids: Vec<String> = recent(&conn).into_iter().map(|c| c.id).collect();
        assert!(ids.contains(&stays.id));
        assert!(ids.contains(&later.id));
        assert_eq!(ids.len(), 2);

        // FTS row went with it.
        let fts_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM clips_fts", [], |r| r.get(0))
            .unwrap();
        assert_eq!(fts_count, 2);
    }

    #[test]
    fn deletes_from_both_tables() {
        let conn = test_conn();
        let a = save(&conn, "keep me");
        let b = save(&conn, "delete me");

        delete_clips(&conn, std::slice::from_ref(&b.id)).unwrap();

        let remaining = recent(&conn);
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, a.id);

        let fts_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM clips_fts WHERE id = ?1",
                [&b.id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            fts_count, 0,
            "deleted clip should be removed from the FTS index too"
        );
    }

    #[test]
    fn bulk_deletes_multiple_ids() {
        let conn = test_conn();
        let a = save(&conn, "one");
        let b = save(&conn, "two");
        let c = save(&conn, "three");

        delete_clips(&conn, &[a.id, c.id]).unwrap();

        let remaining = recent(&conn);
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, b.id);
    }

    #[test]
    fn empty_id_list_is_a_no_op() {
        let conn = test_conn();
        save(&conn, "untouched");

        delete_clips(&conn, &[]).unwrap();

        assert_eq!(recent(&conn).len(), 1);
    }

    #[test]
    fn under_the_limit_evicts_nothing() {
        let conn = test_conn();
        for i in 0..5 {
            save(&conn, &format!("clip {i}"));
        }
        assert_eq!(enforce_free_tier_limit(&conn, 500).unwrap().len(), 0);
        assert_eq!(recent(&conn).len(), 5);
    }

    #[test]
    fn over_the_limit_evicts_oldest_first() {
        let conn = test_conn();
        // save_clip already calls enforce_free_tier_limit on every insert
        // with the real FREE_TIER_LIMIT — use a tiny limit directly here
        // to keep the test fast instead of inserting 501 rows.
        for i in 0..5 {
            let id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO clips (id, content, timestamp) VALUES (?1, ?2, ?3)",
                (&id, format!("clip {i}"), i as i64),
            )
            .unwrap();
        }
        let evicted = enforce_free_tier_limit(&conn, 3).unwrap();
        assert_eq!(evicted.len(), 2);

        let remaining = recent(&conn);
        assert_eq!(remaining.len(), 3);
        // The two oldest (timestamps 0 and 1) should be gone.
        assert!(remaining.iter().all(|c| c.timestamp >= 2));
    }

    #[test]
    fn favorited_clips_are_never_evicted() {
        let conn = test_conn();
        let pinned = save(&conn, "pinned");
        set_favorite(&conn, &pinned.id, true).unwrap();
        for i in 0..4 {
            let id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO clips (id, content, timestamp) VALUES (?1, ?2, ?3)",
                (&id, format!("clip {i}"), (i + 1) as i64),
            )
            .unwrap();
        }
        // 5 total clips, limit of 3 — normally the 2 oldest would go, but
        // the pinned one (timestamp 0, oldest of all) must survive.
        enforce_free_tier_limit(&conn, 3).unwrap();

        let remaining = recent(&conn);
        assert!(
            remaining.iter().any(|c| c.id == pinned.id),
            "pinned clip must survive eviction"
        );
    }

    #[test]
    fn workspaces_isolate_history() {
        let conn = test_conn();
        save_clip(
            &conn,
            "personal clip",
            ContentType::PlainText,
            "app",
            "Personal",
        )
        .unwrap();
        save_clip(&conn, "work clip", ContentType::PlainText, "app", "Work").unwrap();

        assert_eq!(get_recent_clips(&conn, "Personal", 100).unwrap().len(), 1);
        assert_eq!(get_recent_clips(&conn, "Work", 100).unwrap().len(), 1);
        assert_eq!(
            get_recent_clips(&conn, "Personal", 100).unwrap()[0].preview,
            "personal clip"
        );
    }

    #[test]
    fn switch_workspace_updates_current() {
        let conn = test_conn();
        assert_eq!(get_current_workspace(&conn).unwrap(), "Personal");
        switch_workspace(&conn, "Work").unwrap();
        assert_eq!(get_current_workspace(&conn).unwrap(), "Work");

        let list = list_workspaces(&conn).unwrap();
        assert!(list.iter().find(|w| w.name == "Work").unwrap().is_current);
        assert!(
            !list
                .iter()
                .find(|w| w.name == "Personal")
                .unwrap()
                .is_current
        );
    }

    #[test]
    fn create_workspace_enforces_max_five() {
        let conn = test_conn();
        for name in ["W3", "W4", "W5"] {
            assert!(create_workspace(&conn, name).unwrap().is_ok());
        }
        // Already at 5 (Personal, Work, W3, W4, W5).
        assert!(create_workspace(&conn, "W6").unwrap().is_err());
    }

    #[test]
    fn create_workspace_rejects_duplicate_name() {
        let conn = test_conn();
        assert!(create_workspace(&conn, "Personal").unwrap().is_err());
    }

    #[test]
    fn delete_workspace_removes_it_and_its_clips() {
        let conn = test_conn();
        save_clip(&conn, "work clip", ContentType::PlainText, "app", "Work").unwrap();

        delete_workspace(&conn, "Work").unwrap().unwrap();

        assert!(!list_workspaces(&conn)
            .unwrap()
            .iter()
            .any(|w| w.name == "Work"));
        assert_eq!(get_recent_clips(&conn, "Work", 100).unwrap().len(), 0);
    }

    #[test]
    fn deleting_current_workspace_falls_back_to_another() {
        let conn = test_conn();
        switch_workspace(&conn, "Work").unwrap();

        delete_workspace(&conn, "Work").unwrap().unwrap();

        assert_eq!(get_current_workspace(&conn).unwrap(), "Personal");
    }

    #[test]
    fn cannot_delete_the_only_remaining_workspace() {
        let conn = test_conn();
        delete_workspace(&conn, "Work").unwrap().unwrap();

        let result = delete_workspace(&conn, "Personal").unwrap();
        assert!(result.is_err());
        assert_eq!(list_workspaces(&conn).unwrap().len(), 1);
    }

    #[test]
    fn collections_group_and_sample_clips() {
        let conn = test_conn();
        let a = save(&conn, "SELECT * FROM users");
        let b = save(&conn, "SELECT * FROM orders");
        save(&conn, "unrelated");
        assign_collection(&conn, &a.id, Some("Work / SQL")).unwrap();
        assign_collection(&conn, &b.id, Some("Work / SQL")).unwrap();

        let collections = list_collections(&conn).unwrap();
        assert_eq!(collections.len(), 1);
        assert_eq!(collections[0].name, "Work / SQL");
        assert_eq!(collections[0].count, 2);
        assert_eq!(collections[0].samples.len(), 2);
    }

    #[test]
    fn add_tag_is_idempotent() {
        let conn = test_conn();
        let clip = save(&conn, "tag me");
        add_tag(&conn, &clip.id, "dev").unwrap();
        add_tag(&conn, &clip.id, "dev").unwrap();
        add_tag(&conn, &clip.id, "prod").unwrap();

        let tags: String = conn
            .query_row("SELECT tags FROM clips WHERE id = ?1", [&clip.id], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(parse_tags(&tags), vec!["dev", "prod"]);
    }

    #[test]
    fn remove_tag_drops_only_that_tag() {
        let conn = test_conn();
        let clip = save(&conn, "tag me");
        add_tag(&conn, &clip.id, "dev").unwrap();
        add_tag(&conn, &clip.id, "prod").unwrap();
        remove_tag(&conn, &clip.id, "dev").unwrap();

        let tags: String = conn
            .query_row("SELECT tags FROM clips WHERE id = ?1", [&clip.id], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(parse_tags(&tags), vec!["prod"]);
    }
}
