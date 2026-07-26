# macOS Permissions

Allowed:

- Notifications: native limit alerts.
- Network client: Cursor usage API and local notification bridge.
- Keychain read: `cursor-access-token` only.
- Local read: provider data under `~/.codex`, `~/.claude`, `~/.config/claude`, and Xcode Claude agent project data.
- Local write: application-managed WebView storage for desktop settings.
- External browser open: documented setup links only.
- CLI execution: `claude` and `codex` only when Full or Best source priority is selected.
- Background service registration: the bundled user LaunchAgent through `SMAppService`, subject to user approval.

Not allowed without a new documented reason:

- Photos, Camera, Microphone, Contacts, Calendar.
- Desktop, Documents, Downloads, or full-disk access.
- Browser cookies or web session tokens.
- Cursor refresh token.
- Arbitrary shell commands or arbitrary external URLs.

## User Guidance

`Help → Permissions` is the application entry point for permission diagnostics and recovery.

It reports the Background Agent registration or approval state and provides links to the relevant macOS System Settings pages. These macOS-specific controls are not shown on Windows or Linux.

The application may register a service that is not registered, but it must not silently override a background item that the user disabled or that macOS denied.

Check release artifacts with:

```text
codesign -d --entitlements - "AI Limits.app"
codesign -dv "AI Limits.app"
```
