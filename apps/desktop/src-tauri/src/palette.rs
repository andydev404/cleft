use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, WindowEvent};

pub const PALETTE_WINDOW: &str = "main";

// Chromium browsers also bind ⌘⇧V and can bounce focus back within ~1-2s of
// show(), which the focus-loss handler would misread as "user clicked away"
// and hide the window. Ignore focus loss (and the shortcut's own hide path)
// within this grace window; Esc still closes instantly.
const SHOW_GRACE_MS: i64 = 2000;

#[derive(Default)]
pub struct LastShown(AtomicI64);

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

pub fn show_palette(app: &AppHandle) {
    let Some(window) = app.get_webview_window(PALETTE_WINDOW) else {
        return;
    };
    app.state::<LastShown>()
        .0
        .store(now_millis(), Ordering::Relaxed);
    window.show().ok();
    window.set_focus().ok();
    app.emit("palette-shown", ()).ok();
}

// The window is pre-rendered and hidden, so toggling visibility is instant.
pub fn toggle_palette(app: &AppHandle) {
    let Some(window) = app.get_webview_window(PALETTE_WINDOW) else {
        return;
    };

    if window.is_visible().unwrap_or(false) {
        if within_grace_period(&app.state::<LastShown>()) {
            return;
        }
        window.hide().ok();
    } else {
        show_palette(app);
    }
}

// Dismisses on focus loss — instant hide, skipped within the grace window.
pub fn handle_window_event(window: &tauri::Window, event: &WindowEvent) {
    if let WindowEvent::Focused(false) = event {
        if !within_grace_period(&window.state::<LastShown>()) {
            window.hide().ok();
        }
    }
}

fn within_grace_period(last_shown: &LastShown) -> bool {
    now_millis() - last_shown.0.load(Ordering::Relaxed) < SHOW_GRACE_MS
}
