//! Native `NSPanel`-hosted macOS Menu Bar Popover.
//!
//! Replaces the plain `WebviewWindow` attempt described in the "Status"
//! section of docs/desktop/mac-popover.md, which could not be made to appear
//! on another app's active Fullscreen Space in a real manual test. This
//! module builds a genuine `NSPanel` (`NonactivatingPanel` style mask — the
//! standard AppKit mechanism real menu-bar utilities use for exactly this
//! class of window) and hosts the *same* `popover.html`/`popover.js` web UI
//! inside it, via `wry`'s own public `WebViewBuilder::build_as_child` API —
//! the same WebKit embedding engine Tauri itself uses for its ordinary
//! windows, just attached to a window this module builds natively instead of
//! one Tauri's `WebviewWindowBuilder` builds.
//!
//! Two things this buys over the old `WebviewWindow` attempt, both direct
//! consequences of the window's Objective-C class actually being `NSPanel`
//! rather than a plain `NSWindow` with matching flags set on it:
//!
//! - `NonactivatingPanel` is documented by Apple to let the panel become the
//!   key window (so it can receive the very first click) *without* making
//!   the owning application active — exactly the "first click" and
//!   "Space-switch" problems the old implementation needed two separate
//!   hand-rolled workarounds for (`order_front_without_activating` and a
//!   ruled-out isa-swizzle). Here both fall out of the style mask itself.
//! - AppKit's window server treats genuine `NSPanel`/utility windows
//!   differently from an ordinary `NSWindow` with respect to Fullscreen Space
//!   compositing, which is the specific criterion the old window failed.
//!
//! ## Why `wry::WebViewBuilder` instead of hand-writing a `WKWebView`
//!
//! A raw `WKWebView` loaded via `loadFileURL` would run under a `file://`
//! origin, which is a *different* WebKit storage origin than the Main
//! Window's `tauri://localhost` — breaking the "same settings/localStorage
//! model" requirement at the storage layer, before IPC even enters into it.
//! `wry::WebViewBuilder::with_custom_protocol` registers a `WKURLSchemeHandler`
//! for the same `"tauri"` scheme Tauri's own webviews use on macOS (see
//! `tauri-2.11.5/src/webview/mod.rs`'s `is_local_url`, which only ever checks
//! `url.scheme() == "tauri"` on this platform), and this module points the
//! Popover's initial URL at `tauri://localhost/popover.html` — the same
//! `scheme://host` Tauri's Main Window uses — sharing the same default
//! `WKWebsiteDataStore` (neither webview asks for a non-persistent or
//! identified store), and therefore, expected to share `localStorage` under
//! that shared origin. This is the load-bearing assumption of the whole
//! design and is called out as unverified in docs/desktop/mac-popover.md —
//! it could not be checked against a real multi-webview macOS process in this
//! environment.
//!
//! Reusing `wry` this way is not a second, competing WebKit wrapper: it is
//! the same crate `tauri-runtime-wry` already depends on to build the Main
//! Window's own webview (see Cargo.lock), used here directly via its public
//! embedding API (`build_as_child`, documented for exactly this "host a
//! webview inside a window you built yourself" use case) instead of
//! reimplementing `WKWebViewConfiguration`/`WKUserContentController` bridging
//! by hand.
//!
//! ## The IPC bridge
//!
//! `popover.js` and the modules it pulls in (`settings.js`, `theme.js`,
//! `providers.js`, `help.js`, `links.js`) call exactly two things through
//! `window.__TAURI__`: `core.invoke(cmd, payload)` and `event.listen(name, cb)`
//! (see BOOTSTRAP_JS below). Rather than reimplementing Tauri's actual
//! internal `__TAURI_INTERNALS__` invoke/event plumbing (a larger, more
//! version-coupled surface than this window needs), `BOOTSTRAP_JS` defines a
//! small, self-contained shim exposing just that surface, backed by wry's own
//! `window.ipc.postMessage(string)` (auto-injected by
//! `WebViewBuilder::with_ipc_handler`) for the Rust-bound direction, and a
//! `window.__popoverResolve`/`window.__popoverDispatchEvent` pair this module
//! calls via `WebView::evaluate_script` for the JS-bound direction. No
//! frontend file is modified to make this work: from the page's point of
//! view `window.__TAURI__.core.invoke`/`.event.listen` behave the same as
//! under a real Tauri-built window.

use std::ffi::c_void;
use std::ptr::NonNull;
use std::sync::{Mutex, OnceLock};

use block2::RcBlock;
use objc2::rc::{Allocated, Retained};
use objc2::runtime::{AnyClass, AnyObject, Bool, ClassBuilder, Sel};
use objc2::{class, msg_send, sel, MainThreadOnly};
use objc2_app_kit::{
    NSBackingStoreType, NSColor, NSEvent, NSEventMask, NSPanel, NSScreen,
    NSVisualEffectBlendingMode, NSVisualEffectMaterial, NSVisualEffectState, NSVisualEffectView,
    NSWindow, NSWindowCollectionBehavior, NSWindowDidExitFullScreenNotification, NSWindowStyleMask,
    NSWorkspace, NSWorkspaceActiveSpaceDidChangeNotification,
};
use objc2_foundation::{
    MainThreadMarker, NSNotification, NSNotificationCenter, NSPoint, NSRect, NSSize,
};
use objc2_quartz_core::kCACornerCurveContinuous;
use serde_json::{json, Value as JsonValue};
use tauri::Listener;
use tauri::Manager;
use wry::http::Request as HttpRequest;
use wry::http::Response as HttpResponse;
use wry::raw_window_handle::{
    AppKitWindowHandle, HandleError, HasWindowHandle, RawWindowHandle, WindowHandle,
};
use wry::{Rect, WebView, WebViewBuilder, WebViewExtMacOS};

use crate::windows::{
    POPOVER_CORNER_RADIUS, POPOVER_DEFAULT_HEIGHT, POPOVER_MAX_HEIGHT, POPOVER_MENU_BAR_GAP,
    POPOVER_MIN_HEIGHT, POPOVER_SCREEN_MARGIN, POPOVER_WIDTH,
};

/// The custom URL scheme the Popover's webview is served over. Deliberately
/// the same scheme name Tauri's own windows use on macOS
/// (`tauri-2.11.5/src/webview/mod.rs`'s `is_local_url`), so the Popover's
/// `tauri://localhost` origin matches the Main Window's — see the module
/// doc comment for why that equality is load-bearing.
const POPOVER_SCHEME: &str = "tauri";

