# Notifications

This document defines the initial requirements for system notifications.

---

## Goal

The application should be able to notify the user when an important event happens.

Notifications are a shared application capability. They are not tied only to a future desktop interface.

---

## Architecture

The application uses one common notification tool.

Application logic calls a single interface, for example `notify(title, message)`.

Platform-specific behavior is hidden behind three adapters:

- macOS adapter
- Windows adapter
- Linux adapter

The target source structure is:

```text
src/
  notifications/
    mod.rs
    macos.rs
    windows.rs
    linux.rs
    noop.rs
```

`noop.rs` is the fallback for unsupported or unavailable notification environments.

---

## UX/UI

Notifications are system notifications. They should use the native notification UI of the current operating system.

Initial notification types are hardcoded:

- 75% remaining:
  - trigger when 25% or more of the limit is spent
  - color, if supported by the operating system: green
- 50% remaining:
  - trigger when 50% or more of the limit is spent
  - color, if supported by the operating system: yellow
- 25% remaining:
  - trigger when 75% or more of the limit is spent
  - color, if supported by the operating system: orange
- 10% remaining:
  - trigger when 90% or more of the limit is spent
  - color, if supported by the operating system: red

Notification text template:

```text
$EMOJI AI Limits
$PROVIDER_NAME $TYPE - NN% limit remains
reset yyyy-mm-dd hh:mm UTC(+/-N)
```

On platforms that support notification title/subtitle/body, `$EMOJI AI Limits` is the title, the limit line is the subtitle, and the reset line is the body.

Fields:

- `EMOJI`:
  - `🟢` for 75-50% remaining
  - `🟡` for 50-25% remaining
  - `🟠` for 25-10% remaining
  - `🔴` for less than 10% remaining
- `PROVIDER_NAME`: `Codex`, `Claude`, or `Cursor`
- `TYPE`: `5h`, `weekly`, `auto`, `plan`, or `api`

Examples:

```text
🟡 AI Limits
Codex weekly - 44% left
reset 2026-07-07 22:22 UTC+3
```

```text
🟢 AI Limits
Cursor auto - 65% left
reset 2026-07-07 22:22 UTC
```

```text
🔴 AI Limits
Claude 5h - 7% left
reset 2026-07-07 22:22 UTC-6
```

Rules:

- notifications do not replace each other; every notification is kept as a separate system notification
- the same running process should not repeatedly send the same notification
- notifications are independent for each provider, for example Codex, Claude, and Cursor
- notifications are also independent for each called data source
- if different data sources return different limit data for the same provider, this is acceptable
- each called and enabled source is evaluated separately and can produce its own notification

---

## Calculation

Notification triggers are calculated from structured data.

Structured data is used because it is standardized and easier to process consistently across providers and sources.

---

## Testing

Manual testing can use a fake notification trigger without provider data:

```text
ai-limits --test-notification=75
ai-limits --test-notification=50
ai-limits --test-notification=25
ai-limits --test-notification=10
```

This sends a real system notification through the current platform adapter.

Trigger calculation is covered with unit tests using fake structured data.

---

## Platform Scope

Development targets:

- macOS
- Windows
- Linux

Initial development will be checked directly on macOS.

Windows and Linux are developed theoretically for now. They must be tested later by external testers who have access to those systems.
