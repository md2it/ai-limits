# Tauri

## Choice

Tauri is the planned desktop framework for this project.

The choice was made because Tauri produces lightweight desktop applications. The final app should be small, fast, and available for the three main desktop operating systems: Linux, macOS, and Windows.

Rust also fits this direction well, because Tauri uses Rust for the native application layer.

## Provider Data Contract

The current desktop UI does not depend on one combined provider response for normal refresh behavior.

Tauri commands support fetching one provider at a time, for example Codex, Claude, or Cursor. The frontend starts provider refreshes in parallel and updates each provider block as soon as that provider result is available.

This keeps slow sources from blocking unrelated provider blocks and matches the planned per-provider update frequency controls.

This section documents the factual command/response contract implemented in `src-tauri/src/commands.rs` and used by `frontend/index.html`.

### Commands

#### `get_provider_limits`

Input:

```json
{
  "query": {
    "enabledCodex": true,
    "enabledClaude": true,
    "enabledCursor": true,
    "useCliFallback": false,
    "notificationsEnabled": true
  }
}
```

Rust input type: `ProviderLimitsQuery`.

Response on success:

```json
[
  {
    "id": "codex",
    "label": "Codex",
    "sourceId": "codex-local",
    "dataTimestamp": "Jul 5, 19:28",
    "selectedUpdateFrequency": "5 min",
    "limits": [
      {
        "label": "5h",
        "remainingPercentage": 92.0,
        "resetTime": "20:48"
      }
    ],
    "errorMessage": null,
    "noFreshData": false
  }
]
```

The command returns `Result<Vec<ProviderLimits>, String>`.

Frontend status: currently not called by `frontend/index.html`; normal refresh uses `get_single_provider_limits` per enabled provider.

#### `get_single_provider_limits`

Input:

```json
{
  "providerId": "codex",
  "query": {
    "enabledCodex": true,
    "enabledClaude": true,
    "enabledCursor": true,
    "useCliFallback": false,
    "notificationsEnabled": true
  }
}
```

Rust inputs:

- `provider_id: String`
- `query: ProviderLimitsQuery`

Response on success:

```json
{
  "id": "codex",
  "label": "Codex",
  "sourceId": "codex-local",
  "dataTimestamp": "Jul 5, 19:28",
  "selectedUpdateFrequency": "5 min",
  "limits": [
    {
      "label": "5h",
      "remainingPercentage": 92.0,
      "resetTime": "20:48"
    }
  ],
  "errorMessage": null,
  "noFreshData": false
}
```

The command returns `Result<ProviderLimits, String>`.

If the provider is disabled or unknown for the passed query, the command returns an error string:

```text
Provider '{provider_id}' is disabled or unknown
```

Frontend usage:

- called as `invoke("get_single_provider_limits", { providerId, query: settingsToQuery() })`
- one call is started per enabled provider.
- successful provider responses mount or update one provider block.
- thrown command errors are caught by the frontend and shown only as a transient provider `Failed` status.

#### `open_external_url`

Input:

```json
{
  "url": "https://github.com/md2it/ai-limits/blob/main/docs/setup/claude-cli.md"
}
```

Rust input:

- `url: String`

Response on success:

```json
null
```

The command returns `Result<(), String>`.

Allowed URLs:

- `https://github.com/md2it/ai-limits/blob/main/docs/setup/claude-cli.md`
- `https://github.com/md2it/ai-limits/blob/main/docs/setup/codex-cli.md`

Any other URL returns:

```text
External URL is not allowed
```

Frontend usage:

- called from CLI setup guide buttons in the modal.
- if Tauri is unavailable, the frontend falls back to `window.open`.

### Query Fields

`ProviderLimitsQuery` uses camelCase JSON fields:

| Field | Type | Frontend source | Backend effect |
| --- | --- | --- | --- |
| `enabledCodex` | boolean | `appSettings.codex` | includes/excludes Codex source plan |
| `enabledClaude` | boolean | `appSettings.cloud` | includes/excludes Claude source plan |
| `enabledCursor` | boolean | `appSettings.cursor` | includes/excludes Cursor source plan |
| `useCliFallback` | boolean | `appSettings.useCliFallback` | enables wider CLI fallback in `UiSourcePlanOptions` |
| `notificationsEnabled` | boolean | `appSettings.notifications` | allows notification checks for successful reports |

The Rust type has defaults, but the current frontend passes all fields explicitly.

### Provider Response Fields

`ProviderLimits` uses camelCase JSON fields:

| Field | Type | Backend value | Frontend usage |
| --- | --- | --- | --- |
| `id` | string | source plan label, for example `codex` | DOM key, timer key, provider block identity |
| `label` | string | capitalized `id` | provider heading and accessibility labels |
| `sourceId` | string or null | display label from structured source, or null on provider error | `Source: {sourceId}`; null displays `unknown` |
| `dataTimestamp` | string or null | formatted `data_as_of`, `"unknown"` when source timestamp is absent, or null on provider error | `(as of {timestamp})`; null displays `unknown` |
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

### Status, Source, Time, And Errors

Provider availability is represented through the combination of `limits`, `errorMessage`, and `noFreshData`:

- available with limits: `limits` is non-empty, `errorMessage` is null, `noFreshData` is false.
- no fresh usable data: `limits` is empty, `errorMessage` may contain a backend message, `noFreshData` is true.
- provider/core error: `limits` is empty, `errorMessage` contains the error, `noFreshData` is false.

Source is represented by `sourceId`. The backend already converts structured source information to a display label with `source_label_for_display`.

Timestamps are represented as strings:

- `dataTimestamp` for the source data time.
- `limits[].resetTime` for a limit reset time.

The backend formats timestamps with `format_user_timestamp`; the frontend still runs display cleanup/local formatting where possible before rendering.

Errors are represented in two layers:

- command-level `Err(String)`, for example unknown or disabled provider.
- provider-level `errorMessage` inside a successful `ProviderLimits` response.

The current frontend does not display command-level error text. It marks the provider's transient status as `Failed`.

## Release Scheme

Desktop builds should be produced in GitHub Actions, not only on a local machine.

The release pipeline should build each platform in its native environment:

- macOS build on a macOS runner.
- Windows build on a Windows runner.
- Linux build on a Linux runner.

The first release stage should focus on unsigned builds and downloadable artifacts. Signing and notarization should be added later, after the build pipeline is stable.

## Initial Milestones

1. Add Tauri project structure.
2. Verify local development on macOS.
3. Add GitHub Actions for Linux, macOS, and Windows builds.
4. Publish build artifacts from GitHub Actions.
5. Add GitHub Releases.
6. Add macOS signing and notarization.
7. Add Windows signing when needed.
