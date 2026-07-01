pub struct FrontmostApp {
    pub bundle_id: String,
    pub pid: i32,
}

// NSWorkspace.frontmostApplication requires no macOS permission at all —
// unlike AppleScript's "tell application" (never used, see sensitive.rs),
// this is a plain OS query.
#[cfg(target_os = "macos")]
pub fn current() -> Option<FrontmostApp> {
    use objc2_app_kit::NSWorkspace;

    let workspace = NSWorkspace::sharedWorkspace();
    let app = workspace.frontmostApplication()?;
    let bundle_id = app.bundleIdentifier()?.to_string();
    let pid = app.processIdentifier();
    Some(FrontmostApp { bundle_id, pid })
}

#[cfg(not(target_os = "macos"))]
pub fn current() -> Option<FrontmostApp> {
    None
}
