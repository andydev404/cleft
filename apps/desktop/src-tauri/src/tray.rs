use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{App, Emitter};

use crate::palette::{show_palette, toggle_palette};

pub fn init(app: &App) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    let accelerator = "Cmd+Shift+V";
    #[cfg(not(target_os = "macos"))]
    let accelerator = "Ctrl+Alt+V";
    let open_item = MenuItem::with_id(app, "open", "Open Cleft", true, Some(accelerator))?;
    // The Accessibility permission only exists on macOS — no dead
    // "Permissions…" entry on other platforms.
    #[cfg(target_os = "macos")]
    let permissions_item =
        MenuItem::with_id(app, "permissions", "Permissions…", true, None::<&str>)?;
    let update_item = MenuItem::with_id(
        app,
        "check_updates",
        "Check for Updates…",
        true,
        None::<&str>,
    )?;
    let quit_item = PredefinedMenuItem::quit(app, Some("Quit Cleft"))?;
    let separator_top = PredefinedMenuItem::separator(app)?;
    let separator_bottom = PredefinedMenuItem::separator(app)?;

    let mut items: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = vec![&open_item, &separator_top];
    #[cfg(target_os = "macos")]
    items.push(&permissions_item);
    items.extend([
        &update_item as &dyn tauri::menu::IsMenuItem<tauri::Wry>,
        &separator_bottom,
        &quit_item,
    ]);
    let menu = Menu::with_items(app, &items)?;

    // Not a template image — the icon is a full-color logo, not a
    // monochrome alpha mask, so `icon_as_template` would render a blob.
    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Cleft")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => toggle_palette(app),
            // Re-triggers the native Accessibility dialog via the
            // frontend (runwayStore.requestPermission).
            "permissions" => {
                show_palette(app);
                app.emit("replay-runway", ()).ok();
            }
            // Handled on the frontend (updateStore) so manual checks
            // share the same dialog/toast UX as the startup check.
            "check_updates" => {
                show_palette(app);
                app.emit("check-for-updates", ()).ok();
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}