/// Minimal `window.__TAURI__` shim covering exactly the surface
/// popover.html's scripts use: `core.invoke(cmd, payload)` returning a
/// Promise, and `event.listen(name, cb)` returning an unlisten function
/// wrapped in a Promise (matching the real API's shape, which every caller
/// here already treats as async). `event.emit` is a harmless no-op stub: the
/// Popover has no settings UI of its own, so per
/// docs/desktop/mac-popover.md#cross-window-sync it is only ever a listener,
/// never an emitter, in practice.
const BOOTSTRAP_JS: &str = r#"
(function () {
  if (window.__TAURI__) {
    return;
  }
  let nextId = 1;
  const pending = new Map();
  const listeners = new Map();

  window.__TAURI__ = {
    core: {
      invoke: function (cmd, payload) {
        return new Promise(function (resolve, reject) {
          const id = nextId++;
          pending.set(id, { resolve: resolve, reject: reject });
          window.ipc.postMessage(JSON.stringify({ id: id, cmd: cmd, payload: payload || {} }));
        });
      },
    },
    event: {
      listen: function (name, cb) {
        if (!listeners.has(name)) {
          listeners.set(name, []);
        }
        listeners.get(name).push(cb);
        return Promise.resolve(function unlisten() {
          const arr = listeners.get(name);
          if (!arr) {
            return;
          }
          const i = arr.indexOf(cb);
          if (i >= 0) {
            arr.splice(i, 1);
          }
        });
      },
      emit: function () {
        return Promise.resolve();
      },
    },
  };

  window.__popoverResolve = function (id, ok, payload) {
    const p = pending.get(id);
    if (!p) {
      return;
    }
    pending.delete(id);
    if (ok) {
      p.resolve(payload);
    } else {
      p.reject(payload);
    }
  };

  window.__popoverDispatchEvent = function (name, payload) {
    const arr = listeners.get(name);
    if (!arr) {
      return;
    }
    arr.slice().forEach(function (cb) {
      cb({ event: name, payload: payload });
    });
  };
})();
"#;

/// Wraps a value that is only ever touched from the main thread so it can
/// live in a `static`. Same pattern (and the same underlying reason —
/// `Retained<_>`/`WebView` make no `Send`/`Sync` promises Rust can see, but
/// every access here is disciplined to the main thread the way AppKit and
/// WebKit both require) main.rs already uses for its own dismiss-monitor
/// statics.
struct MainThreadOnlyBox<T>(T);
// SAFETY: see the type's doc comment.
unsafe impl<T> Send for MainThreadOnlyBox<T> {}

struct Panel {
    panel: Retained<NSPanel>,
    webview: WebView,
    /// Set while the outside-click/Escape/Space-change dismiss monitors are
    /// installed (i.e. while the panel is visible). Mirrors
    /// `POPOVER_DISMISS_MONITORS` in the pre-`NSPanel` implementation.
    dismiss_monitors: Option<DismissMonitors>,
}

struct DismissMonitors {
    global_click: Retained<AnyObject>,
    local_click: Retained<AnyObject>,
    global_key: Retained<AnyObject>,
    local_key: Retained<AnyObject>,
    space_change: Retained<objc2::runtime::ProtocolObject<dyn objc2::runtime::NSObjectProtocol>>,
}

type NotificationObserver =
    Retained<objc2::runtime::ProtocolObject<dyn objc2::runtime::NSObjectProtocol>>;

static POPOVER: OnceLock<Mutex<MainThreadOnlyBox<Panel>>> = OnceLock::new();

/// The observer token lives for the application's lifetime. It watches only
/// the Main Window's fullscreen exit, the transition that otherwise leaves a
/// reused panel associated with AI Limits' former fullscreen Space.
static MAIN_WINDOW_FULLSCREEN_OBSERVER: OnceLock<Mutex<MainThreadOnlyBox<NotificationObserver>>> =
    OnceLock::new();

fn with_panel<R>(f: impl FnOnce(&mut Panel) -> R) -> Option<R> {
    let cell = POPOVER.get()?;
    let mut guard = cell.lock().unwrap();
    Some(f(&mut guard.0))
}

/// A thin `HasWindowHandle` wrapper around the panel's own content view, so
/// `wry::WebViewBuilder::build_as_child` can attach a `WKWebView` as a
/// subview of it. Only used once, at construction time.
struct ContentViewHandle(NonNull<c_void>);

impl HasWindowHandle for ContentViewHandle {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let handle = AppKitWindowHandle::new(self.0);
        // SAFETY: `self.0` is the panel's own content view, valid for as
        // long as the panel itself is (the panel is never released while
        // the app runs — see `install`), which outlives this borrow.
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::AppKit(handle)) })
    }
}

/// Builds the Popover panel and its hosted webview, and stores both in the
/// module-level static. Called once from `main.rs`'s `setup`. The panel is
/// built hidden — nothing here shows it; it only ever appears via
/// `toggle_from_tray`, matching the Launch Behavior spec in
/// docs/desktop/mac-popover.md (no surface opens automatically on any launch
/// path).
pub fn install(app: &tauri::AppHandle) {
    let mtm = MainThreadMarker::new().expect("popover_panel::install must run on the main thread");

    let initial_size = NSSize::new(POPOVER_WIDTH, POPOVER_DEFAULT_HEIGHT);
    let panel = create_panel(mtm, initial_size);
    let webview = create_webview(app, &panel, initial_size);

    POPOVER
        .set(Mutex::new(MainThreadOnlyBox(Panel {
            panel,
            webview,
            dismiss_monitors: None,
        })))
        .unwrap_or_else(|_| panic!("popover_panel::install called more than once"));
}

