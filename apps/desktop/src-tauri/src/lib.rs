mod automation;
mod ax;
mod blocklist;
mod classifier;
mod clipboard;
mod commands;
mod context;
mod db;
mod frontmost_app;
mod keychain;
mod markdown;
mod onboarding;
mod palette;
mod pasteboard;
mod search;
mod sensitive;
mod tray;

use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use clipboard::DbState;
use palette::{show_palette, toggle_palette, PALETTE_WINDOW};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Cmd+Shift+V on macOS; Ctrl+Alt+V elsewhere. Not Ctrl+Shift+V — a
    // global hotkey would steal "paste as plain text" from Chrome, Word,
    // and Slack system-wide. Not Win+V-adjacent either: Windows reserves
    // that space for its native clipboard history.
    #[cfg(target_os = "macos")]
    let shortcut_modifiers = Modifiers::SUPER | Modifiers::SHIFT;
    #[cfg(not(target_os = "macos"))]
    let shortcut_modifiers = Modifiers::CONTROL | Modifiers::ALT;
    let shortcut = Shortcut::new(Some(shortcut_modifiers), Code::KeyV);

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, scut, event| {
                    if *scut == shortcut && event.state() == ShortcutState::Pressed {
                        toggle_palette(app);
                    }
                })
                .build(),
        );

    // NSPanel support (see palette.rs for why the palette must be a panel).
    #[cfg(target_os = "macos")]
    let builder = builder.plugin(tauri_nspanel::init());

    builder
        .invoke_handler(tauri::generate_handler![
            commands::get_history,
            commands::search_clips,
            commands::get_clip_content,
            commands::render_markdown,
            commands::delete_clips,
            commands::set_favorite,
            commands::list_workspaces,
            commands::create_workspace,
            commands::switch_workspace,
            commands::delete_workspace,
            commands::assign_collection,
            commands::list_collections,
            commands::add_tag,
            commands::remove_tag,
            commands::list_rules,
            commands::create_rule,
            commands::update_rule,
            commands::set_rule_enabled,
            commands::delete_rule,
            commands::copy_to_clipboard,
            commands::check_accessibility_trusted,
            commands::request_accessibility_trust,
            commands::mark_onboarded,
            commands::hide_palette,
            commands::check_for_update,
            commands::install_update,
            commands::is_autostart_enabled,
            commands::set_autostart
        ])
        .on_window_event(palette::handle_window_event)
        .setup(move |app| {
            app.manage(palette::LastShown::default());
            app.manage(commands::UpdateState::default());
            blocklist::ensure_exists(app.handle());
            let conn = db::init_db(app.handle())?;
            automation::init_table(&conn)?;
            app.manage(DbState(Mutex::new(conn)));
            clipboard::start_monitor(app.handle().clone());

            // Native vibrancy instead of CSS backdrop-filter — WebKit/WebView2
            // resampling behind a transparent window every frame caused
            // visible compositing glitches even at idle.
            #[cfg(target_os = "macos")]
            {
                if let Some(window) = app.get_webview_window(PALETTE_WINDOW) {
                    window_vibrancy::apply_vibrancy(
                        &window,
                        window_vibrancy::NSVisualEffectMaterial::Popover,
                        None,
                        Some(16.0),
                    )
                    .expect("failed to apply macOS vibrancy");
                }
                // After vibrancy: swizzle the window into a non-activating
                // NSPanel so it can appear over fullscreen Spaces.
                palette::init_panel(app.handle());
            }

            // Acrylic is the broadly-compatible choice (Windows 10 1809+);
            // Mica would need a Windows-11-only code path. Not `.expect()`-ed
            // like the macOS call above — this hasn't been verified against
            // a real Windows build in this environment (no Windows machine
            // available), so a failure here degrades to the plain
            // transparent window rather than taking the app down with it.
            #[cfg(target_os = "windows")]
            if let Some(window) = app.get_webview_window(PALETTE_WINDOW) {
                let _ = window_vibrancy::apply_acrylic(&window, Some((252, 252, 254, 180)));
            }

            app.global_shortcut().register(shortcut)?;

            // Tray-only presence, no Dock icon — a floating command palette,
            // like Raycast/Maccy-style clipboard managers.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            tray::init(app)?;

            // First launch only: reveal immediately so the permission
            // banner is seen before the user knows the shortcut exists.
            if !onboarding::is_onboarded(app.handle()) {
                show_palette(app.handle());
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
