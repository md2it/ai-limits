# Notification Testing

## Testing

General testing entry point: [Testing](../testing/testing.md).

Manual testing verifies notification delivery from the running desktop application after an eligible limit event. The terminal interface is not a notification test or delivery channel.

Trigger calculation is covered with unit tests using fake structured data, including:

- low-remaining threshold matching
- 100% again when previous remaining is below 100 and current is exactly 100
- no 100% again when previous is missing, already 100, or current is not exactly 100

---

## Platform Scope

Development targets:

- macOS
- Windows
- Linux

The target delivery adapter for every supported desktop platform is Tauri notifications. Notification delivery must not require a local TCP listener or a separately running CLI process.

Initial development is checked directly on macOS. Windows and Linux behavior must be tested later by external testers who have access to those systems.