/// Builds the native `NSPanel` itself: borderless, non-activating, vibrancy
/// material behind the (fully transparent, see `create_webview`) webview,
/// floating above ordinary windows and — via `set_collection_behavior`/
/// `set_level` below — above another app's Fullscreen Space too.
fn create_panel(mtm: MainThreadMarker, size: NSSize) -> Retained<NSPanel> {
    let rect = NSRect::new(NSPoint::new(0.0, 0.0), size);
    let style = NSWindowStyleMask::Borderless | NSWindowStyleMask::NonactivatingPanel;

    // A borderless NSPanel cannot become key unless its subclass opts in.
    // The non-activating style still keeps that key transition from
    // activating AI Limits, allowing Escape to reach the panel immediately.
    let allocated: Allocated<NSPanel> = unsafe { msg_send![popover_panel_class(), alloc] };
    let panel = NSPanel::initWithContentRect_styleMask_backing_defer(
        allocated,
        rect,
        style,
        NSBackingStoreType::Buffered,
        false,
    );

    // SAFETY: this panel is never created inside a window controller and is
    // kept alive for the process lifetime by `POPOVER`, so disabling
    // release-on-close (which this panel never receives anyway, since it's
    // only ever hidden via `orderOut:`) just documents that intent.
    unsafe { panel.setReleasedWhenClosed(false) };

    panel.setOpaque(false);
    panel.setBackgroundColor(Some(&NSColor::clearColor()));
    panel.setHasShadow(true);
    panel.setFloatingPanel(true);
    // A menu-bar panel is meant to work without becoming the app's main
    // window's replacement; `becomesKeyOnlyIfNeeded` left at its default
    // (false) so the panel *can* become key on click — this is what fixes
    // the old implementation's "first click is swallowed" trade-off, see the
    // module doc comment.
    panel.setHidesOnDeactivate(false);

    set_collection_behavior(&panel);
    set_level(&panel);

    let content_view = build_vibrancy_view(mtm, size);
    panel.setContentView(Some(&content_view));

    panel
}

/// Registers the one-purpose NSPanel subclass needed for a borderless panel
/// to accept keyboard focus. This is a supported subclass override, not an
/// isa swap or a mutation of AppKit's existing classes.
fn popover_panel_class() -> &'static AnyClass {
    static CLASS: OnceLock<&'static AnyClass> = OnceLock::new();

    CLASS.get_or_init(|| {
        unsafe extern "C-unwind" fn can_become_key_window(_panel: &NSPanel, _cmd: Sel) -> Bool {
            Bool::YES
        }

        let mut builder = ClassBuilder::new(c"AILimitsPopoverPanel", class!(NSPanel))
            .expect("failed to register the Popover NSPanel subclass");
        // SAFETY: `canBecomeKeyWindow` returns Objective-C BOOL and matches
        // the inherited selector's signature exactly.
        unsafe {
            builder.add_method(
                sel!(canBecomeKeyWindow),
                can_become_key_window as unsafe extern "C-unwind" fn(_, _) -> _,
            );
        }
        builder.register()
    })
}

/// The panel's `contentView`: an `NSVisualEffectView` using the same
/// `.menu` material the system's own menu-bar status-item dropdowns use
/// (Wi-Fi, Bluetooth, Control Center) — rounded to `POPOVER_CORNER_RADIUS`
/// (kept equal to the CSS radius the panel content uses — see
/// docs/desktop/mac-popover.md#visual-layer).
///
/// `.popover` (the previous material here) is Apple's material for a true
/// `NSPopover` with an arrow/tip pointing back at what opened it; its
/// background tint reads visibly different from the menu-bar status-item
/// dropdowns this panel is actually trying to match, since this panel has no
/// arrow and is anchored under a menu-bar icon like those dropdowns are, not
/// under an on-screen control the way a real `NSPopover` is. `.menu` is the
/// material those dropdowns use, and matches their color/opacity instead.
fn build_vibrancy_view(mtm: MainThreadMarker, size: NSSize) -> Retained<NSVisualEffectView> {
    let rect = NSRect::new(NSPoint::new(0.0, 0.0), size);
    let view = NSVisualEffectView::initWithFrame(NSVisualEffectView::alloc(mtm), rect);

    view.setMaterial(NSVisualEffectMaterial::Menu);
    view.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
    view.setState(NSVisualEffectState::Active);
    view.setWantsLayer(true);
    if let Some(layer) = view.layer() {
        layer.setCornerRadius(POPOVER_CORNER_RADIUS);
        layer.setMasksToBounds(true);
        // System menu-bar panels round with the "squircle" continuous
        // curve, not a plain circular arc; at this same radius a circular
        // corner reads visibly tighter/sharper than the system's, which is
        // what made this panel's radius look off next to Control Center's.
        // SAFETY: reading the extern `NSString` constant is safe; only its
        // `extern` declaration requires the block.
        layer.setCornerCurve(unsafe { kCACornerCurveContinuous });
    }
    view.setAutoresizingMask(
        objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable
            | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable,
    );

    view
}

/// Builds the `wry`-hosted webview, attached as a child of the panel's
/// content view, pointed at `tauri://localhost/popover.html` — see the
/// module doc comment for why that exact scheme+host matters. Registers the
/// custom protocol (serving the bundled frontend via
/// `AppHandle::asset_resolver()`) and the IPC handler (`handle_ipc`) that
/// backs the `window.__TAURI__` shim in `BOOTSTRAP_JS`.
fn create_webview(app: &tauri::AppHandle, panel: &NSPanel, size: NSSize) -> WebView {
    let content_view = panel
        .contentView()
        .expect("panel was just given a contentView in create_panel");
    // SAFETY: `contentView` returns the panel's own live NSView, kept alive
    // by the panel itself (owned by `POPOVER` for the process lifetime).
    let view_ptr = Retained::as_ptr(&content_view) as *mut c_void;
    let handle =
        ContentViewHandle(NonNull::new(view_ptr).expect("contentView pointer is never null"));

    let protocol_app = app.clone();
    let ipc_app = app.clone();

    let webview = WebViewBuilder::new()
        .with_url(format!("{POPOVER_SCHEME}://localhost/popover.html"))
        .with_transparent(true)
        .with_initialization_script(BOOTSTRAP_JS)
        .with_custom_protocol(POPOVER_SCHEME.to_string(), move |_id, request| {
            serve_asset(&protocol_app, request)
        })
        .with_ipc_handler(move |request| handle_ipc(&ipc_app, request.body()))
        .with_bounds(Rect {
            position: wry::dpi::LogicalPosition::new(0.0, 0.0).into(),
            size: wry::dpi::LogicalSize::new(size.width, size.height).into(),
        })
        .build_as_child(&handle)
        .expect("failed to build the Popover's webview");

    // `WKWebView` composites its GPU-accelerated content through its own
    // remote layer, which does not automatically inherit `masksToBounds`
    // from the content view it was added as a subview of. Left unmasked,
    // the webview's own square corners show through at the very corners
    // the vibrancy view's rounding is supposed to clip — the "sharp corners
    // peeking out from under the rounded rect" this panel used to show. The
    // fix is to round the webview's own layer to the same radius/curve.
    let wk_view = webview.webview();
    wk_view.setWantsLayer(true);
    if let Some(layer) = wk_view.layer() {
        layer.setCornerRadius(POPOVER_CORNER_RADIUS);
        layer.setMasksToBounds(true);
        // SAFETY: reading the extern `NSString` constant is safe; only its
        // `extern` declaration requires the block.
        layer.setCornerCurve(unsafe { kCACornerCurveContinuous });
    }

    webview
}

