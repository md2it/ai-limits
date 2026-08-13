# Tauri UI Settings

## Settings

The Settings page, reached from the top nav, is grouped into provider visibility, display, and other sections. The desktop application always uses the CLI-first source chain; source priority is not a user setting.

The provider visibility section has toggles:

- Cursor
- Cloud
- Codex

## Display

The display section has toggles:

- Show limits
- Show plan
- Show source
- Show update time

These apply to every provider block at once; there is no per-provider override. Show limits and Show plan control the matching sections defined in [provider-blocks.md](provider-blocks.md) and [provider-block-content.md](provider-block-content.md). Show source and Show update time control the two parts of the source line.

Defaults:

- Show limits, Show plan, and Show update time are on; Show source is off.

User experience:

- Turning a display toggle off hides the matching content in every provider block immediately, without waiting for a refresh.
- Turning a display toggle back on immediately shows the matching content again using the data already held for each provider, without triggering a refresh.
- This differs from the provider visibility toggles below, whose effect on the next limits request takes effect on the next refresh; display toggles never change what data is requested, only what is rendered from data already on hand.

## Other

The other section has:

- Update frequency (dropdown)
- Notifications
- Automatic updates
- Dark theme

Update frequency is one shared setting for provider-limit data. It affects every enabled provider card; there are no per-provider refresh-frequency settings. Its default value is `10 min`. Changing it saves the choice and applies it immediately to the shared refresh schedule. It does not itself start a data refresh.

Defaults:

- Notifications, Cursor, Cloud, and Codex are on
- Automatic updates are on
- Dark theme follows the system theme until the user changes it manually

User experience:

- Notifications controls whether the app sends system limit alerts for every notification type in [../../notifications/content.md](../../notifications/content.md), including low remaining and 100% again
- Automatic updates controls application-version checks and downloads; it does not control provider-limit refreshes
- Cursor, Cloud, and Codex control which provider blocks are shown and which providers are included in the next limits request
- Cloud corresponds to Claude
- Changing a toggle saves the choice and hides disabled provider blocks, but does not start a refresh
- The update-frequency setting applies to every enabled provider; changing it does not start a refresh

Settings storage:

- settings are saved in `localStorage` under `ai-limits-settings`.
- theme preference is saved in `localStorage` under `ai-limits-theme`.
- these saved settings are frontend state; they are not returned by the backend.
- Update frequency, Show limits, Show plan, Show source, and Show update time are saved in `ai-limits-settings` alongside the other settings; they are not sent to the backend as part of a limits request.

Settings request mapping:

| UI setting | Command query field |
| --- | --- |
| Notifications | `notificationsEnabled` |
| Cursor | `enabledCursor` |
| Cloud | `enabledClaude` |
| Codex | `enabledCodex` |

Display settings have no command query field: they never affect what is requested from the backend, only what the frontend renders from the response it already has.
