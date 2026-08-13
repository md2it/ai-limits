mod commands;
mod launch;
mod notifications;
mod platform;
#[cfg(target_os = "macos")]
mod popover_panel;
mod windows;

use std::collections::HashSet;
use std::process::ExitCode;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use tauri::Manager;

use crate::windows::MAIN_WINDOW_LABEL;

/// Menu item id opening (showing + focusing) the Main Window.
#[cfg(target_os = "macos")]
const MENU_ID_OPEN_MAIN_WINDOW: &str = "open-main-window";

/// Menu item id opening the Main Window on its Settings panel.
const MENU_ID_OPEN_SETTINGS: &str = "open-settings";

/// Help sub-pages exposed in the macOS native Help menu. Kept in sync with the
/// `HELP_CHAPTERS` list in the frontend (frontend/modules/help-chapters.js) so
/// the native menu mirrors the in-app Help sidebar.
#[cfg(target_os = "macos")]
const HELP_CHAPTERS: &[(&str, &str)] = &[
    ("about", "About"),
    ("providers", "Providers"),
    ("source", "Source"),
    ("data-errors", "Data availability"),
    ("notifications", "Notifications"),
    ("updates", "Updates"),
    ("permissions", "Permissions"),
    ("cli-mode", "CLI mode"),
    ("limitations", "Limitations"),
    ("for-developers", "For developers"),
];

fn main() {
    if std::env::args().nth(1).as_deref() == Some("--cli") {
        if ai_limits::cli::run_with_args(std::env::args().skip(2)) == ExitCode::FAILURE {
            std::process::exit(1);
        }
        return;
    }

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(Arc::new(Mutex::new(HashSet::<String>::new())))
        .manage(commands::new_structured_info_cache())
        .manage(commands::CollectionCoordinator::new())
        .setup(|app| {
            let store_path = notifications::previous_remaining_store_path(app.handle())?;
            let remaining_store: Arc<dyn ai_limits::notifications::PreviousRemainingStore> =
                Arc::new(ai_limits::notifications::FileRemainingStore::new(
                    store_path,
                ));
            app.manage(remaining_store);
            #[cfg(target_os = "macos")]
            {
                install_help_menu(app)?;
                install_main_window_close_guard(app);
                popover_panel::install(app.handle());
                popover_panel::finish_install(app.handle());
                popover_panel::install_main_window_fullscreen_observer(app);
                install_tray_icon(app)?;
            }
            launch::show_initial_window(app.handle());
            Ok(())
        })
        .on_menu_event(|app, event| {
            handle_menu_event(app, event.id().as_ref());
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_version,
            commands::get_single_provider_limits,
            commands::get_cached_provider_limits,
            commands::open_external_url,
            commands::start_provider_cli_login,
            commands::get_cli_command,
            commands::run_cli_in_terminal,
            commands::open_main_window,
            commands::open_main_window_settings,
            commands::open_main_window_help,
            commands::hide_popover,
            commands::set_popover_height,
            commands::app_update::download_app_update,
            commands::app_update::restart_app,
        ])
        .build(tauri::generate_context!())
        .expect("failed to build Tauri application");

    app.run(|_app_handle, _event| {
        // Dock icon "reopen" gesture (clicking the Dock icon, or the standard
        // app-reactivation event, when the app has no visible windows or is
        // simply not frontmost) — opens/activates the Main Window only, never
        // the Popover. See docs/desktop/mac-popover.md#entry-points.
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen { .. } = _event {
            if let Some(main) = _app_handle.get_webview_window(MAIN_WINDOW_LABEL) {
                let _ = main.show();
                let _ = main.set_focus();
            }
        }
    });
}