/// Serves a bundled frontend asset for the Popover's custom-protocol
/// requests, via the same `AssetResolver` the Main Window's own
/// `tauri://localhost` requests are served from (see
/// `tauri::app::AssetResolver::get` / `get_for_scheme`) — this module does
/// not reimplement asset lookup, just wires the existing public API into
/// `wry`'s custom-protocol hook.
fn serve_asset(
    app: &tauri::AppHandle,
    request: HttpRequest<Vec<u8>>,
) -> HttpResponse<std::borrow::Cow<'static, [u8]>> {
    let raw_path = request.uri().path();
    let path = raw_path.trim_start_matches('/');
    let path = if path.is_empty() {
        "popover.html"
    } else {
        path
    };

    match app.asset_resolver().get(path.to_string()) {
        Some(asset) => HttpResponse::builder()
            .status(200)
            .header("Content-Type", asset.mime_type)
            .header(
                "Access-Control-Allow-Origin",
                format!("{POPOVER_SCHEME}://localhost"),
            )
            .body(std::borrow::Cow::Owned(asset.bytes))
            .unwrap(),
        None => HttpResponse::builder()
            .status(404)
            .body(std::borrow::Cow::Borrowed(&[][..]))
            .unwrap(),
    }
}

/// A parsed `{ id, cmd, payload }` message posted by `BOOTSTRAP_JS`'s
/// `core.invoke`.
#[derive(serde::Deserialize)]
struct IpcRequest {
    id: u64,
    cmd: String,
    #[serde(default)]
    payload: JsonValue,
}

/// Handles one `window.ipc.postMessage(...)` call from `BOOTSTRAP_JS`,
/// dispatching to the same underlying command implementations the Main
/// Window's real Tauri `invoke` calls reach — see `commands/mod.rs`. Direct
/// (synchronous) commands resolve immediately; `get_single_provider_limits`
/// is spawned onto the async runtime and resolves the JS Promise from its
/// completion callback instead, mirroring the `#[tauri::command] pub async
/// fn` it wraps.
fn handle_ipc(app: &tauri::AppHandle, body: &str) {
    let request: IpcRequest = match serde_json::from_str(body) {
        Ok(request) => request,
        Err(error) => {
            eprintln!("Popover: could not parse IPC message ({error}): {body}");
            return;
        }
    };

    match request.cmd.as_str() {
        "open_main_window" => {
            let _ = crate::commands::open_main_window(app.clone());
            resolve(request.id, Ok(JsonValue::Null));
        }
        "open_main_window_settings" => {
            let _ = crate::commands::open_main_window_settings(app.clone());
            resolve(request.id, Ok(JsonValue::Null));
        }
        "open_main_window_help" => {
            let chapter = request
                .payload
                .get("chapter")
                .and_then(JsonValue::as_str)
                .map(str::to_string);
            let _ = crate::commands::open_main_window_help(app.clone(), chapter);
            resolve(request.id, Ok(JsonValue::Null));
        }
        "hide_popover" => {
            hide();
            resolve(request.id, Ok(JsonValue::Null));
        }
        "set_popover_height" => {
            let height = request.payload.get("height").and_then(JsonValue::as_f64);
            match height {
                Some(height) => {
                    set_height(height);
                    resolve(request.id, Ok(JsonValue::Null));
                }
                None => resolve(
                    request.id,
                    Err(json!("Popover height must be a finite number")),
                ),
            }
        }
        "open_external_url" => {
            let url = request
                .payload
                .get("url")
                .and_then(JsonValue::as_str)
                .unwrap_or_default()
                .to_string();
            let id = request.id;
            tauri::async_runtime::spawn(async move {
                let result = crate::commands::open_external_url(url).await;
                resolve_on_main_thread(id, result.map(|_| JsonValue::Null).map_err(|e| json!(e)));
            });
        }
        "start_provider_cli_login" => {
            let provider = request
                .payload
                .get("provider")
                .and_then(JsonValue::as_str)
                .unwrap_or_default()
                .to_string();
            let id = request.id;
            tauri::async_runtime::spawn(async move {
                let result = crate::commands::start_provider_cli_login(provider).await;
                resolve_on_main_thread(id, result.map(|_| JsonValue::Null).map_err(|e| json!(e)));
            });
        }
        "get_single_provider_limits" => {
            let provider_id = request
                .payload
                .get("providerId")
                .and_then(JsonValue::as_str)
                .unwrap_or_default()
                .to_string();
            let query: Result<crate::commands::ProviderLimitsQuery, _> =
                serde_json::from_value(request.payload.get("query").cloned().unwrap_or_default());
            let id = request.id;

            let query = match query {
                Ok(query) => query,
                Err(error) => {
                    resolve(id, Err(json!(error.to_string())));
                    return;
                }
            };

            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                let sent_notifications = app
                    .state::<std::sync::Arc<std::sync::Mutex<std::collections::HashSet<String>>>>();
                let remaining_store = app
                    .state::<std::sync::Arc<dyn ai_limits::notifications::PreviousRemainingStore>>(
                    );
                let structured_cache = app.state::<crate::commands::StructuredInfoCache>();
                let coordinator = app.state::<crate::commands::CollectionCoordinator>();
                let result = crate::commands::get_single_provider_limits(
                    provider_id,
                    query,
                    app.clone(),
                    sent_notifications,
                    remaining_store,
                    structured_cache,
                    coordinator,
                )
                .await;
                let payload = match result {
                    Ok(limits) => serde_json::to_value(limits).map_err(|e| json!(e.to_string())),
                    Err(error) => Err(json!(error)),
                };
                resolve_on_main_thread(id, payload);
            });
        }
        "get_cached_provider_limits" => {
            let provider_id = request
                .payload
                .get("providerId")
                .and_then(JsonValue::as_str)
                .unwrap_or_default()
                .to_string();
            let structured_cache = app.state::<crate::commands::StructuredInfoCache>();
            let payload =
                crate::commands::get_cached_provider_limits(provider_id, structured_cache);
            resolve(
                request.id,
                serde_json::to_value(payload).map_err(|error| json!(error.to_string())),
            );
        }
        "configure_background_refresh" => {
            let config =
                serde_json::from_value(request.payload.get("config").cloned().unwrap_or_default());
            match config {
                Ok(config) => {
                    let scheduler = app.state::<crate::commands::BackgroundRefreshScheduler>();
                    resolve(
                        request.id,
                        scheduler
                            .configure(config)
                            .map(|_| JsonValue::Null)
                            .map_err(|error| json!(error)),
                    );
                }
                Err(error) => resolve(request.id, Err(json!(error.to_string()))),
            }
        }
        other => {
            eprintln!("Popover: unknown IPC command {other:?}");
            resolve(request.id, Err(json!(format!("unknown command {other}"))));
        }
    }
}

