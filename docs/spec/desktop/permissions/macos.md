# macOS Permissions

Allowed:

- Notifications: native limit alerts.
- Network client: Cursor usage API.
- Keychain read: `cursor-access-token` only.
- Local read: provider data under `~/.codex`, `~/.claude`, `~/.config/claude`, and Xcode Claude agent project data.
- Local write: application-managed WebView storage for desktop settings.
- External browser open: documented setup links only.
- CLI execution: `claude` and `codex` only for the desktop application's CLI-first provider chains.

Not allowed without a new documented reason:

- Photos, Camera, Microphone, Contacts, Calendar.
- Desktop, Documents, Downloads, or full-disk access.
- Browser cookies or web session tokens.
- Cursor refresh token.
- Arbitrary shell commands or arbitrary external URLs.

Check release artifacts with:

```text
codesign -d --entitlements - "AI Limits.app"
codesign -dv "AI Limits.app"
```
