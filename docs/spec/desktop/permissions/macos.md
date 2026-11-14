# macOS Permissions

Allowed:

- Notifications: native limit alerts.
- Network client: Cursor API2 DashboardService reads and `/oauth/token` session recovery; no model invocation
- Keychain read: `cursor-access-token` and `cursor-refresh-token` for API2 collection and bounded session recovery; no Keychain writes
- Cursor IDE local read: only the credential keys in the standard `state.vscdb`, opened read-only as specified in [Cursor session recovery](../../get-limits/providers/cursor-auth.md)
- Local read: provider data under `~/.codex`, `~/.claude`, `~/.config/claude`, and Xcode Claude agent project data.
- Local write: application-managed WebView storage for desktop settings.
- External browser open: documented setup links only.
- CLI execution: `claude` and `codex` only for the desktop application's CLI-first provider chains.

Not allowed without a new documented reason:

- Photos, Camera, Microphone, Contacts, Calendar.
- Desktop, Documents, Downloads, or full-disk access.
- Browser cookies or web session tokens.
- Writes to Cursor credential stores, browser credential extraction, or persistent copies of Cursor secrets
- Arbitrary shell commands or arbitrary external URLs.

The refresh-token read is necessary to recover ordinary access-token expiry without repeatedly asking the user to sign in. This is target permission scope; live validation is required before enabling the recovery flow. Secrets remain backend-only and in memory.

Check release artifacts with:

```text
codesign -d --entitlements - "AI Limits.app"
codesign -dv "AI Limits.app"
```

Cursor IDE credential reads execute the system `sqlite3` utility directly with a constant read-only query. No shell is used. If sqlite3 is absent, IDE fallback is unavailable; CLI credential reads remain independent.
