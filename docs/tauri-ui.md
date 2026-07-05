# Tauri UI

## Goal

The first Tauri UI is a mock interface map.

It should show where future provider data, refresh settings, and notification settings will be connected. It does not need polished visual design.

## Scope

This stage uses mock data only.

Do not connect:

- real providers
- `get_limits.rs`
- notifications logic
- GitHub Actions or release logic

## Layout

The UI contains:

- header with the centered application name
- toolbar with last updated time, settings gear, and Refresh
- three inline rounded squares for providers:
  - Codex
  - Cursor
  - Claude

Each provider square represents one provider and contains that provider's limit details.

## Provider Square

Each provider square contains:

- provider name
- limit rows
- source line
- update frequency dropdown at the bottom

The provider content should roughly match the current terminal output model.

Example data shape:

```text
     --------- CURSOR --------
plan ■■■■■■■■■■■■■■□□□□□□□□□□□  54.6% left | reset Jul 28, 03:00 UTC+3
auto ■■■■■■■■■■■■■■■■□□□□□□□□□  63.7% left
api  ■■■■■■□□□□□□□□□□□□□□□□□□□  24.5% left
Source cursor-api2: Jul  5, 19:28 UTC+3

     --------- CODEX ---------
5h   ■■■■■■■■■■■■■■■■■■■■■■■□□  92.0% left | reset Jul  5, 20:48 UTC+3
7d   ■■■■■■■■■□□□□□□□□□□□□□□□□  35.0% left | reset Jul 10, 03:55 UTC+3
Source codex-local: Jul  5, 19:28 UTC+3

     --------- CLAUDE --------
5h   ■■■■■■■■■■■■■■■■■■■■■■■■■ 100.0% left | reset Jul  6, 00:20 UTC+3
7d   ■■■■■■■■■■■■■■■■■■■■■□□□□  84.0% left | reset Jul  7, 13:00 UTC+3
Source claude-cli: Jul  5, 19:29 UTC+3
```

The UI does not need to use terminal-style ASCII rendering. The example defines the information that must be visible.

## Update Frequency

Each provider square has a dropdown at the bottom.

Options:

- Manual only
- 1 min
- 5 min
- 10 min
- 30 min
- 1 hour

Default value:

- 5 min

## Refresh Behavior

Provider blocks should render immediately when the UI opens. Empty data is acceptable while a provider has not returned data yet.

Each provider block refreshes independently:

- initial load starts refreshes for enabled providers in parallel
- manual Refresh starts refreshes for enabled providers in parallel
- scheduled refresh runs only for the provider whose interval fired
- a slow or failed provider must not block other provider blocks from updating
- each block owns its own loading, updated, and failed status
- global loading should not hide or block provider blocks

The preferred integration model is one Tauri request per provider. The frontend should not call a combined all-provider request and then wait for the slowest provider before updating the screen.

## Settings

The gear icon in the toolbar opens a dropdown with toggles:

- Notifications
- Cursor
- Cloud
- Codex
- Use CLI fallback

Defaults:

- Notifications, Cursor, Cloud, and Codex are on
- Use CLI fallback is off

User experience:

- Notifications controls whether the app sends system limit alerts
- Cursor, Cloud, and Codex control which provider blocks are shown and which providers are included in the next limits request
- Cloud corresponds to Claude
- Use CLI fallback controls whether wider source fallback is used when fetching limits, like terminal `ai-limits --best`
- Changing a toggle saves the choice and hides disabled provider blocks, but does not start a refresh
- Saved choices apply on the next manual refresh or scheduled provider update

## Mock Requirements

Mock data should be structured near the UI code and named clearly, so future developers can replace it with a Tauri command response.

The mock should include:

- provider id
- provider label
- limit rows
- remaining percentage
- optional reset time
- source id
- data timestamp
- selected update frequency

## Boundaries

- UI mock must not duplicate provider-fetching logic.
- UI mock must not decide real limit semantics.
- UI mock must not call provider commands.
- UI mock must not implement real notification behavior.
- Future integration should replace mock data with structured data from the Rust core through Tauri commands.
