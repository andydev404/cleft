use crate::automation::{self, CaptureContext};
use crate::ax;
use crate::blocklist;
use crate::classifier;
use crate::context;
use crate::db;
use crate::frontmost_app;
use crate::pasteboard;
use crate::sensitive;
use rusqlite::Connection;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;

pub struct DbState(pub Mutex<Connection>);

// Writing a clip back to the clipboard (the "Copy" button, ⌘1-9, paste
// stack, etc.) bumps NSPasteboard.changeCount exactly like a real copy
// from another app — the monitor loop below can't tell them apart by the
// counter alone. This records the changeCount our own write produced, so
// the loop can recognize "that change was us" and skip capturing it as a
// brand new clip. Without this, copying an existing clip re-added it to
// the list as a duplicate every time.
static SELF_WRITE_CHANGE_COUNT: AtomicIsize = AtomicIsize::new(-1);

pub fn mark_self_write() {
    SELF_WRITE_CHANGE_COUNT.store(pasteboard::change_count(), Ordering::Relaxed);
}

// Same bounded-fetch mechanic as context.rs's spawn_fetch, but synchronous:
// only called when an enabled automation rule actually needs window title
// or URL to evaluate its trigger, and only for up to 50ms, so capture
// latency is unaffected unless that kind of rule is in use.
fn fetch_context_bounded(pid: i32) -> (Option<String>, Option<String>) {
    if !ax::is_trusted() {
        return (None, None);
    }
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(ax::fetch_context(pid));
    });
    match rx.recv_timeout(Duration::from_millis(50)) {
        Ok(ctx) => (ctx.window_title, ctx.url),
        Err(_) => (None, None),
    }
}

pub fn start_monitor(app_handle: AppHandle) {
    thread::spawn(move || {
        let mut last_change_count: isize = -1;
        // NSWorkspace.frontmostApplication can transiently return nil —
        // confirmed live, most easily triggered right as our own palette
        // window activates (e.g. someone copies something then immediately
        // hits ⌘⇧V to check it). A capture landing in that gap would
        // otherwise lose bundle_id/pid entirely, permanently — no
        // automation trigger, no window title, no URL, for that clip.
        // Falling back to the last successfully-resolved frontmost app is
        // a far better guess than "unknown," since nothing else plausibly
        // changed focus in the interim.
        let mut last_known_frontmost: Option<frontmost_app::FrontmostApp> = None;

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

            if count == SELF_WRITE_CHANGE_COUNT.load(Ordering::Relaxed) {
                continue;
            }

            let frontmost = frontmost_app::current().or_else(|| last_known_frontmost.clone());
            if frontmost.is_some() {
                last_known_frontmost = frontmost.clone();
            }
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
            let pid = frontmost.map(|a| a.pid);

            let state = app_handle.state::<DbState>();
            let conn = state.0.lock().unwrap();

            // Rules run in the Rust core, before the clip is emitted.
            // Only fetch window title / URL synchronously (bounded,
            // 50ms) when a rule actually needs them for its trigger —
            // otherwise capture latency is unaffected, same as when
            // automation isn't in use at all.
            let rules = automation::list_rules(&conn).unwrap_or_default();
            let (window_title, url) = if automation::needs_window_context(&rules) {
                pid.map(fetch_context_bounded).unwrap_or((None, None))
            } else {
                (None, None)
            };
            let outcome = automation::evaluate(
                &rules,
                &CaptureContext {
                    bundle_id: &bundle_id,
                    content: &text,
                    content_type,
                    window_title: window_title.as_deref(),
                    url: url.as_deref(),
                },
            );
            // Extends Layer 2/3 with user-authored content rules — never
            // saved at all, same zero-log posture as the built-in layers.
            if outcome.block {
                continue;
            }

            let current_workspace = db::get_current_workspace(&conn)
                .unwrap_or_else(|_| db::DEFAULT_WORKSPACE.to_string());
            let workspace = outcome.workspace.as_deref().unwrap_or(&current_workspace);
            let Ok((mut metadata, evicted)) =
                db::save_clip(&conn, &text, content_type, &bundle_id, workspace)
            else {
                continue;
            };

            if let Some(collection) = &outcome.collection {
                if db::assign_collection(&conn, &metadata.id, Some(collection)).is_ok() {
                    metadata.collection = Some(collection.clone());
                }
            }
            for tag in &outcome.tags {
                if db::add_tag(&conn, &metadata.id, tag).is_ok() {
                    metadata.tags.push(tag.clone());
                }
            }
            if outcome.pin && db::set_favorite(&conn, &metadata.id, true).is_ok() {
                metadata.is_favorite = true;
            }
            drop(conn);

            let clip_id = metadata.id.clone();
            app_handle.emit("clip-added", metadata).ok();
            // The rolling FIFO evicted the oldest unpinned clip(s) to make
            // room — the frontend's in-memory list needs to drop them too,
            // or clicking one would silently 404 against get_clip_content.
            if !evicted.is_empty() {
                app_handle.emit("clips-evicted", evicted).ok();
            }

            // Context capture: separate thread, 50ms hard timeout, never
            // blocks the save/emit above. See context.rs. Still runs even
            // when the bounded fetch above already ran for automation —
            // that one wasn't persisted, this is the one that actually
            // updates window_title/url in the DB and over IPC.
            if let Some(pid) = pid {
                context::spawn_fetch(app_handle.clone(), clip_id, pid);
            }
        }
    });
}
