mod ax;
mod blocklist;
mod classifier;
mod clipboard;
mod context;
mod db;
mod frontmost_app;
mod keychain;
mod markdown;
mod onboarding;
mod pasteboard;
mod search;
mod sensitive;

use clipboard::DbState;
use db::ClipMetadata;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, WindowEvent};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

const PALETTE_WINDOW: &str = "main";
// macOS occasionally fires a spurious Focused(false) right as a borderless
// always-on-top window is shown and given focus, before Focused(true)
// settles — observed directly while testing this, not a hypothetical.
// Ignoring focus-loss for a brief window after we ourselves showed it
// avoids the show() being immediately undone by our own blur handler.
const FOCUS_LOSS_GRACE_MS: i64 = 300;

struct LastShown(AtomicI64);

fn now_millis() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
}

// Window is pre-rendered and hidden — toggling visibility reveals it
// instantly rather than spawning anything new, matching the < 100ms target.
fn toggle_palette(app: &AppHandle) {
    let Some(window) = app.get_webview_window(PALETTE_WINDOW) else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        window.hide().ok();
    } else {
        app.state::<LastShown>().0.store(now_millis(), Ordering::Relaxed);
        window.show().ok();
        window.set_focus().ok();
        app.emit("palette-shown", ()).ok();
    }
}

#[tauri::command]
fn get_history(state: tauri::State<DbState>) -> Vec<ClipMetadata> {
    let conn = state.0.lock().unwrap();
    db::get_recent_clips(&conn, 500).unwrap_or_default()
}

#[tauri::command]
fn search_clips(query: String, state: tauri::State<DbState>) -> Vec<ClipMetadata> {
    let conn = state.0.lock().unwrap();
    search::search_clips(&conn, &query, 200).unwrap_or_default()
}

// The one place the IPC contract allows full content across the bridge:
// an explicit, single-clip fetch by primary key, only on selection.
#[tauri::command]
fn get_clip_content(id: String, state: tauri::State<DbState>) -> Option<String> {
    let conn = state.0.lock().unwrap();
    db::get_clip_content(&conn, &id).unwrap_or_default()
}

#[tauri::command]
fn render_markdown(content: String) -> String {
    markdown::render(&content)
}

// User-initiated paste-back — distinct from the capture path, which never
// touches the clipboard from the frontend. Routed through Rust like every
// other clipboard interaction in this app, not the JS clipboard-manager API.
#[tauri::command]
fn copy_to_clipboard(text: String, app: AppHandle) {
    app.clipboard().write_text(text).ok();
}

#[tauri::command]
fn check_accessibility_trusted() -> bool {
    ax::is_trusted()
}

// Triggers the macOS system dialog if not already trusted. The Permission
// Runway (frontend) re-polls check_accessibility_trusted afterward, since
// the user's actual choice happens in System Settings, not synchronously.
#[tauri::command]
fn request_accessibility_trust() -> bool {
    ax::request_trust()
}

#[tauri::command]
fn mark_onboarded(app: AppHandle) {
    onboarding::mark_onboarded(&app);
}

// Esc dismisses the palette — no animation, matches focus-loss dismissal.
#[tauri::command]
fn hide_palette(window: tauri::Window) {
    window.hide().ok();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let shortcut = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyV);

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, scut, event| {
                    if *scut == shortcut && event.state() == ShortcutState::Pressed {
                        toggle_palette(app);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            get_history,
            search_clips,
            get_clip_content,
            render_markdown,
            copy_to_clipboard,
            check_accessibility_trusted,
            request_accessibility_trust,
            mark_onboarded,
            hide_palette
        ])
        .on_window_event(|window, event| {
            // Dismisses on focus loss — no animation, instant hide. Skipped
            // within FOCUS_LOSS_GRACE_MS of our own show(), see LastShown.
            if let WindowEvent::Focused(false) = event {
                let last_shown = window.state::<LastShown>().0.load(Ordering::Relaxed);
                if now_millis() - last_shown > FOCUS_LOSS_GRACE_MS {
                    window.hide().ok();
                }
            }
        })
        .setup(move |app| {
            app.manage(LastShown(AtomicI64::new(0)));
            blocklist::ensure_exists(app.handle());
            let conn = db::init_db(app.handle())?;
            app.manage(DbState(Mutex::new(conn)));
            clipboard::start_monitor(app.handle().clone());

            app.global_shortcut().register(shortcut)?;

            // First launch only: reveal immediately so the Permission
            // Runway is seen before the user knows the shortcut exists.
            if !onboarding::is_onboarded(app.handle()) {
                if let Some(window) = app.get_webview_window(PALETTE_WINDOW) {
                    app.state::<LastShown>().0.store(now_millis(), Ordering::Relaxed);
                    window.show()?;
                    window.set_focus()?;
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
