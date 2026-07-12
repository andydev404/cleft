use crate::classifier::ContentType;
use crate::db::{row_to_metadata, ClipMetadata, CLIP_COLUMNS};
use regex::Regex;
use rusqlite::Connection;
use std::sync::LazyLock;

// Collection names routinely contain spaces ("Work / SQL Queries"), unlike
// type:/app: values — so `in:"..."` needs to be pulled out before the query
// is split on whitespace, not handled word-by-word like the other filters.
static IN_FILTER_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"in:"([^"]*)""#).unwrap());

// FTS5's default unicode61 tokenizer already splits snake_case and
// kebab-case into separate tokens (underscore/hyphen aren't "letters" in
// its classification) — verified directly, not assumed. camelCase is the
// one case it doesn't handle, so that's the only preprocessing needed
// before text goes into the FTS index or a query.
pub fn split_camel_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    let mut prev_lower = false;
    for c in s.chars() {
        if prev_lower && c.is_uppercase() {
            out.push(' ');
        }
        out.push(c);
        prev_lower = c.is_lowercase();
    }
    out
}

fn content_type_from_alias(alias: &str) -> Option<ContentType> {
    let ct = match alias.to_lowercase().as_str() {
        "plaintext" | "text" => ContentType::PlainText,
        "code" => ContentType::Code,
        "url" | "link" => ContentType::Url,
        "sql" => ContentType::Sql,
        "json" => ContentType::Json,
        "markdown" | "md" => ContentType::Markdown,
        "color" => ContentType::Color,
        "email" => ContentType::Email,
        "filepath" | "path" => ContentType::FilePath,
        "html" => ContentType::Html,
        _ => return None,
    };
    Some(ct)
}

struct ParsedQuery {
    terms: Vec<String>,
    content_type: Option<ContentType>,
    app: Option<String>,
    collection: Option<String>,
    tag: Option<String>,
}

// Supports `type:sql`, `app:tableplus`, `tag:dev`, and `in:"collection name"`
// filters inline in the query text; everything else becomes free-text search
// terms. `after:`/`before:` (dates) are still deferred — they need their own
// UI affordance, not just parsing.
fn parse_query(query: &str) -> ParsedQuery {
    let mut terms = Vec::new();
    let mut content_type = None;
    let mut app = None;
    let mut tag = None;

    let (collection, rest) = match IN_FILTER_RE.captures(query) {
        Some(caps) => (
            Some(caps[1].to_string()),
            IN_FILTER_RE.replace(query, "").to_string(),
        ),
        None => (None, query.to_string()),
    };

    for word in rest.split_whitespace() {
        if let Some(value) = word.strip_prefix("type:") {
            content_type = content_type_from_alias(value);
        } else if let Some(value) = word.strip_prefix("app:") {
            app = Some(value.to_lowercase());
        } else if let Some(value) = word.strip_prefix("tag:") {
            tag = Some(value.to_string());
        } else {
            terms.push(split_camel_case(word));
        }
    }

    ParsedQuery {
        terms,
        content_type,
        app,
        collection,
        tag,
    }
}

pub fn search_clips(
    conn: &Connection,
    workspace: &str,
    query: &str,
    limit: i64,
) -> rusqlite::Result<Vec<ClipMetadata>> {
    let parsed = parse_query(query);

    let mut filter_sql = String::new();
    let mut filter_params: Vec<Box<dyn rusqlite::types::ToSql>> =
        vec![Box::new(workspace.to_string())];
    if let Some(ct) = parsed.content_type {
        filter_sql.push_str(" AND clips.content_type = ?");
        filter_params.push(Box::new(ct.as_str().to_string()));
    }
    if let Some(app) = &parsed.app {
        filter_sql.push_str(" AND lower(clips.source_app) LIKE ?");
        filter_params.push(Box::new(format!("%{app}%")));
    }
    if let Some(collection) = &parsed.collection {
        filter_sql.push_str(" AND clips.collection = ?");
        filter_params.push(Box::new(collection.clone()));
    }
    if let Some(tag) = &parsed.tag {
        // Tags are stored as a single comma-joined column, not a normalized
        // table — wrapping both sides in commas before the LIKE turns a
        // substring check into an exact-element check, so "dev" doesn't
        // also match "development".
        filter_sql.push_str(" AND (',' || lower(clips.tags) || ',') LIKE ?");
        filter_params.push(Box::new(format!("%,{},%", tag.to_lowercase())));
    }

    if parsed.terms.is_empty() {
        // Filters only (or an empty query) — plain browse, most recent first.
        let sql = format!(
            "SELECT {CLIP_COLUMNS} FROM clips WHERE workspace = ?{filter_sql} ORDER BY timestamp DESC LIMIT ?"
        );
        let mut stmt = conn.prepare(&sql)?;
        filter_params.push(Box::new(limit));
        let params: Vec<&dyn rusqlite::types::ToSql> =
            filter_params.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(params.as_slice(), row_to_metadata)?;
        return rows.collect();
    }

    // Each term becomes a quoted prefix phrase (`"foo"*`) so search-as-you-
    // type matches partial words; space between phrases is FTS5's implicit
    // AND.
    let match_query = parsed
        .terms
        .iter()
        .map(|t| format!("\"{}\"*", t.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" ");

    let sql = format!(
        "SELECT clips.id, clips.content, clips.content_type, clips.source_app,
                clips.window_title, clips.url, clips.timestamp, clips.is_favorite,
                clips.workspace, clips.tags, clips.collection
         FROM clips_fts
         JOIN clips ON clips.id = clips_fts.id
         WHERE clips_fts MATCH ? AND clips.workspace = ?{filter_sql}
         ORDER BY bm25(clips_fts) LIMIT ?"
    );
    let mut stmt = conn.prepare(&sql)?;
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(match_query)];
    params.extend(filter_params);
    params.push(Box::new(limit));
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(param_refs.as_slice(), row_to_metadata)?;
    rows.collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_camel_case() {
        assert_eq!(split_camel_case("dockerCompose"), "docker Compose");
        assert_eq!(split_camel_case("plain"), "plain");
    }

    #[test]
    fn parses_type_and_app_filters() {
        let parsed = parse_query("type:sql app:tableplus prod users");
        assert_eq!(parsed.content_type, Some(ContentType::Sql));
        assert_eq!(parsed.app, Some("tableplus".to_string()));
        assert_eq!(parsed.terms, vec!["prod", "users"]);
    }

    #[test]
    fn parses_collection_filter() {
        let parsed = parse_query(r#"in:"Work / SQL Queries" prod"#);
        assert_eq!(parsed.collection, Some("Work / SQL Queries".to_string()));
        assert_eq!(parsed.terms, vec!["prod"]);
    }

    #[test]
    fn parses_tag_filter() {
        let parsed = parse_query("tag:dev prod");
        assert_eq!(parsed.tag, Some("dev".to_string()));
        assert_eq!(parsed.terms, vec!["prod"]);
    }

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
        conn
    }

    #[test]
    fn tag_filter_matches_exact_tag_only() {
        use crate::db;
        let conn = test_conn();

        let (dev_clip, _) =
            db::save_clip(&conn, "select 1", ContentType::Sql, "app", "Personal").unwrap();
        db::add_tag(&conn, &dev_clip.id, "dev").unwrap();
        let (development_clip, _) =
            db::save_clip(&conn, "select 2", ContentType::Sql, "app", "Personal").unwrap();
        db::add_tag(&conn, &development_clip.id, "development").unwrap();

        let results = search_clips(&conn, "Personal", "tag:dev", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, dev_clip.id);
    }
}
