# Tauri UI Controls

## Update Frequency

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
UPDATE NOW
```

This button refreshes only that provider block.

## Settings

The settings button opens a dropdown grouped into behavior, provider visibility, and appearance sections.

The behavior section is first.

It has a source priority three-option segmented control:

- Fast
- Full
- Best

Default value:

- Fast

Source priority behavior:

- Fast uses the `fast_free` source chain from [../get-info/source-chains.md](../get-info/source-chains.md).
- Full uses the `cli_fallback` source chain.
- Best uses the `cli_first` source chain.
- Full and Best may take longer than Fast because they can run provider CLI checks.
- Best usually provides more accurate and current Codex and Claude data because it starts provider CLI checks first.
- Cursor uses its existing Cursor source and is not affected by source priority.

The behavior section includes an information action. It opens the shared source priority modal explaining the Fast, Full, and Best modes, their source chains, the speed/accuracy tradeoff, and the provider scope.

The behavior section has toggles:

- Notifications

The provider visibility section has toggles:

- Cursor
- Cloud
- Codex

The appearance section has toggles:

- Dark theme

Defaults:

- Notifications, Cursor, Cloud, and Codex are on
- Dark theme follows the system theme until the user changes it manually

User experience:

- Notifications controls whether the app sends system limit alerts
- Cursor, Cloud, and Codex control which provider blocks are shown and which providers are included in the next limits request
- Cloud corresponds to Claude
- Changing a toggle saves the choice and hides disabled provider blocks, but does not start a refresh
- Saved choices apply on the next manual refresh or scheduled provider update

Settings storage:

- settings are saved in `localStorage` under `ai-limits-settings`.
- theme preference is saved in `localStorage` under `ai-limits-theme`.
- per-provider update intervals are saved in `localStorage` under `ai-limits-provider-intervals`.
- these saved settings are frontend state; they are not returned by the backend.

Settings request mapping:

| UI setting | Command query field |
| --- | --- |
| Notifications | `notificationsEnabled` |
| Cursor | `enabledCursor` |
| Cloud | `enabledClaude` |
| Codex | `enabledCodex` |
| Source priority | `sourcePriority` |