/// Resolves (or rejects) the JS Promise `window.__TAURI__.core.invoke`
/// returned for `id`, by evaluating a call into `window.__popoverResolve`.
/// Must run on the main thread (the panel's webview is main-thread-only, per
/// every AppKit/WebKit type this module touches).
fn resolve(id: u64, result: Result<JsonValue, JsonValue>) {
    let (ok, payload) = match result {
        Ok(value) => (true, value),
        Err(value) => (false, value),
    };
    let script = format!("window.__popoverResolve({id}, {ok}, {payload})");
    with_panel(|panel| {
        if let Err(error) = panel.webview.evaluate_script(&script) {
            eprintln!("Popover: failed to resolve IPC call {id}: {error}");
        }
    });
}

/// Same as `resolve`, but hops onto the main thread first — for use from
/// inside a `tauri::async_runtime::spawn`'d task, which may run on a tokio
/// worker thread rather than the main thread.
fn resolve_on_main_thread(id: u64, result: Result<JsonValue, JsonValue>) {
    let Some(app) = APP_HANDLE.get() else {
        return;
    };
    let _ = app.run_on_main_thread(move || resolve(id, result));
}

/// Stashed by `install` so `resolve_on_main_thread` (called from spawned
/// async tasks that don't otherwise have an `AppHandle` on hand) can reach
/// `AppHandle::run_on_main_thread`. `AppHandle` is `Send`/`Sync` (unlike the
/// panel/webview themselves), so this needs no `MainThreadOnlyBox` wrapper.
static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();

/// Forwards a Tauri app event to the Popover's already-loaded page, via
/// `window.__popoverDispatchEvent`. Used for `SETTINGS_CHANGED_EVENT`/
/// `THEME_CHANGED_EVENT`, the same two events the old `WebviewWindow`
/// implementation relied on Tauri's own cross-window event delivery for —
/// since the Popover is no longer a Tauri-managed window at all, this module
/// has to do that forwarding itself. See `install_event_forwarding`.
fn dispatch_event(name: &str, payload_json: &str) {
    let name_json = json!(name);
    let script = format!("window.__popoverDispatchEvent({name_json}, {payload_json})");
    with_panel(|panel| {
        if let Err(error) = panel.webview.evaluate_script(&script) {
            eprintln!("Popover: failed to dispatch event {name}: {error}");
        }
    });
}

/// Subscribes to the Tauri app events the Popover's frontend listens for
/// (`settings-changed`, `theme-changed`, `provider-updated`,
/// `provider-refresh-started`, `provider-refresh-failed`), forwarding each
/// to the page via `dispatch_event`. Called once from `install`, after
/// `APP_HANDLE` is stashed.
fn install_event_forwarding(app: &tauri::AppHandle) {
    let settings_app = app.clone();
    app.listen("settings-changed", move |event| {
        let payload = event.payload().to_string();
        let _ =
            settings_app.run_on_main_thread(move || dispatch_event("settings-changed", &payload));
    });

    let theme_app = app.clone();
    app.listen("theme-changed", move |event| {
        let payload = event.payload().to_string();
        let _ = theme_app.run_on_main_thread(move || dispatch_event("theme-changed", &payload));
    });

    let provider_updated_app = app.clone();
    app.listen(crate::commands::PROVIDER_UPDATED_EVENT, move |event| {
        let payload = event.payload().to_string();
        let _ = provider_updated_app.run_on_main_thread(move || {
            dispatch_event(crate::commands::PROVIDER_UPDATED_EVENT, &payload)
        });
    });

    let provider_refresh_started_app = app.clone();
    app.listen(
        crate::commands::PROVIDER_REFRESH_STARTED_EVENT,
        move |event| {
            let payload = event.payload().to_string();
            let _ = provider_refresh_started_app.run_on_main_thread(move || {
                dispatch_event(crate::commands::PROVIDER_REFRESH_STARTED_EVENT, &payload)
            });
        },
    );

    let provider_refresh_failed_app = app.clone();
    app.listen(
        crate::commands::PROVIDER_REFRESH_FAILED_EVENT,
        move |event| {
            let payload = event.payload().to_string();
            let _ = provider_refresh_failed_app.run_on_main_thread(move || {
                dispatch_event(crate::commands::PROVIDER_REFRESH_FAILED_EVENT, &payload)
            });
        },
    );
}

/// Finishes wiring the Popover up: stashes the `AppHandle` `resolve_on_main_thread`
/// needs and installs the settings/theme event forwarding. Split out from
/// `install` only so `main.rs` can call it once the panel/webview already
/// exist; call once, right after `install`.
pub fn finish_install(app: &tauri::AppHandle) {
    APP_HANDLE
        .set(app.clone())
        .unwrap_or_else(|_| panic!("popover_panel::finish_install called more than once"));
    install_event_forwarding(app);
}

/// Resets the panel's Space membership after the Main Window leaves native
/// fullscreen. AppKit can retain the panel as an auxiliary of that fullscreen
/// Space when it was shown there; simply setting the same behavior again on
/// a later show does not detach that association.
pub fn install_main_window_fullscreen_observer(app: &tauri::App) {
    let Some(main_window) = app.get_webview_window(crate::windows::MAIN_WINDOW_LABEL) else {
        return;
    };
    let Ok(main_window_ptr) = main_window.ns_window() else {
        return;
    };
    // SAFETY: Tauri returned the live NSWindow pointer for the application's
    // always-alive Main Window. The observer is installed during setup and is
    // removed only when the process exits, after AppKit has stopped sending
    // window notifications.
    let main_window = unsafe { &*main_window_ptr.cast::<NSWindow>() };
    let block = RcBlock::new(move |_notification: NonNull<NSNotification>| {
        reset_space_membership_after_main_fullscreen();
    });
    // SAFETY: the observer is scoped to the live Main Window; `queue: None`
    // invokes the block synchronously on AppKit's main thread.
    let observer = unsafe {
        NSNotificationCenter::defaultCenter().addObserverForName_object_queue_usingBlock(
            Some(NSWindowDidExitFullScreenNotification),
            Some(main_window.as_ref()),
            None,
            &block,
        )
    };
    MAIN_WINDOW_FULLSCREEN_OBSERVER
        .set(Mutex::new(MainThreadOnlyBox(observer)))
        .unwrap_or_else(|_| panic!("Main Window fullscreen observer installed more than once"));
    #[cfg(debug_assertions)]
    eprintln!("Popover: installed Main Window fullscreen-exit observer");
}

