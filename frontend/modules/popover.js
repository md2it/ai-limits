// Entry point for the real macOS Menu Bar Popover window (frontend/popover.html).
//
// Unlike the `?showcase=popover` preview in showcase.js — which relocates the
// Main Window's live `#provider-list` DOM node because in a browser preview
// "Main Window" and "Popover" are the same JS context — this file runs in
// its own, separate Tauri WebviewWindow. There is no DOM to relocate here:
// this module mounts and drives its own real provider cards through the same
// providers.js pipeline the Main Window uses (frontend/modules/main.js),
// against the real `get_single_provider_limits` Tauri command and the real
// settings/localStorage model. See docs/desktop/mac-popover.md.
//
// Loading this file directly in a browser is expected to work structurally:
// `window.__TAURI__` will be undefined, so providers.js's real IPC path
// throws and the cards show the same "Tauri API is unavailable" error state
// the Main Window would show under the same condition.
import { SETTINGS_CHANGED_EVENT, THEME_CHANGED_EVENT } from "./constants.js";
import { initTheme, applyAppTheme, reloadAppTheme } from "./theme.js";
import { initSettings, reloadAppSettings } from "./settings.js";
import {
  initProviders,
  initProviderIntervals,
  listenForSystemThemeMeterRefresh,
  refreshEnabledProviders,
  refreshProviderSectionsFromCache,
  applySharedUpdateFrequency,
  syncBackgroundRefreshSchedule,
} from "./providers.js";
import {
  buildPopoverHeaderHtml,
  attachPopoverHeaderHandlers,
  observePopoverScroll,
} from "./popover-toolbar.js";

const popoverRoot = document.querySelector("#popover-root");
const providerList = document.querySelector("#provider-list");
const statusLine = document.querySelector("#status-line");
const headerMount = document.querySelector("#popover-header-mount");

// Resolves Tauri's invoke, or null outside Tauri (this page opened in a plain
// browser). Every call site below no-ops in that case rather than throwing.
function tauriInvoke() {
  return window.__TAURI__?.core?.invoke ?? null;
}

// Real cross-window navigation into the Main Window, backing
// window.__openMainWindowHelp/__openMainWindowSettings/__openMainApplication
// (see docs/desktop/mac-popover.md#toolbar and #entry-points). Each invokes a
// Tauri command that shows+focuses the "main" window
// (src-tauri/src/commands/mod.rs: open_main_window / open_main_window_settings
// / open_main_window_help). Invoke rejections are swallowed: there is no UI
// here to surface an error to.
window.__openMainWindowHelp = (chapterId) => {
  tauriInvoke()?.("open_main_window_help", { chapter: chapterId ?? null }).catch(() => {});
};

window.__openMainWindowSettings = () => {
  tauriInvoke()?.("open_main_window_settings").catch(() => {});
};

window.__openMainApplication = () => {
  tauriInvoke()?.("open_main_window").catch(() => {});
};

// [info], [gear] and "Open Application" are pure navigation into the Main
// Window (see docs/desktop/mac-popover.md#toolbar) — the Popover renders no
// Help or Settings UI itself. [update all] is real, local behavior:
// refreshes this window's own mounted provider cards, no cross-window IPC
// needed.
const headerHandlers = {
  onOpenApp() {
    window.__openMainApplication?.();
  },
  onUpdateAll() {
    refreshEnabledProviders();
  },
  onInfo() {
    window.__openMainWindowHelp?.();
  },
  onGear() {
    window.__openMainWindowSettings?.();
  },
};

// The Popover has no Settings UI of its own (see Roles in
// docs/desktop/mac-popover.md), so there are no setting input elements to
// hand to initSettings/initTheme here — it only needs to read the same
// localStorage-backed state the Main Window writes to. Passing an empty
// object for `inputs` is safe: settings.js only dereferences it from
// syncSettingsInputs/handleSettingsChange/handleDisplaySettingsChange, none
// of which this window calls (there's nothing here for the user to change
// locally). For the same reason no onChanged/onDisplayChanged callbacks are
// passed: they only fire for a change made *within* this window, which
// cannot happen. The cross-window signals are the Tauri event listeners
// further down.
initTheme(null);
initSettings({});
initProviders({ providerList, statusLine }, { surface: "popover" });

