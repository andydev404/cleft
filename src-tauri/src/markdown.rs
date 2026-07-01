// pulldown-cmark passes raw HTML embedded in markdown straight through to
// its output (that's normal CommonMark behavior), and this content is
// captured from the clipboard — not authored by us. Sending it to the
// frontend for dangerouslySetInnerHTML without sanitizing first would let
// a clipped snippet like "<img src=x onerror=alert(1)>" run script inside
// the app's own webview. ammonia strips that down to a safe tag/attribute
// allowlist before it ever reaches the frontend.
pub fn render(content: &str) -> String {
    let parser = pulldown_cmark::Parser::new(content);
    let mut unsafe_html = String::new();
    pulldown_cmark::html::push_html(&mut unsafe_html, parser);
    ammonia::clean(&unsafe_html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_basic_formatting() {
        let html = render("# Heading\n\n**bold** and _italic_");
        assert!(html.contains("<h1>"));
        assert!(html.contains("<strong>bold</strong>"));
    }

    #[test]
    fn strips_script_tags() {
        let html = render("hello <script>alert(1)</script> world");
        assert!(!html.contains("<script"));
    }

    #[test]
    fn strips_event_handler_attributes() {
        let html = render(r#"<img src="x" onerror="alert(1)">"#);
        assert!(!html.contains("onerror"));
    }

    #[test]
    fn strips_javascript_scheme_links() {
        let html = render("[click me](javascript:alert(1))");
        assert!(!html.contains("javascript:"));
    }
}
