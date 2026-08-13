# Notification Content

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

### Low remaining

Notification types based on remaining limit thresholds:

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

Text template:

```text
$EMOJI AI Limits
$PROVIDER_NAME $TYPE - NN% left
reset $TIME
```

On platforms that support notification title/subtitle/body, `$EMOJI AI Limits` is the title, the limit line is the subtitle, and the reset line is the body.

Fields:

- `EMOJI`:
  - `🟢` for 75-50% remaining
  - `🟡` for 50-25% remaining
  - `🟠` for 25-10% remaining
  - `🔴` for less than 10% remaining
- `PROVIDER_NAME`: a canonical name from the shared [provider naming rules](../presentation/provider-names.md)
- `TYPE`: `5h`, `weekly`, `Cursor Models`, or `Other Models`
- `TIME`: reset timestamp formatted according to [time-display.md](../presentation/time-display.md). The notification body keeps the contextual `reset ` prefix; the shared time formatter only owns the date-time value

Examples:

```text
🟡 AI Limits
Codex weekly - 44% left
reset 22:22
```

```text
🟢 AI Limits
Cursor Cursor Models - 65% left
reset Jul 7, 22:22
```

```text
🔴 AI Limits
Claude 5h - 7% left
reset Jul 7, 22:22
```

### 100% again

Fires when a limit is fully available again. Trigger rules are in [overview.md](overview.md).

Text template:

```text
🔔 AI Limits
$PROVIDER_NAME $TYPE - 100% again
reset $TIME
```

Same field rules as low remaining for `PROVIDER_NAME`, `TYPE`, and `TIME`.

Example:

```text
🔔 AI Limits
Codex 5h - 100% again
reset Jul 7, 22:22
```
