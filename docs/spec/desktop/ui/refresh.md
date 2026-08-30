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

The application process runs one background refresh scheduler shared by every surface. The frontend remains the owner of the saved update-frequency and provider settings and sends the current configuration through `configure_background_refresh`; the scheduler owns all periodic wakeups so collection continues when Main Window and Menu Bar Popover are both hidden. The next-refresh target for a provider is its last actual collection instant plus the shared interval, never a per-window "when did I last observe a fetch resolve" clock. A collection started by the scheduler, either surface, or `UPDATE ALL DATA NOW` resets that provider's schedule, and `CollectionCoordinator` merges concurrent requests for the same provider into one actual collection.

Frontend surfaces keep no periodic refresh timers. They project the next-refresh target for display from `ProviderLimits.collectedAt` and the shared interval, then update that projection whenever an own response or a cross-surface event reports another attempt.

A surface applies `collectedAt` to its schedule from four places:

- its own `get_single_provider_limits` response, on both success and (implicitly, via the existing retry-anchor behavior) failure — a failed collection has no `collectedAt` and anchors the retry to the attempt's own clock instead, same as before.
- `get_cached_provider_limits`, read once per enabled provider when a surface initializes its provider list — see [frontend-state.md](frontend-state.md#shared-structured-data-cache).
- the `provider-updated` event, emitted after any surface's successful collection — see [frontend-state.md](frontend-state.md#shared-structured-data-cache).
- the `provider-refresh-failed` event, emitted after any surface's failed collection — a surface that did not itself request the collection anchors its retry the same way a failed request of its own would, rather than leaving its schedule stale — see [frontend-state.md](frontend-state.md#shared-structured-data-cache).

The card animation that accompanies a refresh (the "is-refreshing" glare) follows its own cross-window lifecycle on top of this schedule — see [refresh-animation.md](refresh-animation.md).

Selecting `Manual only` clears all native scheduler deadlines but does not suppress the normal initial collection when the application starts or a provider is newly enabled.

## Boundaries

- UI must not duplicate provider-fetching logic.
- UI must not decide real limit semantics.
- Future integration should use structured data from the Rust core through Tauri commands.
