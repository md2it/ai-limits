import { getUpdateFrequency, isProviderEnabled } from "./settings.js";

const providerNextRefreshAt = new Map();
// Epoch ms of this provider's last actual collection. Seeded from the
// backend's `collectedAt` (raw ISO, unlike `dataTimestamp` which arrives
// pre-formatted for display) whenever it is known, so the schedule is based
// on the shared collection instant rather than whenever this particular
// window happened to observe a fetch resolve — both Main Window and Popover
// compute the same next-refresh target from the same collection. Falls back
// to this call's own Date.now() only when no `collectedAt` is available
// (a failed fetch, or a snapshot that predates this field).
const providerLastUpdateAt = new Map();

const LEGACY_PROVIDER_INTERVALS_STORAGE_KEY = "ai-limits-provider-intervals";

// Drops the pre-shared-frequency per-provider intervals key if it is still
// present from an earlier install. Safe to call every startup.
export function initProviderIntervals() {
  localStorage.removeItem(LEGACY_PROVIDER_INTERVALS_STORAGE_KEY);
}

export function updateFrequencyToSeconds(frequency) {
  switch (frequency) {
    case "1 min":
      return 60;
    case "5 min":
      return 300;
    case "10 min":
      return 600;
    case "30 min":
      return 1_800;
    case "1 hour":
      return 3_600;
    default:
      return null;
  }
}

// null means the next scheduled refresh is unknown or there is none (manual only).
export function getProviderNextRefreshAt(providerId) {
  return providerNextRefreshAt.get(providerId) ?? null;
}

export function clearProviderRefreshProjection(providerId) {
  providerNextRefreshAt.set(providerId, null);
}

// Marks `providerId`'s last update instant. Pass the backend's raw
// `collectedAt` when available, so the schedule is anchored to the actual
// collection time shared by every surface; pass nothing (or an unparseable
// value) to fall back to this call's own Date.now() — used as the retry
// anchor after a failed fetch, where no collection happened at all: an
// attempt just happened either way, so the next one is a full interval out
// rather than an immediate hot loop. Does not itself (re)schedule anything;
// pair with recalculateProviderNextRefreshAt.
export function recordProviderUpdateNow(providerId, collectedAt) {
  const parsed = collectedAt ? Date.parse(collectedAt) : NaN;
  providerLastUpdateAt.set(providerId, Number.isNaN(parsed) ? Date.now() : parsed);
}

// Projects the native background scheduler's next target into frontend state
// for display. Actual wakeups belong to the application process, not to a
// hidden webview timer; provider-updated/provider-refresh-failed events call
// this again after every real attempt and keep the projection aligned.
export function recalculateProviderNextRefreshAt(providerId) {
  if (!isProviderEnabled(providerId)) {
    providerNextRefreshAt.set(providerId, null);
    return;
  }

  const intervalSeconds = updateFrequencyToSeconds(getUpdateFrequency());
  if (intervalSeconds == null) {
    providerNextRefreshAt.set(providerId, null);
    return;
  }

  const lastUpdateMs = providerLastUpdateAt.get(providerId) ?? null;
  if (lastUpdateMs == null) {
    providerNextRefreshAt.set(providerId, Date.now());
    return;
  }

  const nextRefreshAt = lastUpdateMs + intervalSeconds * 1000;
  providerNextRefreshAt.set(providerId, nextRefreshAt);
}
