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
- content area below the header
- three inline rounded squares for providers:
  - Codex
  - Cursor
  - Claude
- settings area with a mock `Notifications` toggle

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

The dropdown is a mock at this stage.

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
