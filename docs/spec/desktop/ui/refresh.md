# Tauri UI Refresh

Provider blocks should render immediately when the UI opens. Empty data is acceptable while a provider has not returned data yet.

Each provider block refreshes independently:

- initial load starts refreshes for enabled providers in parallel
- `UPDATE ALL DATA NOW` starts refreshes for enabled providers in parallel
- `UPDATE ALL DATA NOW` refreshes every enabled provider
- scheduled refresh uses one user-selected interval for every enabled provider
- a slow or failed provider must not block other provider blocks from updating
- global loading should not hide or block provider blocks

The preferred integration model is one Tauri request per provider. The frontend should not call a combined all-provider request and then wait for the slowest provider before updating the screen.

## Shared Refresh Schedule

Main Window and Menu Bar Popover each run their own refresh timers, but both use the same user-selected update interval for every enabled provider. The next-refresh target for a provider is its last actual collection instant (`ProviderLimits.collectedAt`, the backend's `collected_at`) plus that shared interval — never a per-window "when did I last observe a fetch resolve" clock. A collection started by either surface or by `UPDATE ALL DATA NOW` updates both surfaces' schedules once its result is applied.

A surface applies `collectedAt` to its schedule from four places:

- its own `get_single_provider_limits` response, on both success and (implicitly, via the existing retry-anchor behavior) failure — a failed collection has no `collectedAt` and anchors the retry to the attempt's own clock instead, same as before.
- `get_cached_provider_limits`, read once per enabled provider when a surface initializes its provider list — see [frontend-state.md](frontend-state.md#shared-structured-data-cache).
- the `provider-updated` event, emitted after any surface's successful collection — see [frontend-state.md](frontend-state.md#shared-structured-data-cache).
- the `provider-refresh-failed` event, emitted after any surface's failed collection — a surface that did not itself request the collection anchors its retry the same way a failed request of its own would, rather than leaving its schedule stale — see [frontend-state.md](frontend-state.md#shared-structured-data-cache).

The card animation that accompanies a refresh (the "is-refreshing" glare) follows its own cross-window lifecycle on top of this schedule — see [refresh-animation.md](refresh-animation.md).

## Boundaries

- UI must not duplicate provider-fetching logic.
- UI must not decide real limit semantics.
- Future integration should use structured data from the Rust core through Tauri commands.
