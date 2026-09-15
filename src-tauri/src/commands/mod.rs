pub mod app_update;
mod background_refresh;
mod collect;
mod provider_limits;
mod structured_cache;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use tauri::Manager;

use crate::platform::terminal;
use ai_limits::infra::os_access;
use ai_limits::infra::os_access::{
    allowed_cli_command_is_available, CLAUDE_CLI_COMMAND, CODEX_CLI_COMMAND,
};
use ai_limits::notifications::PreviousRemainingStore;
use ai_limits::types::CliAuthorization;

use crate::windows::MAIN_WINDOW_LABEL;

pub use background_refresh::BackgroundRefreshScheduler;
#[cfg(target_os = "macos")]
pub use collect::{
    PROVIDER_REFRESH_FAILED_EVENT, PROVIDER_REFRESH_STARTED_EVENT, PROVIDER_UPDATED_EVENT,
};
pub use provider_limits::{ProviderLimits, ProviderLimitsQuery};
pub use structured_cache::{new_structured_info_cache, CollectionCoordinator, StructuredInfoCache};

use collect::collect_single_provider_limits;
use provider_limits::provider_limits_from_structured;
use structured_cache::read_cached_structured_info;

#[tauri::command]
pub fn get_app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[tauri::command]
pub async fn get_single_provider_limits(
    provider_id: String,
    query: ProviderLimitsQuery,
    app: tauri::AppHandle,
    sent_notifications: tauri::State<'_, Arc<Mutex<HashSet<String>>>>,
    remaining_store: tauri::State<'_, Arc<dyn PreviousRemainingStore>>,
    structured_cache: tauri::State<'_, StructuredInfoCache>,
    coordinator: tauri::State<'_, CollectionCoordinator>,
) -> Result<ProviderLimits, String> {
    let sent_notifications = Arc::clone(sent_notifications.inner());
    let remaining_store = Arc::clone(remaining_store.inner());
    let structured_cache = Arc::clone(structured_cache.inner());
    let coordinator = coordinator.inner().clone();

    collect_single_provider_limits(
        &provider_id,
        &query,
        app,
        sent_notifications,
        remaining_store,
        structured_cache,
        coordinator,
    )
    .await
}

/// Reads the current shared structured-data snapshot for `provider_id`
/// without starting or joining a collection — `None` if no surface has
/// collected it yet this session. Lets a surface that is initializing (or
/// showing a just-enabled provider) render another surface's already-current
/// result immediately, deferring to its own scheduled/manual refresh for
/// anything newer. See docs/desktop/ui/frontend-state.md.
#[tauri::command]
pub fn get_cached_provider_limits(
    provider_id: String,
    structured_cache: tauri::State<'_, StructuredInfoCache>,
) -> Option<ProviderLimits> {
    let structured = read_cached_structured_info(structured_cache.inner(), &provider_id)?;
    Some(provider_limits_from_structured(&provider_id, &structured))
}

#[tauri::command]
pub fn configure_background_refresh(
    config: background_refresh::BackgroundRefreshConfig,
    scheduler: tauri::State<'_, BackgroundRefreshScheduler>,
) -> Result<(), String> {
    scheduler.configure(config)
}

#[tauri::command]
pub async fn open_external_url(url: String) -> Result<(), String> {
    if !os_access::is_allowed_external_url(&url) {
        return Err("External URL is not allowed".to_string());
    }

    os_access::open_external_url_with_system(&url).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn start_provider_cli_login(provider: String) -> Result<(), String> {
    let auth = CliAuthorization::parse(&provider)?;
    let cli_command = match auth {
        CliAuthorization::Codex => CODEX_CLI_COMMAND,
        CliAuthorization::Claude => CLAUDE_CLI_COMMAND,
    };

    if !allowed_cli_command_is_available(cli_command) {
        return Err(format!(
            "{cli_command} is not installed or is not available in PATH"
        ));
    }

    terminal::open_with_command(auth.login_command()).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_cli_command() -> Result<String, String> {
    terminal::cli_command_for_current_executable()
}

#[tauri::command]
pub fn run_cli_in_terminal() -> Result<(), String> {
    terminal::run_current_cli_in_terminal()
}

/// Shows and focuses the Main Window, without navigating it anywhere —
/// backs the Popover's "Open Application" button
/// (docs/desktop/mac-popover.md#entry-points). No-op if the Main Window does
/// not exist (should not happen in practice: it is the app's implicit
/// default window).
#[tauri::command]
pub fn open_main_window(app: tauri::AppHandle) -> Result<(), String> {
    show_and_focus_main_window(&app);
    Ok(())
}

/// Shows and focuses the Main Window, then asks it to open Settings — backs
/// the Popover's `[gear]` toolbar button
/// (docs/desktop/mac-popover.md#toolbar). Mirrors the "open-settings" branch
/// of `handle_menu_event` in main.rs, which does the same thing in response
/// to the native app-menu Settings item instead of a Popover click.
#[tauri::command]
pub fn open_main_window_settings(app: tauri::AppHandle) -> Result<(), String> {
    show_and_focus_main_window(&app);

    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let _ = window.eval("window.__openSettingsFromNative && window.__openSettingsFromNative()");
    }

    Ok(())
}

/// Shows and focuses the Main Window, then asks it to open Help — optionally
/// on a specific chapter — backing the Popover's `[info]` toolbar button and
/// a provider card's "More details" link when no local Help view exists
/// (docs/desktop/mac-popover.md#toolbar). Mirrors the "help:<id>" branch of
/// `handle_menu_event` in main.rs.
///
/// `chapter` is forwarded as-is: `None` evals
/// `window.__openHelpFromNative()` with no argument at all (not `null`), so
/// the frontend's own `openHelp(chapterId = DEFAULT_HELP_CHAPTER)` default
/// parameter kicks in exactly as it does for the Main Window's own Help
/// button — passing an explicit `null` would bypass that default.
#[tauri::command]
pub fn open_main_window_help(app: tauri::AppHandle, chapter: Option<String>) -> Result<(), String> {
    show_and_focus_main_window(&app);

    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let call = match chapter {
            Some(chapter) => {
                let chapter_json =
                    serde_json::to_string(&chapter).unwrap_or_else(|_| "null".to_string());
                format!(
                    "window.__openHelpFromNative && window.__openHelpFromNative({chapter_json})"
                )
            }
            None => "window.__openHelpFromNative && window.__openHelpFromNative()".to_string(),
        };
        let _ = window.eval(call);
    }

    Ok(())
}

