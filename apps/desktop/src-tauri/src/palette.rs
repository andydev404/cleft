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

fn mark_shown(app: &AppHandle) {
    app.state::<LastShown>()
        .0
        .store(now_millis(), Ordering::Relaxed);
}

fn within_grace_period(last_shown: &LastShown) -> bool {
    now_millis() - last_shown.0.load(Ordering::Relaxed) < SHOW_GRACE_MS
}

// macOS: the palette is a non-activating NSPanel, not a regular NSWindow.
// A regular window of an activating app can never join another app's
// fullscreen Space — CanJoinAllSpaces/FullScreenAuxiliary flags or not
// (verified empirically: isOnActiveSpace stayed false with behavior=257).
// A NonActivatingPanel becomes key and displays over fullscreen apps
// without activating us, which is how Spotlight/Raycast behave.
#[cfg(target_os = "macos")]
#[allow(deprecated)] // tauri-nspanel re-exports the legacy cocoa crate's types
mod platform {
    use super::*;
    use tauri_nspanel::cocoa::appkit::NSWindowCollectionBehavior;
    use tauri_nspanel::{panel_delegate, ManagerExt, WebviewWindowExt};

    const NS_WINDOW_STYLE_MASK_NON_ACTIVATING_PANEL: i32 = 1 << 7;

    // Called once from setup, after vibrancy is applied.
    pub fn init_panel(app: &AppHandle) {
        let window = app
            .get_webview_window(PALETTE_WINDOW)
            .expect("palette window should exist");
        let panel = window
            .to_panel()
            .expect("failed to convert palette window to NSPanel");

        panel.set_style_mask(NS_WINDOW_STYLE_MASK_NON_ACTIVATING_PANEL);
        panel.set_collection_behaviour(
            NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary,
        );

        // Swizzling replaces tao's window delegate, so WindowEvent::Focused
        // no longer reaches on_window_event — dismiss-on-blur lives here.
        let delegate = panel_delegate!(PalettePanelDelegate {
            window_did_resign_key
        });
        let handle = app.clone();
        delegate.set_listener(Box::new(move |name: String| {
            if name.as_str() == "window_did_resign_key"
                && !within_grace_period(&handle.state::<LastShown>())
            {
                if let Ok(panel) = handle.get_webview_panel(PALETTE_WINDOW) {
                    panel.order_out(None);
                }
            }
        }));
        panel.set_delegate(delegate);
    }

    pub fn show_palette(app: &AppHandle) {
        let Ok(panel) = app.get_webview_panel(PALETTE_WINDOW) else {
            return;
        };
        mark_shown(app);
        // orderFrontRegardless + makeKeyWindow — displays on the active
        // Space (fullscreen included) without activating the app.
        panel.show();
        app.emit("palette-shown", ()).ok();
    }

    pub fn toggle_palette(app: &AppHandle) {
        let Ok(panel) = app.get_webview_panel(PALETTE_WINDOW) else {
            return;
        };
        if panel.is_visible() {
            if within_grace_period(&app.state::<LastShown>()) {
                return;
            }
            panel.order_out(None);
        } else {
            show_palette(app);
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use super::*;

    pub fn show_palette(app: &AppHandle) {
        let Some(window) = app.get_webview_window(PALETTE_WINDOW) else {
            return;
        };
        mark_shown(app);
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
}

pub use platform::*;

// Dismisses on focus loss — instant hide, skipped within the grace window.
// On macOS this never fires (the panel delegate handles it instead); it's
// the live path on Windows.
pub fn handle_window_event(window: &tauri::Window, event: &WindowEvent) {
    if let WindowEvent::Focused(false) = event {
        if !within_grace_period(&window.state::<LastShown>()) {
            window.hide().ok();
        }
    }
}
