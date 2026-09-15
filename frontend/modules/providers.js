import {
  PROVIDER_IDS,
  PROVIDER_REFRESH_FAILED_EVENT,
  PROVIDER_REFRESH_STARTED_EVENT,
  PROVIDER_UPDATED_EVENT,
} from "./constants.js";
import { getUpdateFrequency, isProviderEnabled, settingsToQuery } from "./settings.js";
import { syncSystemTheme } from "./theme.js";
import { openHelp } from "./help.js";
import { openExternalUrl } from "./links.js";
import { isScreenshotShowcase, SHOWCASE_PROVIDERS } from "./showcase.js";
import { clearProviderRefreshProjection, getProviderNextRefreshAt, recalculateProviderNextRefreshAt, recordProviderUpdateNow, updateFrequencyToSeconds } from "./provider-refresh-intervals.js";
import { initSectionSlotAlignment, scheduleSectionSlotAlignment } from "./provider-section-alignment.js";
import { createEmptyProvider, renderProvider, updateProviderBlockData, updateProviderUpdateTimeText } from "./provider-rendering.js";

export { initProviderIntervals } from "./provider-refresh-intervals.js";
export { scheduleSectionSlotAlignment };

let providerList = null;
let statusLine = null;
let providerSurface = "main";

const providerRefreshInFlight = new Set();
// Providers with a collection currently in flight in *another* surface,
// tracked from PROVIDER_REFRESH_STARTED_EVENT until the matching
// PROVIDER_UPDATED_EVENT/PROVIDER_REFRESH_FAILED_EVENT arrives. Kept separate
// from providerRefreshInFlight (this window's own requests) so the two can be
// unioned in updateRefreshVisual without either clearing the other early.
const providerRemoteRefreshInFlight = new Set();
// Providers currently playing a short, non-loading "something changed" flash
// (see flashRemoteUpdate) — a card that receives applied data or a failure it
// never saw a providerRemoteRefreshInFlight signal for (the started event did
// not arrive, or arrived and resolved between two ticks). Kept separate from
// providerRemoteRefreshInFlight so its own short timeout doesn't fight a
// still-active loading state for the same provider.
const providerFlashRefreshing = new Set();
// How long the flash plays: one full pass of the "is-refreshing" glare
// (see @keyframes provider-refresh-glare in styles/providers.css), long
// enough to read as "this card just changed" without lingering like a real
// loading state.
const REMOTE_UPDATE_FLASH_MS = 1800;
const providerDataCache = new Map();

export function initProviders(elements, { surface = "main" } = {}) {
  providerList = elements.providerList;
  statusLine = elements.statusLine;
  providerSurface = surface;
  initSectionSlotAlignment(providerList);

  // Backend-emitted (not by any frontend module): fires after every
  // successful actual collection, from whichever surface started it. Lets
  // this window pick up a result the other open surface collected without
  // starting a collection of its own — see PROVIDER_UPDATED_EVENT in
  // constants.js and docs/desktop/ui/frontend-state.md.
  window.__TAURI__?.event?.listen?.(PROVIDER_UPDATED_EVENT, (event) => {
    applyRemoteProviderUpdate(event.payload);
  });

  // Backend-emitted, fires right as a collection begins, from whichever
  // surface started it (including this one — the event is app-wide and this
  // window receives its own). Lets this window's card show the same
  // in-flight refresh animation a collection started elsewhere is already
  // showing there, without starting a collection of its own.
  window.__TAURI__?.event?.listen?.(PROVIDER_REFRESH_STARTED_EVENT, (event) => {
    const providerId = event.payload?.id;
    if (typeof providerId !== "string") {
      return;
    }
    providerRemoteRefreshInFlight.add(providerId);
    updateRefreshVisual(providerId);
  });

  // Backend-emitted, fires after a failed actual collection. Lets a surface
  // that did not start the collection show the same error state instead of
  // sitting on stale data with no explanation.
  window.__TAURI__?.event?.listen?.(PROVIDER_REFRESH_FAILED_EVENT, (event) => {
    applyRemoteProviderFailure(event.payload);
  });
}