// The header is mounted once and never rebuilt, so its handlers are attached
// once too.
headerMount.innerHTML = buildPopoverHeaderHtml();
attachPopoverHeaderHandlers(popoverRoot, headerHandlers);
observePopoverScroll(popoverRoot);

// Cross-window settings sync (Main Window -> Popover; see
// docs/desktop/mac-popover.md — "Display settings apply synchronously"
// under Intent). settings.js emits SETTINGS_CHANGED_EVENT after any settings
// save, in *any* window running under Tauri; this window is the only listener
// today (the Popover has no settings UI to originate the event itself). Each
// Tauri webview window is its own JS context with its own copy of
// settings.js's module-scoped `appSettings`, so localStorage alone does not
// make an already-open Popover notice a Main Window change — reloadAppSettings()
// re-reads it here first, then the already-mounted card sections (which
// providers.js itself filters down to enabled providers) are refreshed from
// that freshly-loaded state.
//
// Both a provider-visibility change and a display-toggle change are cheap
// and idempotent to redo, so this handler does not need to branch on
// `kind` — it simply keeps both in sync unconditionally on every event.
if (window.__TAURI__?.event?.listen) {
  window.__TAURI__.event.listen(SETTINGS_CHANGED_EVENT, () => {
    reloadAppSettings();
    refreshProviderSectionsFromCache();
    applySharedUpdateFrequency();
    syncBackgroundRefreshSchedule();
  });

  // Same mechanism for the manual "Dark theme" toggle, which lives in
  // theme.js's own storage key and module rather than in settings.js.
  // System theme changes need no event — the media-query listeners below
  // fire in this window independently.
  window.__TAURI__.event.listen(THEME_CHANGED_EVENT, () => {
    reloadAppTheme();
    refreshProviderSectionsFromCache();
  });
}

// Content-driven window height (see docs/desktop/mac-popover.md#window-size).
// The native side owns the width and clamps the height, so this only has to
// report what the panel would like to be: the header strip plus the natural
// height of the card list. It deliberately does not measure
// `.popover-scroll` itself — that element is stretched to whatever height the
// window currently has, which would make the measurement circular and pin the
// window at its first size forever.
function reportPopoverHeight() {
  const invoke = tauriInvoke();
  if (!invoke) {
    return;
  }

  const header = popoverRoot.querySelector(".popover-header");
  const height =
    (header?.offsetHeight ?? 0) +
    providerList.offsetHeight +
    (statusLine.hidden ? 0 : statusLine.offsetHeight);

  invoke("set_popover_height", { height }).catch(() => {});
}

// The card list changes height on nearly every interaction — data resolving,
// a display toggle syncing in from the Main
// Window — so the report is driven by observing it rather than hooked into
// each of those call sites.
if (typeof window.ResizeObserver === "function") {
  const heightObserver = new ResizeObserver(reportPopoverHeight);
  heightObserver.observe(providerList);
  heightObserver.observe(statusLine);
}

// Esc closes the panel, the way a system popover does.
// `hide_popover` is a macOS-only Tauri command; outside Tauri
// (browser/showcase) invoke resolves to null and this is a no-op.
document.addEventListener(
  "keydown",
  (event) => {
    if (event.key !== "Escape") {
      return;
    }

    tauriInvoke()?.("hide_popover").catch(() => {});
  },
  { capture: true },
);

// A native panel has no WebKit "Reload / Inspect Element" context menu.
document.addEventListener("contextmenu", (event) => {
  event.preventDefault();
});

// `makeKeyAndOrderFront` in popover_panel.rs's `show_near_tray` makes the
// panel key on every show (needed so Escape/outside-click work immediately —
// see docs/desktop/mac-popover.md#native-panel). WebKit reacts to a webview
// becoming key by auto-focusing the first focusable element in the page —
// here, the header's "AI Limits" button — which then paints the
// `:focus-visible` ring as if the user had tabbed to it, on a panel that was
// only ever clicked open. The panel has no keyboard-driven navigation to
// preserve, so this just blurs whatever WebKit auto-focused every time the
// window regains focus, rather than trying to special-case that one button.
window.addEventListener("focus", () => {
  if (document.activeElement instanceof HTMLElement && document.activeElement !== document.body) {
    document.activeElement.blur();
  }
});

listenForSystemThemeMeterRefresh();

initProviderIntervals();
applyAppTheme();
syncBackgroundRefreshSchedule();
refreshEnabledProviders({ initial: true });
