# Notifications

This document defines the target behavior for system notifications.

---

## Goal

The application notifies the user when an important limit event requires attention.

Notifications are a shared product capability. They are used by the desktop interface and can be requested from the terminal interface.

---

## Architecture

The application uses one common notification domain model.

Provider and limit logic produce notification candidates from structured source data. The notification rules stay in the shared Rust core and must not be duplicated in the Tauri frontend.

Notification delivery is separate from notification logic.

The target delivery adapter is Tauri notifications. Platform-specific notification behavior for macOS, Windows, and Linux is delegated to Tauri unless a supported platform later requires product behavior that Tauri cannot provide.

```text
shared core
  structured source data
  notification thresholds
  notification text
  dedupe keys

delivery adapter
  Tauri notifications
  system notification permission
  native system notification delivery
  application icon
  notification click behavior

terminal interface
  existing terminal UI
  optional request to the installed/running Tauri application
```

The shared core must not depend on Tauri or any operating-system notification API. It produces notification candidates and passes them to the delivery layer.

The application does not maintain separate first-party macOS, Windows, and Linux notification adapters in the current target architecture. If Tauri notifications later cannot provide required product behavior on a supported platform, platform-specific delivery adapters may be introduced behind the same delivery interface.

The terminal interface does not send native operating-system notifications directly. When a terminal run needs a system notification, it requests delivery from the installed and available Tauri application.

If the terminal interface cannot hand the notification request to Tauri, it silently skips the system notification. It must not print an additional terminal message, because the terminal UI already contains the relevant information and extra text would not attract attention.

There is no separate macOS helper/notifier in the target architecture.

---

## Delivery Rules

System notifications are delivered through the Tauri notifications adapter.

Rules:

- when the Tauri application is active or minimized, eligible notifications should be delivered as native system notifications through Tauri
- when the terminal interface can reach the installed and available Tauri application, eligible notifications can be delivered through Tauri
- when Tauri is unavailable, the notification is skipped without additional terminal output
- the application does not use a separate notification helper process

The user-facing notification setting controls whether notification checks are enabled.

---

## Click Behavior

Clicking a system notification opens or focuses the Tauri application.

Notification clicks do not need to navigate to a specific provider, limit, event, or screen in the current target behavior.

---

## Branding

System notifications should appear as notifications from the application, not from a script runner, terminal, shell, or development tool.

The notification should use the application identity configured in the Tauri bundle, including:

- application name
- bundle identifier
- application icon

The same application icon is used for notifications. Separate icons per notification type are not required.

---

## UX/UI

Notifications are native system notifications. They should use the current operating system's standard notification UI through Tauri notifications.

The application does not provide an in-app notification center in the current target behavior.

The application does not provide separate in-app toast notifications in the current target behavior.

The application does not keep a notification history in the current target behavior.

Notification actions and buttons are not required.

---

## Notification Types

Notification types are based on remaining limit thresholds:

- 75% remaining:
  - trigger when 25% or more of the limit is spent
  - color, if supported by the notification transport: green
- 50% remaining:
  - trigger when 50% or more of the limit is spent
  - color, if supported by the notification transport: yellow
- 25% remaining:
  - trigger when 75% or more of the limit is spent
  - color, if supported by the notification transport: orange
- 10% remaining:
  - trigger when 90% or more of the limit is spent
  - color, if supported by the notification transport: red

Colors are optional because system notification customization is platform-dependent.

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

---

## Deduplication

Rules:

- notifications do not replace each other; every delivered notification is kept as a separate system notification
- the same running process should not repeatedly send the same notification
- notifications are independent for each provider, for example Codex, Claude, and Cursor
- notifications are independent for each called data source
- if different data sources return different limit data for the same provider, this is acceptable
- each called and enabled source is evaluated separately and can produce its own notification candidate

---

## Calculation

Notification triggers are calculated from structured data.

Structured data is used because it is standardized and easier to process consistently across providers and sources.

Notification calculation is independent from the delivery channel. The same candidate generation rules apply whether the request originates from the Tauri UI or from the terminal interface.

---

## Testing

Manual testing can use fake notification triggers without provider data:

```text
ai-limits --test-notification=75
ai-limits --test-notification=50
ai-limits --test-notification=25
ai-limits --test-notification=10
```

These commands should request delivery through the Tauri notifications adapter.

When the Tauri application is unavailable, test notification commands should complete without sending a system notification and without printing an extra notification message.

Trigger calculation is covered with unit tests using fake structured data.

---

## Platform Scope

Development targets:

- macOS
- Windows
- Linux

The target delivery adapter for every supported desktop platform is Tauri notifications.

Initial development is checked directly on macOS. Windows and Linux behavior must be tested later by external testers who have access to those systems.
