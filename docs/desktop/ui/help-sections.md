# Tauri UI Help Sections

Sections are defined once as `HELP_CHAPTERS` in `frontend/index.html`. Each entry provides an `id`, a menu `label`, and rendered content. Adding an entry adds it to the left menu and, on macOS, to the native Help menu.

Current sections, in menu order:

- `about` — what the app does, that it's free, cross-platform, and notification-driven, and where its data comes from.
- `providers` — how each provider (Codex, Claude, Cursor) gets its data, and that visibility is controlled in settings.
- `source-priority` — the Fast, Full, and Best modes, the speed/accuracy tradeoff, the provider scope, and the CLI setup guide links. See [settings.md](settings.md).
- `data-errors` — why a provider shows "no fresh data" and what to check, with a link to `source-priority`.
- `notifications` — what triggers a system notification and the current macOS-only limitation.
- `permissions` — the OS-level access the app uses (network, Keychain, local files, notifications, CLI execution, and macOS background collection) and why, including available diagnostics and recovery actions.
- `cli-mode` — the tradeoffs of the terminal interface versus the desktop app, the exact command for the running app, and actions to copy or run it in Terminal.
- `limitations` — the current known gaps, mirroring the README limitations list.
- `for-developers` — that the project is MIT-licensed and open source, its stack, and links to GitHub and the license.

Chapters may link to each other via a `data-open-help` button that switches the selected section without leaving the Help page.

The `permissions` section is cross-platform, but its content and actions are platform-aware. On macOS it shows Background Agent registration or approval status and links to the relevant System Settings pages. macOS-specific status, links, and actions are not rendered on Windows or Linux.

An automatic-update action opened from a stale macOS widget routes directly to the `permissions` section.

Keep this content in sync with the app: when a change affects what a section describes — a setting, a state, a permission, or a link — update the matching chapter in `frontend/index.html` as part of that same change, not as a follow-up.
