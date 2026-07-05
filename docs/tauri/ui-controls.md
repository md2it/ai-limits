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
UPD MANUALLY
```

This button refreshes only that provider block.

## Settings

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

Settings storage:

- settings are saved in `localStorage` under `ai-limits-settings`.
- per-provider update intervals are saved in `localStorage` under `ai-limits-provider-intervals`.
- these saved settings are frontend state; they are not returned by the backend.

Settings request mapping:

| UI setting | Command query field |
| --- | --- |
| Notifications | `notificationsEnabled` |
| Cursor | `enabledCursor` |
| Cloud | `enabledClaude` |
| Codex | `enabledCodex` |
| Use CLI too | `useCliFallback` |