// The frontend remains the owner of saved settings, while the application
// process owns periodic wakeups so refresh continues reliably with every
// window hidden. Re-sending an unchanged configuration is harmless and lets
// either surface initialize the single scheduler.
export function syncBackgroundRefreshSchedule() {
  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) {
    return Promise.resolve();
  }

  return invoke("configure_background_refresh", {
    config: {
      query: settingsToQuery(),
      intervalSeconds: updateFrequencyToSeconds(getUpdateFrequency()),
    },
  }).catch((error) => {
    console.error("Could not configure background refresh", error);
  });
}

// Reflects the union of this window's own in-flight refresh
// (providerRefreshInFlight), a refresh started elsewhere but not yet resolved
// (providerRemoteRefreshInFlight), and a short post-hoc flash
// (providerFlashRefreshing) onto the card's busy state, so one animation
// lifecycle covers every case a card's content can change from.
function updateRefreshVisual(providerId) {
  const block = getProviderBlock(providerId);
  if (!block) {
    return;
  }

  const isLoading =
    providerRefreshInFlight.has(providerId) ||
    providerRemoteRefreshInFlight.has(providerId) ||
    providerFlashRefreshing.has(providerId);
  block.classList.toggle("is-refreshing", isLoading);

  for (const manualRefreshOption of block.querySelectorAll("[data-manual-refresh]")) {
    manualRefreshOption.disabled = isLoading;
  }
}

// Plays a short, self-clearing "is-refreshing" flash for a card whose content
// just changed without this window ever seeing a providerRemoteRefreshInFlight
// signal for it (the provider-refresh-started event did not arrive, or
// already resolved by the time this ran) — so a card never silently changes
// content with no visible feedback, regardless of what the started event did.
function flashRemoteUpdate(providerId) {
  providerFlashRefreshing.add(providerId);
  updateRefreshVisual(providerId);
  window.setTimeout(() => {
    providerFlashRefreshing.delete(providerId);
    updateRefreshVisual(providerId);
  }, REMOTE_UPDATE_FLASH_MS);
}

// Applies a provider snapshot collected by *this window's own* in-flight
// refresh is deliberately skipped here: refreshSingleProvider's own success
// path already renders it, and rendering it twice from two code paths would
// race. This only runs for a result collected elsewhere. The remote in-flight
// marker is cleared unconditionally first, regardless of which path renders
// the result, so the loading animation always clears when the collection
// ends; if this window never saw the matching started signal, a short flash
// plays instead so the change is still visible.
function applyRemoteProviderUpdate(provider) {
  if (!provider || typeof provider.id !== "string") {
    return;
  }

  const wasAnimatingRemoteStart = providerRemoteRefreshInFlight.delete(provider.id);
  updateRefreshVisual(provider.id);

  if (providerRefreshInFlight.has(provider.id)) {
    return;
  }

  if (!wasAnimatingRemoteStart) {
    flashRemoteUpdate(provider.id);
  }

  provider.pending = false;
  cacheProviderData(provider);
  recordProviderUpdateNow(provider.id, provider.collectedAt);

  if (!isProviderEnabled(provider.id)) {
    return;
  }

  const block = getProviderBlock(provider.id);
  if (!block) {
    return;
  }

  recalculateProviderNextRefreshAt(provider.id);
  updateProviderBlockData(block, provider, getProviderNextRefreshAt(provider.id));
  attachSectionHandlers(block, provider.id);
  scheduleSectionSlotAlignment();
}

// Same shape and same skip-if-this-window-already-owns-it guard as
// applyRemoteProviderUpdate, for a collection that failed instead of
// succeeding. `provider` is the same ProviderLimits shape with errorMessage
// set and limits empty (built by the backend's provider_error), which the
// existing card renderer already knows how to show as an error state.
// recordProviderUpdateNow is called without a collectedAt, same as the local
// catch-path failure handling in refreshSingleProvider — a failed collection
// has no collectedAt, so the retry anchors to this moment instead.
function applyRemoteProviderFailure(provider) {
  if (!provider || typeof provider.id !== "string") {
    return;
  }

  const wasAnimatingRemoteStart = providerRemoteRefreshInFlight.delete(provider.id);
  updateRefreshVisual(provider.id);

  if (providerRefreshInFlight.has(provider.id)) {
    return;
  }

  if (!wasAnimatingRemoteStart) {
    flashRemoteUpdate(provider.id);
  }

  provider.pending = false;
  cacheProviderData(provider);
  recordProviderUpdateNow(provider.id);

  if (!isProviderEnabled(provider.id)) {
    return;
  }

  const block = getProviderBlock(provider.id);
  if (!block) {
    return;
  }

  recalculateProviderNextRefreshAt(provider.id);
  updateProviderBlockData(block, provider, getProviderNextRefreshAt(provider.id));
  attachSectionHandlers(block, provider.id);
  scheduleSectionSlotAlignment();
}

