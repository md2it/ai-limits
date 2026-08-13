# Tauri UI Frontend State

The current frontend calls:

- `get_single_provider_limits` to run an actual collection for one provider (starts a source-chain walk, or joins one already in flight for that provider — see [Shared Structured Data Cache](#shared-structured-data-cache)).
- `get_cached_provider_limits` to read the shared snapshot for one provider without starting or joining a collection; resolves to `null` if nothing has been collected for that provider yet this session.
- `open_external_url` for allowlisted setup guide links.
- `start_provider_cli_login` only when the user selects the provider Sign in action.

Each provider is requested independently; there is no aggregate limits-fetch command in the desktop IPC contract.

## Shared Structured Data Cache

The application process keeps one current `StructuredSourceInfo` snapshot for each provider, held in `StructuredInfoCache` (`src-tauri/src/commands/structured_cache.rs`). The snapshot is written only after the provider source chain has selected its result and applied account-field backfill.

Main Window and Menu Bar Popover read the same snapshot. Each surface independently projects the unchanged structured data into its own provider-card state and keeps only presentation state locally.

The shared cache contains no raw provider output, stderr, UI-specific `ProviderLimits` model, rendered markup, or cached transport failures. A failed collection does not replace the last successful structured snapshot.

`collected_at` and `data_as_of` remain fields of the unchanged structured snapshot. `collected_at` is the time of the actual collection and is the common basis for update scheduling; `data_as_of` remains the freshness timestamp of the provider data and is displayed to the user according to the presentation rules. `collected_at` also reaches the frontend as `ProviderLimits.collectedAt` (raw ISO-8601, unformatted — unlike `dataTimestamp`, which arrives pre-formatted for display) — see [Provider Fields Used](#provider-fields-used) and [refresh.md](refresh.md#shared-refresh-schedule).

Concurrent requests for the same provider must share one collection operation: `CollectionCoordinator` (same file) tracks in-flight collections per provider id, and a second `get_single_provider_limits` call for a provider already being collected joins that call's result instead of starting a second one. This is also what makes the events below fire exactly once per real collection, regardless of how many surfaces requested it.

Three Tauri app events cover the full lifecycle of one actual collection, all emitted from `run_collection`/`commands/collect.rs` and all forwarded to the Popover the same way (`popover_panel::install_event_forwarding`, same mechanism already used for `settings-changed`/`theme-changed` — see [mac-popover.md](../mac-popover.md#cross-window-sync)):

- `provider-refresh-started` (payload `{ id }`) — emitted right as the collection begins, before the source chain runs. Every open surface listens (`providers.js`) and marks that provider's card as busy (`is-refreshing`, the same class a surface's own manual/scheduled refresh sets), so a refresh started in one surface animates the same card in the other without starting a second collection there.
- `provider-updated` (payload: the same `ProviderLimits` shape a direct response carries) — emitted after a successful collection, once notifications are evaluated from the selected structured snapshot and the shared cache is updated. Every open surface listens and updates its own mounted card from the payload, without starting a collection of its own, and clears the busy state set by `provider-refresh-started`.
- `provider-refresh-failed` (payload: the same `ProviderLimits` shape, built via `provider_error` — `errorMessage` set, `limits` empty) — emitted after a failed collection. A failed collection still updates neither the shared cache nor emits `provider-updated`; this event is what lets a surface that did not start the collection show the same error state instead of sitting on stale data with no explanation, and also clears the busy state.

A surface that itself called `get_single_provider_limits` for the provider in question ignores `provider-updated`/`provider-refresh-failed` for that id (its own request's resolution already renders the result — rendering it twice would race), but still clears the busy state from either event, since both are emitted app-wide including back to the surface that started the collection.

The full cross-window animation lifecycle built on these three events — including the short flash played when a card's content changes without a `provider-refresh-started` signal ever reaching that surface — is documented separately in [refresh-animation.md](refresh-animation.md).

On initializing its provider list (first load, or a provider newly re-enabled), a surface calls `get_cached_provider_limits` for each enabled provider before deciding whether to collect: a provider with an existing shared snapshot renders it immediately and only re-collects if the shared update-frequency schedule (recomputed from that snapshot's `collectedAt`) says a refresh is already due; a provider with no shared snapshot yet collects normally. This is what lets a surface opened after the other has already collected data show it immediately, with no collection of its own.

User-facing problem and recovery rules are documented in [problems.md](problems.md).

## Provider Fields Used

| Backend field | Frontend usage |
| --- | --- |
| `id` | provider block identity, DOM `data-provider-id`, timer maps |
| `label` | provider heading, accessibility labels |
| `limits` | rendered limit rows; empty array selects empty/error state |
| `limits[].label` | row label before `% left` |
| `limits[].remainingPercentage` | displayed percent, bar width, bar color |
| `limits[].resetTime` | optional reset line |
| `plan` | Plan section content, always an object `{ lines: string[], links: { label: string, url: string }[] }`, never `null`. `lines` are ready-to-render text lines built by the backend from `account.plan`, `account.subscription_started_at`, `account.renewal_at`, `account.price_amount`, `account.price_currency`, and `account.price_note`; the frontend renders them verbatim in order. `links` carries at most `Manage plan` (from `account.plan_management_url`) and `Manage billing` (from `account.billing_management_url`); missing URLs are omitted by the backend. When both `lines` and `links` are empty, the section has no heading and no lines; if Show plan is on, the card still reserves the equalized slot — see [provider-blocks.md](provider-blocks.md#section-slot-alignment) |
| `sourceId` | origin label in `{label},`; possible values: `Local files`, `CLI`, `API2`, `Unknown` |
| `dataTimestamp` | `as of {timestamp}`; missing value displays `unknown` |
| `collectedAt` | raw ISO-8601 instant of the actual collection, not displayed; seeds the shared refresh schedule — see [refresh.md](refresh.md#shared-refresh-schedule) |
| `errorMessage` | marks refresh as failed and supplies fallback message outside no-fresh-data and CLI authorization states |
| `noFreshData` | renders the no-fresh-data empty state with a link to data-availability help |
| `authorizationRequired` | when `codex` or `claude`, renders the CLI authorization problem with Sign in and the manual login command |

## Frontend-Only Fields And State

These values are not returned by the backend:

- `pending`, added by `createEmptyProvider` before the first response.
- `appSettings.notifications`.
- `appSettings.cursor`.
- `appSettings.cloud`.
- `appSettings.codex`.
- `appSettings.showLimits`.
- `appSettings.showPlan`.
- `appTheme`, persisted separately from app settings.
- the shared update-frequency setting.
- provider refresh timers.
- provider refresh in-flight markers.
- which top-level page (Overview/Settings/Help) is current, tracked by `switchView()` in `main.js`.
- the selected help chapter.

`appSettings.showLimits` and `appSettings.showPlan` are `true` by default. They toggle the Limits and Plan sections of every provider block in place, without triggering a refresh; see [settings.md](settings.md#display) and [provider-blocks.md](provider-blocks.md).
