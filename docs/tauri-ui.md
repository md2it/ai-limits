# Tauri UI

## Table Of Contents

- [Purpose](#purpose)
- [Window Layout](#window-layout)
  - [Main Content](#main-content)
  - [Bottom Actions](#bottom-actions)
- [Provider Blocks](#provider-blocks)
  - [Block Structure](#block-structure)
  - [Limit Rows](#limit-rows)
  - [Source Line](#source-line)
  - [No Fresh Data State](#no-fresh-data-state)
- [Controls](#controls)
  - [Update Frequency](#update-frequency)
  - [Settings](#settings)
- [Time Display](#time-display)
- [Refresh Behavior](#refresh-behavior)
- [Boundaries](#boundaries)
- [Implementation Prompts](#implementation-prompts)
  - [Prompt 1: Window Layout And Global Controls](#prompt-1-window-layout-and-global-controls)
  - [Prompt 2: Provider Limit Rows](#prompt-2-provider-limit-rows)
  - [Prompt 3: Local Time Formatting](#prompt-3-local-time-formatting)
  - [Prompt 4: Provider Source And Per-Block Update](#prompt-4-provider-source-and-per-block-update)

## Purpose

The Tauri UI shows current provider limits, refresh controls, settings, source metadata, and states where provider data is unavailable.

The UI should remain compact and operational rather than marketing-oriented.

## Window Layout

### Main Content

The UI contains:

- provider blocks as the main content
- bottom action row with global manual update and settings
- centered last update line below the bottom action row
- three inline rounded squares for providers:
  - Codex
  - Cursor
  - Claude

Each provider square represents one provider and contains that provider's limit details.

The window must not show a visible `AI Limits` title in the content area.

### Bottom Actions

Global update controls live at the bottom of the window, below provider blocks.

The bottom action row contains:

- `UPDATE ALL MANUALLY` button
- settings button

The `UPDATE ALL MANUALLY` button takes all available row width except the settings button area. Its label is centered.

The settings button is a small square button on the right side of the same row.

The last update text is shown under the bottom action row, centered. It uses the same information that was previously shown in the top toolbar.

The settings dropdown opens upward from the bottom settings button. It must remain visible and usable across supported window sizes, including narrow and short windows.

## Provider Blocks

### Block Structure

Each provider square contains:

- provider name
- limit rows
- source information split across two lines
- update frequency dropdown near the bottom
- provider-specific manual update button at the bottom

The provider content should roughly match the current terminal output model.

Example data shape:

```text
     --------- CURSOR --------
plan | 54.6% left
■■■■■■■■■■■■□□□
reset Jul 28, 03:00
auto | 63.7% left
■■■■■■■■■■■■■■□□□□
api | 24.5% left
■■■■■□□□□□□□□□□□□□□□
Source cursor-api2
Jul 5, 19:28

     --------- CODEX ---------
5h | 92.0% left
■■■■■■■■■■■■■■■■■■■■■■■□□
reset 20:48
7d | 35.0% left
■■■■■■■■■□□□□□□□□□□□□□□□□
reset Jul 10, 03:55
Source codex-local
Jul 5, 19:28

     --------- CLAUDE --------
5h | 100.0% left
■■■■■■■■■■■■■■■■■■■■■■■■■
reset Jul 6, 00:20
7d | 84.0% left
■■■■■■■■■■■■■■■■■■■■□□□□
reset Jul 7, 13:00
Source claude-cli
Jul 5, 19:29
```

The UI does not need to use terminal-style ASCII rendering. The example defines the information that must be visible.

### Limit Rows

Each limit row is rendered as a vertical group:

1. Top text line above the bar: `{window} | {remaining}% left`, for example `5h | 59.0% left`.
2. Full-width remaining bar.
3. Reset text line below the bar: `reset {time}`, for example `reset Jul 6, 01:49`.

The limit type, such as `5h`, `7d`, `plan`, `auto`, or `api`, must not consume a separate left column. This lets every bar use 100% of the provider block content width.

The remaining bar shows:

- filled segment width equal to remaining percentage
- unfilled spent segment in white or another very light neutral color
- one solid color for the whole filled segment

The filled segment color is calculated from remaining percentage:

- `100%` is green
- `50%` is yellow
- `1%` is red
- intermediate values are interpolated between these anchors

The bar must not use a left-to-right rainbow gradient inside the filled segment. For example, if `10%` remains, the filled 10% segment is a near-red color and the spent 90% segment stays light.

### Source Line

Provider source information is split into two visual lines:

```text
Source codex-local
Jul 5, 22:12
```

Both values are variable data from the application core:

- source id, for example `codex-local`
- data timestamp

### No Fresh Data State

If checked sources return no fresh usable limit records, the provider block must show an empty state instead of a technical error like `No usable limit records from this source`.

Short copy:

```text
No fresh usage data. Try this:
```

Show the existing CLI fallback toggle directly in this state, labeled:

```text
Use CLI too
```

Below the toggle, show a text button:

```text
More details
```

The button opens a modal instead of navigating away.

Modal copy:

```text
CLI data source

CLI fallback is off by default because it can take more time.

If you mostly use Claude or Codex through CLI, this source is usually the most relevant one.

If you do not use CLI but still have trouble getting current limit data, enabling CLI fallback can still help. It is used only as a fallback for providers that do not return data through faster sources.

Provider refreshes run asynchronously, so one slower provider should not block the others.

Setup guides:
Claude guide on GitHub
Codex guide on GitHub
```

The setup links must open externally from the Tauri app:

- Claude guide on GitHub: <https://github.com/md2it/ai-limits/blob/main/docs/setup/claude-cli.md>
- Codex guide on GitHub: <https://github.com/md2it/ai-limits/blob/main/docs/setup/codex-cli.md>

Optional modal helper copy:

```text
You can give this guide to your agent to set it up.
```

## Controls

### Update Frequency

Each provider square has a dropdown at the bottom.

The label is `Upd&nbsp;every`, using a non-breaking space between the words.

Options:

- Manual only
- 1 min
- 5 min
- 10 min
- 30 min
- 1 hour

Default value:

- 5 min

Under the update frequency label and field, each provider block has a button:

```text
UPD MANUALLY
```

This button refreshes only that provider block.

### Settings

The settings button opens a dropdown with toggles:

- Notifications
- Cursor
- Cloud
- Codex
- Use CLI too

Defaults:

- Notifications, Cursor, Cloud, and Codex are on
- Use CLI too is off

User experience:

- Notifications controls whether the app sends system limit alerts
- Cursor, Cloud, and Codex control which provider blocks are shown and which providers are included in the next limits request
- Cloud corresponds to Claude
- Use CLI too controls whether wider source fallback is used when fetching limits, like terminal `ai-limits --best`
- Use CLI too toggles shown in settings and provider empty states must stay visually synchronized
- Changing a toggle saves the choice and hides disabled provider blocks, but does not start a refresh
- Saved choices apply on the next manual refresh or scheduled provider update

## Time Display

All user-facing times in the Tauri UI are displayed in the local time zone of the user's device.

The application must convert timestamps from the core data into device-local time before rendering them.

For today, show only time:

```text
20:48
```

For another date, show date and time:

```text
Jul 6, 01:49
```

Do not show `UTC+3` or another timezone suffix in the UI.

## Refresh Behavior

Provider blocks should render immediately when the UI opens. Empty data is acceptable while a provider has not returned data yet.

Each provider block refreshes independently:

- initial load starts refreshes for enabled providers in parallel
- `UPDATE ALL MANUALLY` starts refreshes for enabled providers in parallel
- `UPD MANUALLY` in one provider block refreshes only that provider
- scheduled refresh runs only for the provider whose interval fired
- a slow or failed provider must not block other provider blocks from updating
- each block owns its own loading, updated, and failed status
- global loading should not hide or block provider blocks

The preferred integration model is one Tauri request per provider. The frontend should not call a combined all-provider request and then wait for the slowest provider before updating the screen.

## Boundaries

- UI must not duplicate provider-fetching logic.
- UI must not decide real limit semantics.
- Future integration should use structured data from the Rust core through Tauri commands.

## Implementation Prompts

Use these prompts sequentially. Each prompt is intentionally small and points to this document as the source of truth.

### Prompt 1: Window Layout And Global Controls

```text
Update the Tauri frontend according to docs/tauri-ui.md.

Scope:
- Remove the visible AI Limits title from the content area.
- Move the global refresh controls to the bottom.
- Rename Refresh to UPDATE ALL MANUALLY and make it fill the row except the settings square.
- Move settings to the right side of the same bottom row.
- Show the centered Last updated line below this row.
- Make the settings dropdown open upward and stay visible at supported window sizes.

Do not change provider data fetching semantics beyond wiring the existing global refresh button in its new location.
```

### Prompt 2: Provider Limit Rows

```text
Update provider limit rendering according to docs/tauri-ui.md.

Scope:
- Render each limit as: top text, full-width bar, reset text.
- Top text format: 5h | 59.0% left.
- Reset text format: reset Jul 6, 01:49, or only reset 01:49 when the reset is today.
- Remove the separate left column for limit type so bars use 100% provider content width.
- Color the filled bar segment by remaining percent: red near 1%, yellow at 50%, green at 100%.
- Keep the spent part white or very light.

Use one solid color for the filled segment, not a rainbow gradient across the bar.
```

### Prompt 3: Local Time Formatting

```text
Update frontend time formatting according to docs/tauri-ui.md.

Scope:
- Display all user-facing timestamps in the device local timezone.
- Convert timestamps from core data before rendering.
- If the date is today, show only HH:MM.
- If the date is not today, show MMM D, HH:MM.
- Remove UTC offset suffixes from source timestamps, reset timestamps, and Last updated.

Keep parsing tolerant of current frontend and core timestamp shapes.
```

### Prompt 4: Provider Source And Per-Block Update

```text
Update provider block controls according to docs/tauri-ui.md.

Scope:
- Split source display into two lines: Source {sourceId} and formatted data timestamp.
- Rename Update frequency to Upd&nbsp;every using a non-breaking space.
- Add UPD MANUALLY under the update frequency row.
- Wire UPD MANUALLY to refresh only that provider.

Keep existing independent provider loading, updated, and failed states.
```
