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
  - trigger when 25% of the limit is spent
  - color, if supported by the operating system: green
  - text: `75% of your limit remains`
- 50% remaining:
  - trigger when 50% of the limit is spent
  - color, if supported by the operating system: yellow
  - text: `50% of your limit remains`
- 25% remaining:
  - trigger when 75% of the limit is spent
  - color, if supported by the operating system: orange
  - text: `25% of your limit remains`
- 10% remaining:
  - trigger when 90% of the limit is spent
  - color, if supported by the operating system: red
  - text: `10% of your limit remains`

Rules:

- notifications do not replace each other; every notification is kept as a separate system notification
- notifications are independent for each provider, for example Codex, Claude, and Cursor
- notifications are also independent for each called data source
- if different data sources return different limit data for the same provider, this is acceptable
- each called and enabled source is evaluated separately and can produce its own notification

---

## Calculation

Notification triggers are calculated from structured data.

Structured data is used because it is standardized and easier to process consistently across providers and sources.

---

## Platform Scope

Development targets:

- macOS
- Windows
- Linux

Initial development will be checked directly on macOS.

Windows and Linux are developed theoretically for now. They must be tested later by external testers who have access to those systems.
