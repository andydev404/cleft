use std::path::PathBuf;
use tauri::{AppHandle, Manager};

// The palette window is hidden by default and only reveals on the global
// shortcut — except for the very first launch, where the Permission Runway
// needs to be seen without the user knowing the shortcut yet. This empty
// marker file is how the app remembers "the first-launch reveal already
// happened" across restarts.
fn marker_path(app_handle: &AppHandle) -> PathBuf {
    app_handle
        .path()
        .app_data_dir()
        .expect("app data dir should be resolvable")
        .join("onboarded")
}

pub fn is_onboarded(app_handle: &AppHandle) -> bool {
    marker_path(app_handle).exists()
}

pub fn mark_onboarded(app_handle: &AppHandle) {
    let path = marker_path(app_handle);
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(path, "");
}
