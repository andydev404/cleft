use crate::ax;
use crate::clipboard::DbState;
use crate::db;
use serde::Serialize;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Serialize, Clone)]
struct ClipContextUpdate {
    id: String,
    window_title: Option<String>,
    url: Option<String>,
}

/// Fetches window title + URL on its own thread with a hard 50ms timeout,
/// so a slow or unresponsive app's Accessibility tree never delays the
/// clip that's already been saved. Times out → the clip just keeps its
/// empty context fields. No retry, no error surfaced anywhere.
pub fn spawn_fetch(app_handle: AppHandle, clip_id: String, pid: i32) {
    thread::spawn(move || {
        if !ax::is_trusted() {
            return;
        }

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let _ = tx.send(ax::fetch_context(pid));
        });

        let Ok(ctx) = rx.recv_timeout(Duration::from_millis(50)) else {
            return;
        };
        if ctx.window_title.is_none() && ctx.url.is_none() {
            return;
        }

        let state = app_handle.state::<DbState>();
        let conn = state.0.lock().unwrap();
        let _ = db::update_clip_context(
            &conn,
            &clip_id,
            ctx.window_title.as_deref(),
            ctx.url.as_deref(),
        );
        drop(conn);

        app_handle
            .emit(
                "clip-context-updated",
                ClipContextUpdate {
                    id: clip_id,
                    window_title: ctx.window_title,
                    url: ctx.url,
                },
            )
            .ok();
    });
}