fn reset_space_membership_after_main_fullscreen() {
    #[cfg(debug_assertions)]
    eprintln!("Popover: Main Window exited fullscreen; resetting panel Space membership");
    hide();
    with_panel(|panel| {
        #[cfg(debug_assertions)]
        eprintln!(
            "Popover: collection behavior before reset: {:?}",
            panel.panel.collectionBehavior()
        );
        panel
            .panel
            .setCollectionBehavior(NSWindowCollectionBehavior::Default);
        set_collection_behavior(&panel.panel);
        #[cfg(debug_assertions)]
        eprintln!(
            "Popover: collection behavior after reset: {:?}",
            panel.panel.collectionBehavior()
        );
    });
}

/// `NSWindow.collectionBehavior` flags controlling which Space(s) the panel
/// belongs to.
///
/// Previously this used `CanJoinAllSpaces`, which makes AppKit's window
/// server genuinely render the panel on *every* Space simultaneously. That
/// caused a real, reproduced-on-device bug: swiping to a different Space
/// while the panel was open showed it briefly on the destination Space
/// before `install_dismiss_monitors`' `NSWorkspaceActiveSpaceDidChangeNotification`
/// handler could hide it — that notification fires only after the Space
/// switch (and its first rendered frame) has already completed, so the
/// app-side `hide()` always lands one frame too late.
///
/// `MoveToActiveSpace` (paired with `CanJoinAllApplications`, still needed
/// for the Fullscreen-Space-of-another-app case) instead moves the panel
/// onto whichever single Space is active *at the moment it is shown* —
/// `reposition_near_tray`/`makeKeyAndOrderFront` in `show_near_tray`, below —
/// rather than joining every Space persistently. The panel is then never
/// present on a Space the user swipes to after opening it, so there is
/// nothing for the window server to render there and nothing for the
/// dismiss-on-Space-change handler to race against. That handler is kept
/// as-is: it still hides the panel so a swipe back to the original Space
/// doesn't leave it open there either.
///
/// `IgnoresCycle` is unrelated to Space membership (see
/// docs/desktop/mac-popover.md#native-panel, "Not in Cmd+Tab or the Window
/// menu") and is unchanged. `Stationary` (unaffected by Exposé/Mission
/// Control) is dropped: `Managed`/`Transient`/`Stationary` are mutually
/// exclusive, and the panel's non-normal window level already makes
/// `Transient` (hidden by Exposé, which is what a menu-bar popup should do)
/// the default when none of the three is set explicitly.
///
/// Re-applied on every show, mirroring the defensive re-application used by
/// the earlier implementation.
fn set_collection_behavior(panel: &NSPanel) {
    panel.setCollectionBehavior(
        NSWindowCollectionBehavior::MoveToActiveSpace
            | NSWindowCollectionBehavior::IgnoresCycle
            | NSWindowCollectionBehavior::CanJoinAllApplications,
    );
}

/// Raises the panel above ordinary floating windows to the level Apple's own
/// docs name for transient, status-item-anchored popup UI. See
/// docs/desktop/mac-popover.md#native-look-and-space-behavior for why this
/// is a separate axis from `collectionBehavior`.
fn set_level(panel: &NSPanel) {
    panel.setLevel(objc2_app_kit::NSPopUpMenuWindowLevel);
}

/// Whether the Popover is currently visible.
pub fn is_visible() -> bool {
    with_panel(|panel| panel.panel.isVisible()).unwrap_or(false)
}

/// Shows the Popover, positioned near the given tray icon rect (physical,
/// top-left-origin screen coordinates — the same convention
/// `TrayIconEvent::Click::rect` and `AppHandle::cursor_position()` already
/// use elsewhere in this app), and installs the native dismiss monitors.
/// Called from the tray icon's left-click handler in main.rs, which also
/// hands over its own `TrayIcon` handle — see `install_dismiss_monitors` for
/// why the outside-click monitor needs it.
pub fn show_near_tray(
    app: &tauri::AppHandle,
    tray: tauri::tray::TrayIcon<tauri::Wry>,
    icon_rect: tauri::Rect,
) {
    with_panel(|panel| {
        set_collection_behavior(&panel.panel);
        set_level(&panel.panel);
        reposition_near_tray(app, &panel.panel, icon_rect);
        // `NonactivatingPanel` may become key without activating its owning
        // application. Making it key on show gives Escape both the native
        // AppKit route and the page-level fallback immediately, rather than
        // relying on the global event monitor alone.
        panel.panel.makeKeyAndOrderFront(None);
        if let Err(error) = panel
            .webview
            .evaluate_script("window.__refreshProvidersFromSharedCache && window.__refreshProvidersFromSharedCache()")
        {
            eprintln!("Popover: failed to refresh cached provider data on open: {error}");
        }
        panel.panel.invalidateShadow();
        panel.dismiss_monitors = Some(install_dismiss_monitors(app, tray));
    });
}

/// Hides the Popover and tears down the dismiss monitors. Idempotent: safe
/// to call while already hidden.
pub fn hide() {
    with_panel(|panel| {
        panel.panel.orderOut(None);
        if let Some(monitors) = panel.dismiss_monitors.take() {
            remove_dismiss_monitors(monitors);
        }
    });
}

/// Resizes the panel to the content height the frontend measured, clamped to
/// `[POPOVER_MIN_HEIGHT, POPOVER_MAX_HEIGHT]`, growing downward from its
/// current top-left anchor so a resize never needs a reposition — same
/// contract as the pre-`NSPanel` `set_popover_height` command (see
/// docs/desktop/mac-popover.md#window-size).
pub fn set_height(height: f64) {
    if !height.is_finite() {
        return;
    }
    let height = height.clamp(POPOVER_MIN_HEIGHT, POPOVER_MAX_HEIGHT);

    with_panel(|panel| {
        let current = panel.panel.frame();
        let top_left_y = current.origin.y + current.size.height;
        let new_frame = NSRect::new(
            NSPoint::new(current.origin.x, top_left_y - height),
            NSSize::new(POPOVER_WIDTH, height),
        );
        panel.panel.setFrame_display(new_frame, true);
        panel.panel.invalidateShadow();
        let _ = panel.webview.set_bounds(Rect {
            position: wry::dpi::LogicalPosition::new(0.0, 0.0).into(),
            size: wry::dpi::LogicalSize::new(POPOVER_WIDTH, height).into(),
        });
    });
}

