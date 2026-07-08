use std::sync::Mutex;
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_updater::UpdaterExt;

use crate::clipboard::{self, DbState};
use crate::db::{self, ClipMetadata};
use crate::{automation, ax, markdown, onboarding, search};

#[tauri::command]
pub fn get_history(state: tauri::State<DbState>) -> Vec<ClipMetadata> {
    let conn = state.0.lock().unwrap();
    let workspace =
        db::get_current_workspace(&conn).unwrap_or_else(|_| db::DEFAULT_WORKSPACE.to_string());
    db::get_recent_clips(&conn, &workspace, 500).unwrap_or_default()
}

#[tauri::command]
pub fn search_clips(query: String, state: tauri::State<DbState>) -> Vec<ClipMetadata> {
    let conn = state.0.lock().unwrap();
    let workspace =
        db::get_current_workspace(&conn).unwrap_or_else(|_| db::DEFAULT_WORKSPACE.to_string());
    search::search_clips(&conn, &workspace, &query, 200).unwrap_or_default()
}

// The only place full clip content crosses the IPC bridge — an explicit
// single-clip fetch on selection; list endpoints only carry previews.
#[tauri::command]
pub fn get_clip_content(id: String, state: tauri::State<DbState>) -> Option<String> {
    let conn = state.0.lock().unwrap();
    db::get_clip_content(&conn, &id).unwrap_or_default()
}

#[tauri::command]
pub fn render_markdown(content: String) -> String {
    markdown::render(&content)
}

#[tauri::command]
pub fn delete_clips(ids: Vec<String>, state: tauri::State<DbState>) {
    let conn = state.0.lock().unwrap();
    db::delete_clips(&conn, &ids).ok();
}

// Persisted because FIFO eviction (db.rs) must never evict pinned clips.
#[tauri::command]
pub fn set_favorite(id: String, favorite: bool, state: tauri::State<DbState>) {
    let conn = state.0.lock().unwrap();
    db::set_favorite(&conn, &id, favorite).ok();
}

#[tauri::command]
pub fn list_workspaces(state: tauri::State<DbState>) -> Vec<db::Workspace> {
    let conn = state.0.lock().unwrap();
    db::list_workspaces(&conn).unwrap_or_default()
}

#[tauri::command]
pub fn create_workspace(name: String, state: tauri::State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    db::create_workspace(&conn, &name).map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn switch_workspace(name: String, state: tauri::State<DbState>) {
    let conn = state.0.lock().unwrap();
    db::switch_workspace(&conn, &name).ok();
}

#[tauri::command]
pub fn delete_workspace(name: String, state: tauri::State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    db::delete_workspace(&conn, &name).map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn assign_collection(
    id: String,
    collection: Option<String>,
    state: tauri::State<DbState>,
) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    db::assign_collection(&conn, &id, collection.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_collections(state: tauri::State<DbState>) -> Vec<db::CollectionSummary> {
    let conn = state.0.lock().unwrap();
    db::list_collections(&conn).unwrap_or_default()
}

#[tauri::command]
pub fn add_tag(id: String, tag: String, state: tauri::State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    db::add_tag(&conn, &id, &tag).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_tag(id: String, tag: String, state: tauri::State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    db::remove_tag(&conn, &id, &tag).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_rules(state: tauri::State<DbState>) -> Vec<automation::Rule> {
    let conn = state.0.lock().unwrap();
    automation::list_rules(&conn).unwrap_or_default()
}

#[tauri::command]
pub fn create_rule(
    trigger_kind: automation::TriggerKind,
    trigger_value: String,
    action_kind: automation::ActionKind,
    action_value: String,
    state: tauri::State<DbState>,
) -> Result<automation::Rule, String> {
    let conn = state.0.lock().unwrap();
    automation::create_rule(
        &conn,
        trigger_kind,
        &trigger_value,
        action_kind,
        &action_value,
    )
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn update_rule(
    id: String,
    trigger_kind: automation::TriggerKind,
    trigger_value: String,
    action_kind: automation::ActionKind,
    action_value: String,
    state: tauri::State<DbState>,
) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    automation::update_rule(
        &conn,
        &id,
        trigger_kind,
        &trigger_value,
        action_kind,
        &action_value,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_rule_enabled(
    id: String,
    enabled: bool,
    state: tauri::State<DbState>,
) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    automation::set_enabled(&conn, &id, enabled).map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn delete_rule(id: String, state: tauri::State<DbState>) {
    let conn = state.0.lock().unwrap();
    automation::delete_rule(&conn, &id).ok();
}

#[tauri::command]
pub fn copy_to_clipboard(text: String, app: AppHandle) {
    app.clipboard().write_text(text).ok();
    // Marks this write as "ours" so the capture monitor doesn't re-add it.
    clipboard::mark_self_write();
}

#[tauri::command]
pub fn check_accessibility_trusted() -> bool {
    ax::is_trusted()
}

// Triggers the macOS system dialog; the frontend re-polls afterward since
// the user's choice happens in System Settings, not synchronously.
#[tauri::command]
pub fn request_accessibility_trust() -> bool {
    ax::request_trust()
}

#[tauri::command]
pub fn mark_onboarded(app: AppHandle) {
    onboarding::mark_onboarded(&app);
}

// The OS's login-item registration is the source of truth — it can also be
// changed outside the app (System Settings > Login Items).
#[tauri::command]
pub fn is_autostart_enabled(app: AppHandle) -> bool {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch().is_enabled().unwrap_or(false)
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    let manager = app.autolaunch();
    if enabled {
        manager.enable()
    } else {
        manager.disable()
    }
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn hide_palette(window: tauri::Window) {
    window.hide().ok();
}

// Holds the `Update` handle between check_for_update and install_update —
// the updater's signature-verified download needs the same handle that
// found it, so we can't just re-check inside install_update.
#[derive(Default)]
pub struct UpdateState(Mutex<Option<tauri_plugin_updater::Update>>);

#[derive(serde::Serialize)]
pub struct UpdateInfo {
    version: String,
    notes: String,
}

#[tauri::command]
pub async fn check_for_update(
    app: AppHandle,
    state: tauri::State<'_, UpdateState>,
) -> Result<Option<UpdateInfo>, String> {
    let update = app
        .updater()
        .map_err(|e| e.to_string())?
        .check()
        .await
        .map_err(|e| e.to_string())?;
    let info = update.as_ref().map(|u| UpdateInfo {
        version: u.version.clone(),
        notes: u.body.clone().unwrap_or_default(),
    });
    *state.0.lock().unwrap() = update;
    Ok(info)
}

#[tauri::command]
pub async fn install_update(
    app: AppHandle,
    state: tauri::State<'_, UpdateState>,
) -> Result<(), String> {
    let update = state
        .0
        .lock()
        .unwrap()
        .take()
        .ok_or("No update available to install")?;
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| e.to_string())?;
    app.restart();
}
