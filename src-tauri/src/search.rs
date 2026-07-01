use crate::classifier::ContentType;
use crate::db::ClipMetadata;
use rusqlite::Connection;

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
}

// Supports `type:sql` and `app:tableplus` filters inline in the query text;
// everything else becomes free-text search terms. `after:`/`before:`/`in:`
// (dates, collections) are deferred — collections don't exist as a feature
// yet, and date filters need their own UI affordance, not just parsing.
fn parse_query(query: &str) -> ParsedQuery {
    let mut terms = Vec::new();
    let mut content_type = None;
    let mut app = None;

    for word in query.split_whitespace() {
        if let Some(value) = word.strip_prefix("type:") {
            content_type = content_type_from_alias(value);
        } else if let Some(value) = word.strip_prefix("app:") {
            app = Some(value.to_lowercase());
        } else {
            terms.push(split_camel_case(word));
        }
    }

    ParsedQuery { terms, content_type, app }
}

fn row_to_metadata(row: &rusqlite::Row) -> rusqlite::Result<ClipMetadata> {
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
}

pub fn search_clips(conn: &Connection, query: &str, limit: i64) -> rusqlite::Result<Vec<ClipMetadata>> {
    let parsed = parse_query(query);

    let mut filter_sql = String::new();
    let mut filter_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    if let Some(ct) = parsed.content_type {
        filter_sql.push_str(" AND clips.content_type = ?");
        filter_params.push(Box::new(ct.as_str().to_string()));
    }
    if let Some(app) = &parsed.app {
        filter_sql.push_str(" AND lower(clips.source_app) LIKE ?");
        filter_params.push(Box::new(format!("%{app}%")));
    }

    if parsed.terms.is_empty() {
        // Filters only (or an empty query) — plain browse, most recent first.
        let sql = format!(
            "SELECT id, content, content_type, source_app, window_title, url, timestamp
             FROM clips WHERE 1=1{filter_sql} ORDER BY timestamp DESC LIMIT ?"
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
                clips.window_title, clips.url, clips.timestamp
         FROM clips_fts
         JOIN clips ON clips.id = clips_fts.id
         WHERE clips_fts MATCH ?{filter_sql}
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
}
