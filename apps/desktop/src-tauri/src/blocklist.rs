use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[cfg(target_os = "macos")]
const TEMPLATE: &str = "\
# Add bundle IDs of apps you never want captured
# Find an app's bundle ID: mdls -name kMDItemCFBundleIdentifier /Applications/AppName.app
#
# com.apple.Terminal
# com.iterm2.iTerm2
# com.yourcompany.internal-tool
";

// Windows has no bundle-id concept — the identifier here is the
// lowercased executable name (see frontmost_app.rs), which is what a
// user would find in Task Manager's "Details" tab.
#[cfg(not(target_os = "macos"))]
const TEMPLATE: &str = "\
# Add executable names of apps you never want captured (lowercase, as
# shown in Task Manager's Details tab)
#
# cmd.exe
# powershell.exe
# internal-tool.exe
";

fn path(app_handle: &AppHandle) -> PathBuf {
    app_handle
        .path()
        .app_data_dir()
        .expect("app data dir should be resolvable")
        .join("blocklist.txt")
}

pub fn ensure_exists(app_handle: &AppHandle) {
    let path = path(app_handle);
    if path.exists() {
        return;
    }
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(path, TEMPLATE);
}

fn is_blocked_content(contents: &str, bundle_id: &str) -> bool {
    contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .any(|line| line == bundle_id)
}

// Re-reads and re-parses the file on every poll cycle instead of caching by
// mtime — a few-hundred-byte local text file read every 250ms is free, and
// it's what makes editing the file while the app is running take effect
// live. Add mtime-based caching only if profiling says otherwise.
pub fn is_blocked(app_handle: &AppHandle, bundle_id: &str) -> bool {
    let Ok(contents) = std::fs::read_to_string(path(app_handle)) else {
        return false;
    };
    is_blocked_content(&contents, bundle_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_uncommented_bundle_id() {
        let content = "# comment\ncom.apple.Terminal\n\n# com.iterm2.iTerm2\n";
        assert!(is_blocked_content(content, "com.apple.Terminal"));
        assert!(!is_blocked_content(content, "com.iterm2.iTerm2"));
        assert!(!is_blocked_content(content, "com.other.app"));
    }
}