function getProviderBlock(providerId) {
  return providerList.querySelector(`[data-provider-id="${providerId}"]`);
}

function attachProviderBlockHandlers(block, providerId) {
  attachSectionHandlers(block, providerId);
}

function attachDataErrorsBlockHandlers(block) {
  const dataErrorsButton = block.querySelector("[data-open-data-errors]");
  if (dataErrorsButton) {
    dataErrorsButton.addEventListener("click", () => {
      openHelp("data-errors");
    });
  }
}

// Generic counterpart to attachDataErrorsBlockHandlers's fixed
// data-open-data-errors wiring: any card element carrying data-open-help
// opens the named Help chapter (e.g. the CLI-not-signed-in state's "Fix
// access" link, which opens the "permissions" chapter).
function attachHelpLinkHandlers(block) {
  for (const link of block.querySelectorAll("[data-open-help]")) {
    link.addEventListener("click", () => {
      openHelp(link.dataset.openHelp);
    });
  }
}

function attachCliAuthorizationHandlers(block) {
  const loginButton = block.querySelector("[data-provider-cli-login]");
  if (!loginButton) {
    return;
  }

  loginButton.addEventListener("click", () => {
    startProviderCliLogin(loginButton.dataset.providerCliLogin);
  });
}

function attachPlanLinkHandlers(block) {
  for (const link of block.querySelectorAll("[data-plan-link-url]")) {
    link.addEventListener("click", () => {
      openExternalUrl(link.dataset.planLinkUrl);
    });
  }
}

// The card's inline "Retry" button (surface === "main" only) lives inside
// `.provider-sections`, which updateProviderBlockData fully replaces on every
// refresh, so it's rebound here rather than once at mount.
function attachRetryHandlers(block, providerId) {
  const sections = block.querySelector(".provider-sections");
  if (!sections) {
    return;
  }

  for (const button of sections.querySelectorAll("[data-manual-refresh]")) {
    button.addEventListener("click", () => {
      refreshSingleProvider(providerId);
    });
  }
}

// Recomputes every enabled provider's next-refresh target from the shared
// update-frequency setting and last collection instant, and refreshes the
// update-time line text. Native schedule synchronization is a separate call
// so this presentation-only function never starts a fetch.
export function applySharedUpdateFrequency() {
  for (const providerId of PROVIDER_IDS) {
    if (!isProviderEnabled(providerId)) {
      continue;
    }

    recalculateProviderNextRefreshAt(providerId);
    const block = getProviderBlock(providerId);
    if (!block) {
      continue;
    }

    updateProviderUpdateTimeText(
      block,
      providerDataCache.get(providerId) ?? { pending: true },
      getProviderNextRefreshAt(providerId),
    );
  }
}

function attachSectionHandlers(block, providerId) {
  attachRetryHandlers(block, providerId);
  attachHelpLinkHandlers(block);
  attachDataErrorsBlockHandlers(block);
  attachCliAuthorizationHandlers(block);
  attachPlanLinkHandlers(block);
}

// Display toggles (Show limits / Show plan) never trigger a
// refresh: they re-render every already-mounted provider block from the
// data already held in providerDataCache, instantly and without a backend
// call. See handleDisplaySettingsChange in settings.js.
export function refreshProviderSectionsFromCache() {
  for (const provider of providerDataCache.values()) {
    const block = getProviderBlock(provider.id);
    if (!block) {
      continue;
    }

    updateProviderBlockData(block, provider, getProviderNextRefreshAt(provider.id));
    attachSectionHandlers(block, provider.id);
  }

  scheduleSectionSlotAlignment();
}