/// Builds the macOS menu bar. Starts from the app/Edit/Window/Help submenus
/// Tauri would generate by default, but drops File and View (unused by this
/// app) and replaces the app-menu Services item with a Settings item that
/// opens the in-app settings panel.
#[cfg(target_os = "macos")]
fn install_help_menu(app: &tauri::App) -> tauri::Result<()> {
    use tauri::menu::{
        Menu, MenuItem, PredefinedMenuItem, Submenu, HELP_SUBMENU_ID, WINDOW_SUBMENU_ID,
    };

    let handle = app.handle();
    let pkg_info = handle.package_info();
    let about_metadata = tauri::menu::AboutMetadata {
        name: Some(pkg_info.name.clone()),
        version: Some(pkg_info.version.to_string()),
        ..Default::default()
    };

    let settings_item = MenuItem::with_id(
        handle,
        MENU_ID_OPEN_SETTINGS,
        "Settings…",
        true,
        Some("Cmd+,"),
    )?;

    let app_menu = Submenu::with_items(
        handle,
        pkg_info.name.clone(),
        true,
        &[
            &PredefinedMenuItem::about(handle, None, Some(about_metadata))?,
            &PredefinedMenuItem::separator(handle)?,
            &settings_item,
            &PredefinedMenuItem::separator(handle)?,
            &PredefinedMenuItem::hide(handle, None)?,
            &PredefinedMenuItem::hide_others(handle, None)?,
            &PredefinedMenuItem::separator(handle)?,
            &PredefinedMenuItem::quit(handle, None)?,
        ],
    )?;

    let edit_menu = Submenu::with_items(
        handle,
        "Edit",
        true,
        &[
            &PredefinedMenuItem::undo(handle, None)?,
            &PredefinedMenuItem::redo(handle, None)?,
            &PredefinedMenuItem::separator(handle)?,
            &PredefinedMenuItem::cut(handle, None)?,
            &PredefinedMenuItem::copy(handle, None)?,
            &PredefinedMenuItem::paste(handle, None)?,
            &PredefinedMenuItem::select_all(handle, None)?,
        ],
    )?;

    let window_menu = Submenu::with_id_and_items(
        handle,
        WINDOW_SUBMENU_ID,
        "Window",
        true,
        &[
            &PredefinedMenuItem::minimize(handle, None)?,
            &PredefinedMenuItem::maximize(handle, None)?,
            &PredefinedMenuItem::separator(handle)?,
            &PredefinedMenuItem::close_window(handle, None)?,
        ],
    )?;

    let help_menu = Submenu::with_id_and_items(handle, HELP_SUBMENU_ID, "Help", true, &[])?;
    for (id, label) in HELP_CHAPTERS {
        let item = MenuItem::with_id(handle, format!("help:{id}"), label, true, None::<&str>)?;
        help_menu.append(&item)?;
    }

    let menu = Menu::with_items(handle, &[&app_menu, &edit_menu, &window_menu, &help_menu])?;

    app.set_menu(menu)?;
    Ok(())
}

/// Intercepts the Main Window's own close request (red traffic light /
/// Cmd+W) and hides it instead of letting it destroy the window, so the
/// application keeps running with the tray icon and Popover available after
/// the Main Window is "closed" — see docs/desktop/mac-popover.md#closing.
/// When the window is in native fullscreen, it must leave fullscreen first;
/// hiding it in place would strand its macOS Space and leave that Space bound
/// to AI Limits.
///
/// `set_fullscreen(false)` only *requests* the exit; the animated transition
/// back to a normal window runs asynchronously on AppKit's main thread and
/// isn't finished by the time this handler returns. Calling `hide()`
/// immediately after used to race that animation: AppKit would cancel the
/// in-flight fullscreen exit's completion and leave the window merely
/// un-fullscreened (and visible) instead of hidden. So the fullscreen case
/// defers `hide()` to `NSWindowDidExitFullScreenNotification`, which fires
/// once the transition genuinely completes; the already-windowed case still
/// hides synchronously since there's no transition to race.
///
/// Quit (Cmd+Q, or the native Quit item installed in `install_help_menu` via
/// `PredefinedMenuItem::quit`) is unaffected by this: it terminates the app
/// directly through the OS's native menu action rather than going through a
/// window's close-request path, so it bypasses this guard entirely.
#[cfg(target_os = "macos")]
fn install_main_window_close_guard(app: &tauri::App) {
    let Some(main_window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        return;
    };

    install_main_window_hide_after_fullscreen_exit_observer(&main_window);

    let window_to_hide = main_window.clone();
    main_window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            if window_to_hide.is_fullscreen().unwrap_or(false) {
                HIDE_MAIN_WINDOW_AFTER_FULLSCREEN_EXIT.store(true, Ordering::SeqCst);
                let _ = window_to_hide.set_fullscreen(false);
            } else {
                let _ = window_to_hide.hide();
            }
        }
    });
}

/// Set by `install_main_window_close_guard` when Cmd+W arrives while the
/// Main Window is fullscreen, and consumed by the fullscreen-exit observer
/// below. Left `false` the rest of the time, so the Main Window exiting
/// fullscreen for any other reason (e.g. the user manually leaving
/// fullscreen) never hides it.
#[cfg(target_os = "macos")]
static HIDE_MAIN_WINDOW_AFTER_FULLSCREEN_EXIT: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Hides the Main Window once its native-fullscreen exit animation actually
/// completes, if that exit was requested by the close guard above. The
/// observer token is leaked intentionally: it must live for the process's
/// lifetime, exactly like the analogous popover observer in
/// `popover_panel::install_main_window_fullscreen_observer`.
#[cfg(target_os = "macos")]
fn install_main_window_hide_after_fullscreen_exit_observer(main_window: &tauri::WebviewWindow) {
    use objc2_app_kit::{NSWindow, NSWindowDidExitFullScreenNotification};
    use objc2_foundation::{NSNotification, NSNotificationCenter};
    use std::ptr::NonNull;

    let Ok(main_window_ptr) = main_window.ns_window() else {
        return;
    };
    // SAFETY: Tauri returned the live NSWindow pointer for the application's
    // always-alive Main Window. The observer is installed during setup and
    // is removed only when the process exits, after AppKit has stopped
    // sending window notifications.
    let ns_window = unsafe { &*main_window_ptr.cast::<NSWindow>() };
    let window_to_hide = main_window.clone();
    let block = block2::RcBlock::new(move |_notification: NonNull<NSNotification>| {
        if HIDE_MAIN_WINDOW_AFTER_FULLSCREEN_EXIT.swap(false, Ordering::SeqCst) {
            let _ = window_to_hide.hide();
        }
    });
    // SAFETY: the observer is scoped to the live Main Window; `queue: None`
    // invokes the block synchronously on AppKit's main thread.
    let observer = unsafe {
        NSNotificationCenter::defaultCenter().addObserverForName_object_queue_usingBlock(
            Some(NSWindowDidExitFullScreenNotification),
            Some(ns_window.as_ref()),
            None,
            &block,
        )
    };
    std::mem::forget(observer);
}

