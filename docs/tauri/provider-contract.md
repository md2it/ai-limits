# Tauri Provider Contract

## Query Fields

`ProviderLimitsQuery` uses camelCase JSON fields:

| Field | Type | Frontend source | Backend effect |
| --- | --- | --- | --- |
| `enabledCodex` | boolean | `appSettings.codex` | includes/excludes Codex source plan |
| `enabledClaude` | boolean | `appSettings.cloud` | includes/excludes Claude source plan |
| `enabledCursor` | boolean | `appSettings.cursor` | includes/excludes Cursor source plan |
| `useCliFallback` | boolean | `appSettings.useCliFallback` | enables wider CLI fallback in `UiSourcePlanOptions` |
| `notificationsEnabled` | boolean | `appSettings.notifications` | allows notification checks for successful reports |

The Rust type has defaults, but the current frontend passes all fields explicitly.

## Provider Response Fields

`ProviderLimits` uses camelCase JSON fields:

| Field | Type | Backend value | Frontend usage |
| --- | --- | --- | --- |
| `id` | string | source plan label, for example `codex` | DOM key, timer key, provider block identity |
| `label` | string | capitalized `id` | provider heading and accessibility labels |
| `sourceId` | string or null | source identifier from structured data, or null on provider error | `{origin label},`; null displays `Unknown` |
| `dataTimestamp` | string or null | formatted `data_as_of`, `"unknown"` when source timestamp is absent, or null on provider error | `as of {timestamp}`; null displays `unknown` |
| `selectedUpdateFrequency` | string | currently always `"5 min"` | used only as fallback when frontend has no saved interval |
| `limits` | array | displayable limit rows | rendered as rows and meters |
| `errorMessage` | string or null | unavailable/no-data message or provider error | controls failed status and message rendering |
| `noFreshData` | boolean | true when access is available but no displayable limit rows exist | selects the no-fresh-data empty state |

`ProviderLimitRow` fields:

| Field | Type | Backend value | Frontend usage |
| --- | --- | --- | --- |
| `label` | string | display window label | row text before percentage |
| `remainingPercentage` | number | normalized remaining percentage | percentage text, meter width, meter color |
| `resetTime` | string or null | formatted reset timestamp | optional `reset {time}` text |

## Status, Source, Time, And Errors

Provider availability is represented through the combination of `limits`, `errorMessage`, and `noFreshData`:

- available with limits: `limits` is non-empty, `errorMessage` is null, `noFreshData` is false.
- no fresh usable data: `limits` is empty, `errorMessage` may contain a backend message, `noFreshData` is true.
- provider/core error: `limits` is empty, `errorMessage` contains the error, `noFreshData` is false.

The source line shows where provider data came from and when it was collected. `sourceId` supplies the origin label; `dataTimestamp` supplies the timestamp text.

Timestamps are represented as strings:

- `dataTimestamp` for the source data time.
- `limits[].resetTime` for a limit reset time.

User-facing timestamp rendering follows the shared rules in [../time-display.md](../time-display.md). The backend formats timestamps with `format_user_timestamp`; the frontend still runs display cleanup/local formatting where possible before rendering.

Errors are represented in two layers:

- command-level `Err(String)`, for example unknown or disabled provider.
- provider-level `errorMessage` inside a successful `ProviderLimits` response.

The current frontend does not display command-level error text. It marks the provider's transient status as `Failed`.
