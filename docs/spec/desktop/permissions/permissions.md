# OS Permissions

Goal: the desktop app should work immediately after launch and request only OS access that is required for enabled product features.

## Rules

- Keep provider data collection enabled by default.
- Keep system notifications enabled by default.
- Do not request broad privacy access when a narrower source is enough.
- Document any new OS access before adding a provider or desktop feature that needs it.
- Verify the final signed release artifact, not only source config.

## Per-Platform Details

- [macOS permissions](macos.md)
- [Windows permissions](windows.md)
- [Linux permissions](linux.md)