/// Creates the macOS menu bar tray icon. Left-clicking it toggles the
/// Popover window — shows and positions it near the icon if hidden, hides it
/// if already visible — and never touches the Main Window, per
/// docs/desktop/mac-popover.md#entry-points. Right-clicking opens the small
/// context menu built by `build_tray_menu` instead, the interaction any
/// menu bar app is expected to offer.
#[cfg(target_os = "macos")]
fn install_tray_icon(app: &tauri::App) -> tauri::Result<()> {
    use tauri::image::Image;
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

    // Purpose-built macOS menu bar template image: monochrome, alpha-only,
    // 18pt rendered at @2x (36×36 px). `icon_as_template(true)` sets
    // `NSImage.isTemplate`, which is what makes AppKit tint it for the current
    // menu bar appearance (light/dark, and inverted while the icon is
    // highlighted) instead of blitting our pixels verbatim. Generated from the
    // app's master artwork by scripts/generate-desktop-icons.sh.
    //
    // Embedded in the binary rather than read from disk at runtime: it ships
    // inside the same `.app` either way, and this removes any resource-path
    // resolution and any chance of a missing-file failure at launch.
    // 36×36 is the only size worth shipping — the tray API takes exactly one
    // image and rescales it to an 18pt height, and macOS has no @3x displays.
    let icon = Image::from_bytes(include_bytes!("../icons/icon-tray.png"))
        .expect("icons/icon-tray.png must be a valid PNG");

    TrayIconBuilder::new()
        .icon(icon)
        .icon_as_template(true)
        .tooltip("AI Limits")
        .menu(&build_tray_menu(app.handle())?)
        // Left click is the Popover toggle handled below; the menu is for the
        // right click only.
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            else {
                return;
            };

            let app = tray.app_handle();

            // Toggle: a second click while the Popover is visible hides it.
            // Routed through the shared `commands::hide_popover` (rather than
            // a bare `popover_panel::hide()`) purely for symmetry with every
            // other hide path; both end up in the same place.
            if popover_panel::is_visible() {
                let _ = commands::hide_popover(app.clone());
                return;
            }

            popover_panel::show_near_tray(app, tray.clone(), rect);
        })
        .build(app)?;

    Ok(())
}

/// Right-click menu for the tray icon. Every item routes through the same
/// ids `handle_menu_event` already dispatches for the native app menu, so
/// there is exactly one implementation of "open the Main Window" and "open
/// Settings"; Quit is the standard predefined item, identical to the app
/// menu's.
#[cfg(target_os = "macos")]
fn build_tray_menu(handle: &tauri::AppHandle) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};

    Menu::with_items(
        handle,
        &[
            &MenuItem::with_id(
                handle,
                MENU_ID_OPEN_MAIN_WINDOW,
                "Open AI Limits",
                true,
                None::<&str>,
            )?,
            &MenuItem::with_id(
                handle,
                MENU_ID_OPEN_SETTINGS,
                "Settings…",
                true,
                None::<&str>,
            )?,
            &PredefinedMenuItem::separator(handle)?,
            &PredefinedMenuItem::quit(handle, None)?,
        ],
    )
}

/// Routes native menu selections — from the app menu and from the tray icon's
/// right-click menu alike — to the commands that own each action. The
/// commands are the same ones the Popover's own toolbar buttons invoke, so
/// there is one implementation per action rather than one per entry point.
fn handle_menu_event(app: &tauri::AppHandle, menu_id: &str) {
    if menu_id == MENU_ID_OPEN_SETTINGS {
        let _ = commands::open_main_window_settings(app.clone());
        return;
    }

    #[cfg(target_os = "macos")]
    if menu_id == MENU_ID_OPEN_MAIN_WINDOW {
        let _ = commands::open_main_window(app.clone());
        return;
    }

    if let Some(chapter) = menu_id.strip_prefix("help:") {
        let _ = commands::open_main_window_help(app.clone(), Some(chapter.to_string()));
    }
}