/// Hides the Popover without destroying it — backs the Popover frontend's
/// Escape-key handler (now usually unreachable directly, since the Popover's
/// own native Escape monitor in `popover_panel` calls straight into
/// `popover_panel::hide()`; this command remains the entry point for every
/// other caller, e.g. `show_and_focus_main_window` below). No-op if the
/// Popover does not exist (any non-macOS platform).
#[tauri::command]
pub fn hide_popover(app: tauri::AppHandle) -> Result<(), String> {
    hide_popover_window(&app);
    Ok(())
}

/// Resizes the Popover to the content height the frontend measured, in
/// logical pixels, so the panel is as tall as what it shows — the way system
/// menu bar panels behave — instead of a fixed size that is too tall for one
/// provider card and too short for three failing ones.
///
/// Contract with the frontend (see docs/desktop/mac-popover.md#window-size):
/// pass the full desired *outer* height of the panel, including its own
/// padding; the width is owned by the native side and never changes. Values
/// outside `[POPOVER_MIN_HEIGHT, POPOVER_MAX_HEIGHT]` are clamped rather than
/// rejected, so an over-tall panel scrolls internally instead of running off
/// the screen. The window grows downwards from its anchor under the tray
/// icon, so a resize never needs a reposition. Calling this repeatedly is
/// cheap and idempotent; a non-finite height is ignored. On a non-macOS
/// build there is no Popover panel and this is a silent no-op.
#[tauri::command]
pub fn set_popover_height(height: f64) -> Result<(), String> {
    if !height.is_finite() {
        return Err("Popover height must be a finite number".to_string());
    }

    #[cfg(target_os = "macos")]
    crate::popover_panel::set_height(height);

    Ok(())
}

/// Shared show+focus logic for the three commands above. No-op if `"main"`
/// does not exist.
fn show_and_focus_main_window(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        return;
    };

    #[cfg(target_os = "macos")]
    crate::platform::dock::set_visible(app, true);

    let _ = window.show();
    let _ = window.set_focus();
    let _ = window.eval(
        "window.__refreshProvidersFromSharedCache && window.__refreshProvidersFromSharedCache()",
    );

    hide_popover_window(app);
}

///
/// Also tears down the native dismiss monitors installed by
/// `install_popover_dismiss_monitors` in main.rs (outside-click, Escape,
/// Space-change — see its doc comment for why they exist), if any are
/// currently installed. This makes `hide_popover_window` the single funnel
/// every hide path goes through: the tray icon's own toggle-off click, the
/// Escape/outside-click/Space-change monitors themselves (each calls the
/// public `hide_popover` command, which calls this), and this function's own
/// other callers below. Without this, hiding the Popover by any path other
/// than the monitors' own dismissal would leak them running in the
/// background against an already-hidden window.
fn hide_popover_window(app: &tauri::AppHandle) {
    let _ = app;
    #[cfg(target_os = "macos")]
    {
        crate::popover_panel::hide();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_authorization_parse_accepts_only_codex_and_claude() {
        assert_eq!(
            CliAuthorization::parse("codex").unwrap().login_command(),
            "codex login"
        );
        assert_eq!(
            CliAuthorization::parse("claude").unwrap().login_command(),
            "claude login"
        );
        assert!(CliAuthorization::parse("cursor").is_err());
        assert!(CliAuthorization::parse("").is_err());
    }
}
