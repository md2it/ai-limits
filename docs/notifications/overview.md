# Notifications

This document defines the target behavior for system notifications.

Notification content, branding, and UI presentation are documented in [content.md](content.md). Testing and platform scope are documented in [testing.md](testing.md).

---

## Goal

The application notifies the user when an important limit event requires attention.

Notifications are a desktop product capability. The terminal interface does not request or deliver system notifications.

---

## Architecture

Code layout (`src/notifications/`):

- `mod.rs` — public facade, delivery trait, candidate orchestration (`send_for_report*`, `notifications_for_*`), and package tests
- `kinds.rs` — notification kinds, colors, and remaining-percent matching
- `content.rs` — notification DTO and title/subtitle/body/label projection
- `store.rs` — previous-remaining store trait and its file-backed implementation

The application uses one common notification domain model.

Provider and limit logic produce notification candidates from structured source data. The notification rules stay in the shared Rust core and must not be duplicated in the Tauri frontend.

Notification delivery is separate from notification logic.

The target delivery adapter is Tauri notifications. Platform-specific notification behavior for macOS, Windows, and Linux is delegated to Tauri unless a supported platform later requires product behavior that Tauri cannot provide.

```text
shared core
  structured source data
  notification thresholds
  previous successful remaining store
  notification text
  dedupe keys

desktop delivery adapter
  Tauri notifications
  system notification permission
  native system notification delivery
  application icon
  notification click behavior
```

The shared core must not depend on Tauri or any operating-system notification API. It produces notification candidates and passes them to the delivery layer.

The application does not maintain separate first-party macOS, Windows, and Linux notification adapters in the current target architecture. If Tauri notifications later cannot provide required product behavior on a supported platform, platform-specific delivery adapters may be introduced behind the same delivery interface.

The terminal interface does not send native operating-system notifications and does not communicate with the desktop application for notification delivery. There is no separate macOS helper, notification service, or local network listener in the target architecture.

---

## Delivery Rules

System notifications are delivered through the Tauri notifications adapter.

Rules:

- when the Tauri application is active or minimized, eligible notifications should be delivered as native system notifications through Tauri
- the application does not use a separate notification helper process
- the application does not open a local network listener for notification delivery

The user-facing notification setting controls whether notification checks are enabled. One setting covers every notification type in [content.md](content.md).

---

## Click Behavior

Clicking a system notification opens or focuses the Tauri application.

Notification clicks do not need to navigate to a specific provider, limit, event, or screen in the current target behavior.

---

## Deduplication

Rules:

- notifications do not replace each other; every delivered notification is kept as a separate system notification
- the same running process should not repeatedly send the same notification
- notifications are independent for each provider, for example Codex, Claude, and Cursor
- low-remaining notifications may still be produced per called data source when sources differ
- the 100% again type uses a shared previous-remaining key of provider + limit name and must not fire twice for the same transition when multiple sources report the same limit; a later, independent drop-below-100-then-100 cycle for that same key is a new transition and must notify again

---

## Previous remaining store

For 100% again, the application keeps the last successful remaining percent per limit.

Key:

- provider identity + limit name
- example: Codex + `5h`
- source identity is not part of the key

Persistence:

- the store survives application restarts
- only a successful structured snapshot with a known remaining percent for that limit updates the stored value
- rejected, unavailable, or otherwise unsuccessful snapshots do not clear or rewrite the stored value

---

## Calculation

Notification triggers are calculated from structured data.

Structured data is used because it is standardized and easier to process consistently across providers and sources.

Notification calculation is independent from the desktop delivery mechanism.

### Low remaining

For each shown limit with a known remaining percent, choose the matching low-remaining type from [content.md](content.md) when remaining is at or below that type's band.

### 100% again

Fire only when all of the following hold:

- the current successful snapshot has remaining percent exactly `100`
- comparison uses the exact value `100`, not a display-rounded percent
- a previous successful remaining value exists for the same provider + limit name key
- that previous value is strictly below `100`

Do not fire when:

- there is no previous successful value for the key (cold start, first sighting, or any gap with nothing stored)
- the previous value is already `100`
- remaining rose but is not exactly `100` (for example `40` → `97`)

After a successful snapshot is evaluated for a limit, update the stored previous remaining for that key to the current remaining percent.

Intentional omission:

- reaching an expected reset date-time alone does not produce a notification
- only an exact return to `100` remaining counts as replenishment confidence for this product version

---

## User Help

Help copy for notifications must stay short. Target meaning for the Help chapter:

- one settings toggle enables all notification types
- low remaining uses the existing threshold icons
- 100% again notifies only on an exact return to 100% after a stored lower reading
- first readings and partial rises below 100% do not notify
- macOS only for now
