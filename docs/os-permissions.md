# OS Permissions

Goal: the desktop app should work immediately after launch and request only OS access that is required for enabled product features.

## Rules

- Keep provider data collection enabled by default.
- Keep system notifications enabled by default.
- Do not request broad privacy access when a narrower source is enough.
- Document any new OS access before adding a provider or desktop feature that needs it.
- Verify the final signed release artifact, not only source config.

## macOS

Allowed:

- Notifications: native limit alerts.
- Network client: Cursor usage API and local notification bridge.
- Keychain read: `cursor-access-token` only.
- Local read: provider data under `~/.codex`, `~/.claude`, `~/.config/claude`, and Xcode Claude agent project data.
- Local write: ai-limits cache/config under `~/.config/ai-limits`.
- External browser open: documented setup links only.
- CLI execution: `claude` and `codex` only when CLI fallback is enabled.

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

## Windows

Allowed target:

- Native notifications.
- Provider-specific local data paths.
- Network only for providers that require a network source.
- External browser open only for documented setup links.

Windows permissions must be reviewed before signed Windows distribution.

## Linux

Allowed target:

- Native desktop notifications where supported.
- Provider-specific local data paths.
- Network only for providers that require a network source.
- External browser open only for documented setup links.

Linux permissions must be reviewed before stable Linux distribution.
