use regex::Regex;
use serde::Serialize;
use std::sync::LazyLock;

// ponytail: `Image` is left out of this enum — nothing in the capture
// pipeline reads image data yet (clipboard.rs only calls read_text()).
// Add it once image capture lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ContentType {
    PlainText,
    Code,
    #[serde(rename = "URL")]
    Url,
    #[serde(rename = "SQL")]
    Sql,
    #[serde(rename = "JSON")]
    Json,
    Markdown,
    Color,
    Email,
    FilePath,
    #[serde(rename = "HTML")]
    Html,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::PlainText => "PlainText",
            ContentType::Code => "Code",
            ContentType::Url => "URL",
            ContentType::Sql => "SQL",
            ContentType::Json => "JSON",
            ContentType::Markdown => "Markdown",
            ContentType::Color => "Color",
            ContentType::Email => "Email",
            ContentType::FilePath => "FilePath",
            ContentType::Html => "HTML",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Code" => ContentType::Code,
            "URL" => ContentType::Url,
            "SQL" => ContentType::Sql,
            "JSON" => ContentType::Json,
            "Markdown" => ContentType::Markdown,
            "Color" => ContentType::Color,
            "Email" => ContentType::Email,
            "FilePath" => ContentType::FilePath,
            "HTML" => ContentType::Html,
            _ => ContentType::PlainText,
        }
    }
}

static HEX_COLOR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^#([0-9a-f]{3,4}|[0-9a-f]{6}|[0-9a-f]{8})$").unwrap());
static FN_COLOR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^(rgb|rgba|hsl|hsla)\([^)]+\)$").unwrap());
static EMAIL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[\w.+-]+@[\w-]+\.[\w.-]+$").unwrap());
static URL_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(https?://|www\.)\S+$").unwrap());
static UNIX_PATH_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(~|\.{1,2})?/(?:[^/\s]+/)*[^/\s]+$").unwrap());
static WINDOWS_PATH_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Za-z]:\\(?:[^\\\s]+\\)*[^\\\s]+$").unwrap());
static SQL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^\s*(SELECT|INSERT\s+INTO|UPDATE|DELETE\s+FROM|CREATE\s+TABLE|ALTER\s+TABLE|DROP\s+TABLE|WITH)\b").unwrap()
});
static MARKDOWN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)(^#{1,6}\s)|(\*\*[^*]+\*\*)|(^[-*]\s)|(\[[^\]]+\]\([^)]+\))|(^```)").unwrap()
});
static CODE_KEYWORD_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(function|const|let|var|def|class|import|return|public|private|void|func|struct|impl)\b").unwrap()
});

fn is_json(trimmed: &str) -> bool {
    (trimmed.starts_with('{') || trimmed.starts_with('['))
        && serde_json::from_str::<serde_json::Value>(trimmed).is_ok()
}

fn is_html(trimmed: &str, content: &str) -> bool {
    trimmed.starts_with('<') && content.contains("</")
}

fn looks_like_code(content: &str) -> bool {
    let has_keyword = CODE_KEYWORD_RE.is_match(content);
    let has_braces = content.contains('{') && content.contains('}');
    let has_semicolons = content.lines().filter(|l| l.trim_end().ends_with(';')).count() >= 2;
    has_keyword && (has_braces || has_semicolons)
}

/// Rule-based classifier — no ML, no model files. Runs at capture time,
/// before the clip is saved.
pub fn classify(content: &str) -> ContentType {
    let trimmed = content.trim();

    if HEX_COLOR_RE.is_match(trimmed) || FN_COLOR_RE.is_match(trimmed) {
        return ContentType::Color;
    }
    if EMAIL_RE.is_match(trimmed) {
        return ContentType::Email;
    }
    if URL_RE.is_match(trimmed) {
        return ContentType::Url;
    }
    if UNIX_PATH_RE.is_match(trimmed) || WINDOWS_PATH_RE.is_match(trimmed) {
        return ContentType::FilePath;
    }
    if is_json(trimmed) {
        return ContentType::Json;
    }
    if is_html(trimmed, content) {
        return ContentType::Html;
    }
    if SQL_RE.is_match(content) {
        return ContentType::Sql;
    }
    if MARKDOWN_RE.is_match(content) {
        return ContentType::Markdown;
    }
    if looks_like_code(content) {
        return ContentType::Code;
    }

    ContentType::PlainText
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_hex_color() {
        assert_eq!(classify("#2563EB"), ContentType::Color);
        assert_eq!(classify("#fff"), ContentType::Color);
    }

    #[test]
    fn detects_function_color() {
        assert_eq!(classify("rgba(37, 99, 235, 0.5)"), ContentType::Color);
    }

    #[test]
    fn detects_email() {
        assert_eq!(classify("dev@example.com"), ContentType::Email);
    }

    #[test]
    fn detects_url() {
        assert_eq!(
            classify("https://github.com/org/repo/issues/42"),
            ContentType::Url
        );
    }

    #[test]
    fn detects_unix_path() {
        assert_eq!(classify("/Users/dev/project/main.rs"), ContentType::FilePath);
    }

    #[test]
    fn detects_json() {
        assert_eq!(classify(r#"{"id": 1, "name": "test"}"#), ContentType::Json);
    }

    #[test]
    fn detects_html() {
        assert_eq!(classify("<div class=\"card\">hi</div>"), ContentType::Html);
    }

    #[test]
    fn detects_sql() {
        assert_eq!(
            classify("SELECT * FROM users WHERE id = 1"),
            ContentType::Sql
        );
    }

    #[test]
    fn detects_markdown() {
        assert_eq!(classify("# Heading\n\nSome **bold** text"), ContentType::Markdown);
    }

    #[test]
    fn detects_code() {
        let snippet = "function add(a, b) {\n  return a + b;\n}";
        assert_eq!(classify(snippet), ContentType::Code);
    }

    #[test]
    fn falls_back_to_plain_text() {
        assert_eq!(
            classify("just a normal sentence with no structure"),
            ContentType::PlainText
        );
    }
}