/// Anchors the panel horizontally centered under the clicked tray icon,
/// `POPOVER_MENU_BAR_GAP` below it, clear of the display's edges by
/// `POPOVER_SCREEN_MARGIN` — same positioning contract as the pre-`NSPanel`
/// implementation's `position_popover_near_tray`
/// (docs/desktop/mac-popover.md#positioning), ported from Tauri's
/// `WebviewWindow`-based monitor/position APIs (still available directly on
/// `AppHandle`, independent of any particular window) to this module's own
/// `NSPanel`.
///
/// The one new piece of math here: `AppHandle`/tao's logical positions are
/// top-left-origin (y grows downward), while `NSWindow::setFrameTopLeftPoint`
/// takes a point in AppKit's screen space, which is bottom-left-origin (y
/// grows upward) — anchored to the *primary* screen's frame regardless of
/// which display the point is actually on. `flip_to_ns_y` converts between
/// the two using exactly the formula tao itself uses for the same purpose
/// (`ns_y = primary_screen_height - logical_y`).
fn reposition_near_tray(app: &tauri::AppHandle, panel: &NSPanel, icon_rect: tauri::Rect) {
    use tauri::{LogicalPosition, LogicalSize};

    let Some(monitor) = monitor_for_tray_icon(app, &icon_rect) else {
        return;
    };

    let scale_factor = monitor.scale_factor();
    let icon_position: LogicalPosition<f64> = icon_rect.position.to_logical(scale_factor);
    let icon_size: LogicalSize<f64> = icon_rect.size.to_logical(scale_factor);
    let screen_position: LogicalPosition<f64> = monitor.position().to_logical(scale_factor);
    let screen_size: LogicalSize<f64> = monitor.size().to_logical(scale_factor);

    let current_frame = panel.frame();
    let panel_width = current_frame.size.width;
    let panel_height = current_frame.size.height;

    let left = screen_position.x + POPOVER_SCREEN_MARGIN;
    let right = screen_position.x + screen_size.width - panel_width - POPOVER_SCREEN_MARGIN;
    let x =
        (icon_position.x + icon_size.width / 2.0 - panel_width / 2.0).clamp(left, right.max(left));

    let top = icon_position.y + icon_size.height + POPOVER_MENU_BAR_GAP;
    let bottom = screen_position.y + screen_size.height - panel_height - POPOVER_SCREEN_MARGIN;
    let logical_top_left_y = top.min(bottom.max(screen_position.y));

    let ns_top_left_y = flip_to_ns_y(logical_top_left_y);
    panel.setFrameTopLeftPoint(NSPoint::new(x, ns_top_left_y));
}

/// Converts a logical, top-left-origin y-coordinate (tao's/`AppHandle`'s
/// convention) into AppKit's bottom-left-origin screen-space y, anchored to
/// the primary screen's height — the same conversion tao itself performs
/// internally when it hands a logical `set_position` to `NSWindow`.
fn flip_to_ns_y(logical_top_left_y: f64) -> f64 {
    let mtm = MainThreadMarker::new().expect("must run on the main thread");
    let screens = NSScreen::screens(mtm);
    let primary_height = screens
        .firstObject()
        .map(|s| s.frame().size.height)
        .unwrap_or(0.0);
    primary_height - logical_top_left_y
}

/// Finds the display the tray icon was clicked on — see the coordinate notes
/// on `reposition_near_tray`. Falls back to the primary monitor if no
/// monitor claims the icon.
fn monitor_for_tray_icon(
    app: &tauri::AppHandle,
    icon_rect: &tauri::Rect,
) -> Option<tauri::Monitor> {
    use tauri::{LogicalPosition, LogicalSize};

    let monitors = app.available_monitors().unwrap_or_default();

    let containing = monitors.into_iter().find(|monitor| {
        let scale_factor = monitor.scale_factor();
        let icon_position: LogicalPosition<f64> = icon_rect.position.to_logical(scale_factor);
        let icon_size: LogicalSize<f64> = icon_rect.size.to_logical(scale_factor);
        let screen_position: LogicalPosition<f64> = monitor.position().to_logical(scale_factor);
        let screen_size: LogicalSize<f64> = monitor.size().to_logical(scale_factor);

        let center_x = icon_position.x + icon_size.width / 2.0;
        let center_y = icon_position.y + icon_size.height / 2.0;

        center_x >= screen_position.x
            && center_x <= screen_position.x + screen_size.width
            && center_y >= screen_position.y
            && center_y <= screen_position.y + screen_size.height
    });

    containing.or_else(|| app.primary_monitor().ok().flatten())
}

/// Installs the three native dismiss mechanisms: a click anywhere outside the
/// panel, Escape, and leaving the Space the Popover was opened on. Both local
/// and global event monitors are required: AppKit delivers global-monitor
/// events only for other applications, while the local monitors cover the
/// Main Window and the key panel itself.
///
/// Takes the `TrayIcon` handle so `dismiss_on_outside_click` can exclude its
/// own rect from "outside" — see that function's doc comment for the race
/// this avoids.
fn install_dismiss_monitors(
    app: &tauri::AppHandle,
    tray: tauri::tray::TrayIcon<tauri::Wry>,
) -> DismissMonitors {
    let click_mask = NSEventMask::LeftMouseDown | NSEventMask::RightMouseDown;
    let click_app = app.clone();
    let click_tray = tray.clone();
    let click_block = RcBlock::new(move |_event: NonNull<NSEvent>| {
        dismiss_on_outside_click(&click_app, &click_tray);
    });
    let global_click =
        NSEvent::addGlobalMonitorForEventsMatchingMask_handler(click_mask, &click_block)
            .expect("failed to install the Popover's outside-click monitor");

    let local_click_app = app.clone();
    let local_click_tray = tray.clone();
    let local_click_block = RcBlock::new(move |event: NonNull<NSEvent>| {
        dismiss_on_outside_click(&local_click_app, &local_click_tray);
        event.as_ptr()
    });
    // SAFETY: the handler returns the same non-null event pointer AppKit
    // supplied, so normal event dispatch continues after dismissal.
    let local_click = unsafe {
        NSEvent::addLocalMonitorForEventsMatchingMask_handler(click_mask, &local_click_block)
    }
    .expect("failed to install the Popover's local outside-click monitor");

    let global_key_block = RcBlock::new(move |event: NonNull<NSEvent>| {
        dismiss_on_escape(event);
    });
    let global_key = NSEvent::addGlobalMonitorForEventsMatchingMask_handler(
        NSEventMask::KeyDown,
        &global_key_block,
    )
    .expect("failed to install the Popover's Escape monitor");

    let local_key_block = RcBlock::new(move |event: NonNull<NSEvent>| {
        dismiss_on_escape(event);
        event.as_ptr()
    });
    // SAFETY: the handler returns the same non-null event pointer AppKit
    // supplied, so normal event dispatch continues after dismissal.
    let local_key = unsafe {
        NSEvent::addLocalMonitorForEventsMatchingMask_handler(
            NSEventMask::KeyDown,
            &local_key_block,
        )
    }
    .expect("failed to install the Popover's local Escape monitor");

    let space_block = RcBlock::new(move |_notification: NonNull<NSNotification>| {
        hide();
    });
    let notification_center = NSWorkspace::sharedWorkspace().notificationCenter();
    // SAFETY: `obj: None` (any poster) and `queue: None` (run synchronously
    // on the posting thread, the main thread for this notification) are both
    // explicitly permitted by the API.
    let space_change = unsafe {
        notification_center.addObserverForName_object_queue_usingBlock(
            Some(NSWorkspaceActiveSpaceDidChangeNotification),
            None,
            None,
            &space_block,
        )
    };

    DismissMonitors {
        global_click,
        local_click,
        global_key,
        local_key,
        space_change,
    }
}

