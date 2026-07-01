use crate::blocklist;
use crate::classifier;
use crate::context;
use crate::db;
use crate::frontmost_app;
use crate::pasteboard;
use crate::sensitive;
use rusqlite::Connection;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;

pub struct DbState(pub Mutex<Connection>);

pub fn start_monitor(app_handle: AppHandle) {
    thread::spawn(move || {
        let mut last_change_count: isize = -1;

        loop {
            thread::sleep(Duration::from_millis(250));

            // Update the "have we seen this pasteboard state" marker before
            // any blocking decision — see pasteboard.rs for why that order
            // matters.
            let count = pasteboard::change_count();
            if count == last_change_count {
                continue;
            }
            last_change_count = count;

            let frontmost = frontmost_app::current();
            let bundle_id = frontmost
                .as_ref()
                .map(|a| a.bundle_id.clone())
                .unwrap_or_default();

            // Layer 1 — hardcoded blacklist. Skip before the clipboard is
            // ever read: no content touches the process, no log entry.
            if sensitive::is_blocked_app(&bundle_id) {
                continue;
            }
            // Layer 2 — user-editable blocklist. Same firing position as
            // Layer 1: still before the clipboard is read.
            if blocklist::is_blocked(&app_handle, &bundle_id) {
                continue;
            }

            let Ok(text) = app_handle.clipboard().read_text() else {
                continue;
            };
            if text.is_empty() {
                continue;
            }

            // Layer 3 — content pattern detection. Zero-log: falls straight
            // through to the next poll, nothing written anywhere.
            if sensitive::is_sensitive(&text, &bundle_id) {
                continue;
            }

            let content_type = classifier::classify(&text);

            let state = app_handle.state::<DbState>();
            let conn = state.0.lock().unwrap();
            let Ok(metadata) = db::save_clip(&conn, &text, content_type, &bundle_id) else {
                continue;
            };
            drop(conn);

            let clip_id = metadata.id.clone();
            app_handle.emit("clip-added", metadata).ok();

            // Context capture: separate thread, 50ms hard timeout, never
            // blocks the save/emit above. See context.rs.
            if let Some(pid) = frontmost.map(|a| a.pid) {
                context::spawn_fetch(app_handle.clone(), clip_id, pid);
            }
        }
    });
}
