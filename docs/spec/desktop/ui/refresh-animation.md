# Provider Card Refresh Animation

This document is the single source of truth for when a provider card plays its "is-refreshing" glare (`.provider-block.is-refreshing`, the diagonal glare sweep defined in `frontend/styles/providers.css`) and how that lifecycle stays consistent between Main Window and Menu Bar Popover. The underlying cross-window data/event contract is documented in [frontend-state.md](frontend-state.md#shared-structured-data-cache) and [refresh.md](refresh.md#shared-refresh-schedule) — this document only covers the animation on top of it.

All of the state below lives in `frontend/modules/providers.js` and is per-window (each surface's own JS module scope), never shared storage — only the three backend events described below cross the window boundary.

## The three animation states

A provider card's busy state is the union of three independent, per-provider tracking sets, reconciled by `updateRefreshVisual(providerId)` into one `is-refreshing` class toggle.

| Set | Set when | Cleared when |
| --- | --- | --- |
| `providerRefreshInFlight` | this window itself started a collection for the provider (`refreshSingleProvider`) — covers `UPDATE ALL DATA NOW`/Popover `[update all]`, the scheduled shared-frequency refresh, and cold start | the window's own `get_single_provider_limits` call resolves (success or failure) |
| `providerRemoteRefreshInFlight` | the `provider-refresh-started` backend event arrives for the provider, from a collection started in *this or another* surface | the matching `provider-updated`/`provider-refresh-failed` event arrives for the same provider |
| `providerFlashRefreshing` | a `provider-updated`/`provider-refresh-failed` event applies new data or a new error to a card that was **not** already covered by `providerRemoteRefreshInFlight` (see [The flash](#the-flash)) | a fixed `REMOTE_UPDATE_FLASH_MS` (1800ms — one pass of the glare) timeout, or earlier reconciliation is not needed since the timeout is the only clearing path |

`updateRefreshVisual` is called after every mutation of any of the three sets, so it always reflects the current union rather than being pushed a `true`/`false` value at each call site.

## Cold start

On initial load, a provider with no cached snapshot (`get_cached_provider_limits` returned `null`) is mounted via `createEmptyProvider` (`pending: true`) and immediately handed to `refreshSingleProvider`, which adds it to `providerRefreshInFlight` before awaiting the response. The card therefore plays the glare from the moment it is mounted until its first response, in every surface that has no snapshot yet for that provider — no special-casing needed, this falls out of the same local in-flight tracking every other refresh uses.

## Refresh started in this window

`refreshSingleProvider(providerId)` — called by `UPDATE ALL DATA NOW`/`[update all]` or a scheduled timer firing — adds the id to `providerRefreshInFlight` before the `get_single_provider_limits` call and removes it in `finally`, calling `updateRefreshVisual` both times. This is enough to animate the card in the window that started the refresh; no event is needed for a window to see its own action.

## Refresh started in another surface

The backend emits `provider-refresh-started` (payload `{ id }`) right as an actual collection begins, from `run_collection` in `src-tauri/src/commands/collect.rs` — before the source chain runs, and exactly once per real collection, since `CollectionCoordinator` guarantees a second caller for the same provider joins the in-flight collection instead of starting another one. Every open surface listens (`initProviders` in `providers.js`) and adds the id to `providerRemoteRefreshInFlight`, so a refresh started in one surface animates the matching card in every other open surface too, without that surface starting a collection of its own. The event is forwarded to the Popover the same way `provider-updated` already is (`popover_panel::install_event_forwarding`); the Main Window receives it through Tauri's ordinary event delivery.

Because the event is broadcast app-wide, the surface that started the collection also receives its own `provider-refresh-started` — harmless, since `providerRemoteRefreshInFlight` and `providerRefreshInFlight` are independent sets feeding the same union.

## Applying the result

`provider-updated` (success) and `provider-refresh-failed` (failure) are both listened for by every surface and applied by `applyRemoteProviderUpdate`/`applyRemoteProviderFailure`. Both functions:

1. Delete the provider id from `providerRemoteRefreshInFlight` (recording whether it was actually present) and call `updateRefreshVisual`, so the in-flight animation always clears when the collection ends, in every surface.
2. If `providerRefreshInFlight` still has the id, return — this window's own request is the one that will render the result (its own `try`/`catch` path in `refreshSingleProvider` already does), so rendering here too would race.
3. Otherwise, render the new data/error onto the card (`updateProviderBlockData`, schedule alignment, restart the refresh timer) exactly as before.

## The flash

Step 1 above only clears an animation that was already playing. A surface that never received (or already resolved) a `provider-refresh-started` signal for that provider — most simply, a surface whose events arrived out of the expected order, or any other path that changes a card's content without this window ever seeing it as "in flight" — would otherwise update the card's content with **no** visible feedback at all. To guarantee a card's content never changes silently, `applyRemoteProviderUpdate`/`applyRemoteProviderFailure` check whether the id *was* present in `providerRemoteRefreshInFlight` right before deleting it (`Set.prototype.delete`'s boolean return value): if it was not, `flashRemoteUpdate(providerId)` adds the id to `providerFlashRefreshing`, plays the same `is-refreshing` glare for `REMOTE_UPDATE_FLASH_MS` (1800ms, one full pass of the glare's own animation cycle), and then removes it.

This makes the "did the card visibly change" guarantee independent of whether `provider-refresh-started` delivery is reliable in every case: even if that event is missed or the timing does not line up, the moment the result itself (`provider-updated`/`provider-refresh-failed`) arrives, the card still flashes once so the user can see something happened, on top of the content actually updating.

## Global refresh

`UPDATE ALL DATA NOW` (Main Window) and `[update all]` (Popover) both call `refreshEnabledProviders()`, which calls `refreshSingleProvider` once per enabled provider — each call is an independent collection with its own `provider-refresh-started`/`provider-updated`/`provider-refresh-failed` lifecycle. There is no separate "bulk" animation state: a global refresh animates every affected card because each of its per-provider refreshes does, in both the initiating surface (via `providerRefreshInFlight`) and every other open surface (via `providerRemoteRefreshInFlight`/the flash fallback).