// Meter fill colors are theme-dependent (see colorForRemaining in
// provider-formatters.js) but only computed at render time, so a system
// theme flip has to force a from-cache re-render or already-mounted meters
// keep the previous theme's color. Shared by main.js and popover.js.
export function listenForSystemThemeMeterRefresh() {
  function onSystemThemeChange() {
    syncSystemTheme();
    refreshProviderSectionsFromCache();
  }

  if (typeof window.matchMedia === "function") {
    window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", onSystemThemeChange);
    window.matchMedia("(prefers-color-scheme: light)").addEventListener("change", onSystemThemeChange);
  }
}

async function startProviderCliLogin(provider) {
  if (provider !== "codex" && provider !== "claude") {
    return;
  }

  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) {
    return;
  }

  try {
    await invoke("start_provider_cli_login", { provider });
  } catch (error) {
    setErrorState(error?.message || String(error) || "Could not start provider sign-in.");
  }
}

function mountProviderBlock(provider) {
  recalculateProviderNextRefreshAt(provider.id);
  const block = renderProvider(provider, getProviderNextRefreshAt(provider.id), providerSurface);
  attachProviderBlockHandlers(block, provider.id);
  return block;
}

function setErrorState(message) {
  statusLine.hidden = false;
  statusLine.classList.remove("loading");
  statusLine.classList.add("error");
  statusLine.textContent = message;
}

function clearStatusState() {
  statusLine.textContent = "";
  statusLine.hidden = true;
  statusLine.classList.remove("loading", "error");
}

function cacheProviderData(provider) {
  providerDataCache.set(provider.id, { ...provider, pending: false });
}

function hasCachedProviderData(providerId) {
  const cached = providerDataCache.get(providerId);
  return cached != null && !cached.pending;
}

function insertProviderBlockInOrder(block, providerId) {
  const enabledProviders = PROVIDER_IDS.filter(isProviderEnabled);
  const targetIndex = enabledProviders.indexOf(providerId);

  for (let i = targetIndex + 1; i < enabledProviders.length; i += 1) {
    const nextBlock = getProviderBlock(enabledProviders[i]);
    if (nextBlock) {
      providerList.insertBefore(block, nextBlock);
      return;
    }
  }

  providerList.appendChild(block);
}

export function removeDisabledProviderBlocks() {
  for (const providerId of PROVIDER_IDS) {
    if (isProviderEnabled(providerId)) {
      continue;
    }

    clearProviderRefreshProjection(providerId);
    getProviderBlock(providerId)?.remove();
  }

  scheduleSectionSlotAlignment();
}

export function restoreNewlyEnabledProviders(providerIds) {
  if (!providerIds.length) {
    return;
  }

  clearStatusState();

  for (const providerId of providerIds) {
    if (getProviderBlock(providerId)) {
      continue;
    }

    if (hasCachedProviderData(providerId)) {
      const block = mountProviderBlock(providerDataCache.get(providerId));
      insertProviderBlockInOrder(block, providerId);
      continue;
    }

    const block = mountProviderBlock(createEmptyProvider(providerId));
    insertProviderBlockInOrder(block, providerId);
    refreshSingleProvider(providerId);
  }

  scheduleSectionSlotAlignment();
}

async function fetchSingleProviderLimits(providerId) {
  if (isScreenshotShowcase) {
    return {
      ...SHOWCASE_PROVIDERS[providerId],
      limits: SHOWCASE_PROVIDERS[providerId].limits.map((limit) => ({ ...limit })),
    };
  }

  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) {
    throw new Error("Tauri API is unavailable");
  }

  return invoke("get_single_provider_limits", {
    providerId,
    query: settingsToQuery(),
  });
}

// Reads the shared structured-data snapshot another surface may already
// have collected for `providerId`, without starting or joining a
// collection. Returns null if nothing is cached yet (or outside Tauri),
// which the caller treats the same as "needs its own collection".
async function loadCachedProviderLimits(providerId) {
  if (isScreenshotShowcase) {
    return null;
  }

  const invoke = window.__TAURI__?.core?.invoke;
  if (!invoke) {
    return null;
  }

  try {
    return await invoke("get_cached_provider_limits", { providerId });
  } catch {
    return null;
  }
}