fn remove_dismiss_monitors(monitors: DismissMonitors) {
    // SAFETY: all five tokens were returned by a prior matching
    // add-monitor/add-observer call and have not been removed since.
    unsafe {
        NSEvent::removeMonitor(&monitors.global_click);
        NSEvent::removeMonitor(&monitors.local_click);
        NSEvent::removeMonitor(&monitors.global_key);
        NSEvent::removeMonitor(&monitors.local_key);
        NSWorkspace::sharedWorkspace()
            .notificationCenter()
            .removeObserver_name_object(monitors.space_change.as_ref(), None, None);
    }
}

/// The click half of `install_dismiss_monitors`: hides the Popover if the
/// current mouse-down location is outside both the panel's own frame *and*
/// the tray icon's own rect.
///
/// The tray icon exclusion matters because of a real race: this is a global
/// `NSEvent` monitor, so it sees the tray icon's own `mouseDown` — which
/// fires *before* `mouseUp`, the event `TrayIconEvent::Click` (handled in
/// main.rs's tray click handler) fires on. Without excluding the tray
/// icon's rect here, clicking it a second time to close an open Popover
/// would hide the panel on that `mouseDown` first; the tray click handler's
/// own toggle-off check (`popover_panel::is_visible()`) would then run on
/// `mouseUp` against an already-hidden panel and reopen it instead of
/// leaving it closed — the exact bug this exclusion fixes. Mirrors
/// `point_is_inside_tray_icon` in the pre-`NSPanel` implementation.
fn dismiss_on_outside_click(app: &tauri::AppHandle, tray: &tauri::tray::TrayIcon<tauri::Wry>) {
    if !is_visible() {
        return;
    }

    let Ok(cursor) = app.cursor_position() else {
        return;
    };

    let inside_panel =
        with_panel(|panel| point_is_inside_panel(&panel.panel, cursor)).unwrap_or(false);
    if inside_panel || point_is_inside_tray_icon(tray, cursor) {
        return;
    }

    hide();
}

/// Hides the Popover for the layout-independent virtual Escape keycode.
fn dismiss_on_escape(event: NonNull<NSEvent>) {
    const ESCAPE_KEY_CODE: u16 = 53;
    // SAFETY: AppKit only ever invokes an event-monitor block with a live event.
    if unsafe { event.as_ref() }.keyCode() == ESCAPE_KEY_CODE {
        hide();
    }
}

/// Whether `point` (physical, top-left-origin screen coordinates — the same
/// convention `AppHandle::cursor_position()` uses) falls inside the panel's
/// current on-screen frame.
fn point_is_inside_panel(panel: &NSPanel, point: tauri::PhysicalPosition<f64>) -> bool {
    let frame = panel.frame();
    let mtm = MainThreadMarker::new().expect("must run on the main thread");
    let screens = NSScreen::screens(mtm);
    let primary_height = screens
        .firstObject()
        .map(|s| s.frame().size.height)
        .unwrap_or(0.0);
    // Convert the panel's AppKit (bottom-left-origin) frame back to
    // physical, top-left-origin coordinates to compare against `point`'s
    // convention.
    let top_left_y_logical = primary_height - (frame.origin.y + frame.size.height);
    let scale_factor = panel_scale_factor(panel);
    let left = frame.origin.x * scale_factor;
    let right = (frame.origin.x + frame.size.width) * scale_factor;
    let top = top_left_y_logical * scale_factor;
    let bottom = (top_left_y_logical + frame.size.height) * scale_factor;

    point.x >= left && point.x <= right && point.y >= top && point.y <= bottom
}

/// Whether `point` (physical, top-left-origin screen coordinates) falls
/// inside the tray icon's current clickable rect. `TrayIcon::rect()`'s
/// `position`/`size` come out of tray-icon's macOS backend already in
/// physical pixels (the same convention `TrayIconEvent::Click::rect` and
/// `cursor_position()` use — see `position_popover_near_tray`'s pre-`NSPanel`
/// doc comment for the same observation), so `to_physical` below is a
/// same-value no-op; the scale factor passed does not matter, but the
/// panel's own is the most convenient one on hand.
fn point_is_inside_tray_icon(
    tray: &tauri::tray::TrayIcon<tauri::Wry>,
    point: tauri::PhysicalPosition<f64>,
) -> bool {
    let Ok(Some(rect)) = tray.rect() else {
        return false;
    };

    let scale_factor = with_panel(|panel| panel_scale_factor(&panel.panel)).unwrap_or(1.0);
    let position = rect.position.to_physical::<f64>(scale_factor);
    let size = rect.size.to_physical::<f64>(scale_factor);

    point.x >= position.x
        && point.x <= position.x + size.width
        && point.y >= position.y
        && point.y <= position.y + size.height
}

fn panel_scale_factor(panel: &NSPanel) -> f64 {
    panel
        .screen()
        .map(|screen| screen.backingScaleFactor())
        .unwrap_or(1.0)
}