// Reconciles an already-initialized surface with the application-wide cache.
// Opening a hidden Main Window or Popover calls this without starting a new
// collection: the user sees every result already known to the application
// immediately, while the native scheduler remains responsible for freshness.
export async function refreshProvidersFromSharedCache() {
  removeDisabledProviderBlocks();

  const enabledProviders = PROVIDER_IDS.filter(isProviderEnabled);
  const cachedSnapshots = await Promise.all(enabledProviders.map(loadCachedProviderLimits));

  for (const [index, providerId] of enabledProviders.entries()) {
    const snapshot = cachedSnapshots[index];
    if (!snapshot) {
      continue;
    }

    snapshot.pending = false;
    cacheProviderData(snapshot);
    recordProviderUpdateNow(providerId, snapshot.collectedAt);

    let block = getProviderBlock(providerId);
    if (!block) {
      block = mountProviderBlock(snapshot);
      insertProviderBlockInOrder(block, providerId);
      continue;
    }

    recalculateProviderNextRefreshAt(providerId);
    updateProviderBlockData(block, snapshot, getProviderNextRefreshAt(providerId));
    attachSectionHandlers(block, providerId);
  }

  scheduleSectionSlotAlignment();
}

// A failed fetch previously rejected silently: providerRefreshInFlight was
// still cleared in `finally`, but nothing told the user the click did
// anything. Manual and scheduled refreshes share this function, so surfacing
// the error here covers both. The busy state now reads on the whole card
// (the "is-refreshing" glare) rather than on a dedicated button.
async function refreshSingleProvider(providerId) {
  if (!isProviderEnabled(providerId) || providerRefreshInFlight.has(providerId)) {
    return;
  }

  providerRefreshInFlight.add(providerId);
  updateRefreshVisual(providerId);

  try {
    const provider = await fetchSingleProviderLimits(providerId);
    provider.pending = false;
    cacheProviderData(provider);
    recordProviderUpdateNow(providerId, provider.collectedAt);

    if (!isProviderEnabled(providerId)) {
      return;
    }

    let block = getProviderBlock(providerId);
    if (!block) {
      block = mountProviderBlock(provider);
      insertProviderBlockInOrder(block, providerId);
    } else {
      recalculateProviderNextRefreshAt(providerId);
      updateProviderBlockData(block, provider, getProviderNextRefreshAt(providerId));
      attachSectionHandlers(block, providerId);
    }
  } catch (error) {
    setErrorState(error?.message || String(error) || `Could not refresh ${providerId}.`);
    // The fetch attempt itself is what just happened "now" — anchor the
    // retry to this moment so it's a full interval out, rather than treating
    // it as an unknown last update, which would trigger another immediate
    // retry and hammer a persistently failing source.
    recordProviderUpdateNow(providerId);
    recalculateProviderNextRefreshAt(providerId);
  } finally {
    providerRefreshInFlight.delete(providerId);
    updateRefreshVisual(providerId);
    scheduleSectionSlotAlignment();
  }
}

export async function refreshEnabledProviders({ initial = false } = {}) {
  removeDisabledProviderBlocks();

  const enabledProviders = PROVIDER_IDS.filter(isProviderEnabled);
  if (!enabledProviders.length) {
    providerList.replaceChildren();
    clearStatusState();
    setErrorState("Enable at least one provider in settings.");
    return;
  }

  clearStatusState();

  if (initial) {
    // A provider the other open surface already collected this session
    // renders from that shared snapshot immediately; mountProviderBlock's
    // own recalculateProviderNextRefreshAt call (fed that snapshot's collectedAt
    // below) projects its next update for display. The native scheduler
    // independently decides whether another collection is due, so this
    // surface collects only when no shared snapshot exists yet.
    const cachedSnapshots = await Promise.all(enabledProviders.map(loadCachedProviderLimits));
    const providersNeedingCollection = [];

    providerList.replaceChildren(
      ...enabledProviders.map((providerId, index) => {
        const snapshot = cachedSnapshots[index];
        if (!snapshot) {
          providersNeedingCollection.push(providerId);
          return mountProviderBlock(createEmptyProvider(providerId));
        }

        snapshot.pending = false;
        cacheProviderData(snapshot);
        recordProviderUpdateNow(providerId, snapshot.collectedAt);
        return mountProviderBlock(snapshot);
      }),
    );
    scheduleSectionSlotAlignment();

    for (const providerId of providersNeedingCollection) {
      refreshSingleProvider(providerId);
    }
    return;
  }

  for (const providerId of enabledProviders) {
    refreshSingleProvider(providerId);
  }
}
