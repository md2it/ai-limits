# Changelog

This file records user-visible changes. Its version sections are used automatically for Git tags and GitHub Release notes.

## Unreleased

- Exposed available Codex limit resets in the terminal and desktop UI.

## [v0.0.13](https://github.com/md2it/ai-limits/releases/tag/v0.0.13) — 2026-07-08

- add desktop source priority modes Fast, Full, and Best
- UI improvements: rewritings, relayout modal window items, grouped the settings
- preserve macOS signing through release zip round-trip
- enforce local commit prefixes
- add contributor release tooling

## [v0.0.12](https://github.com/md2it/ai-limits/releases/tag/v0.0.12) — 2026-07-08

Highlights:
- Fixed macOS signing and notarization preservation through the release zip round-trip: removed `ditto --sequesterRsrc`, added shared verification script, and verify the archived artifact after extract so stapled tickets survive download.
- Added contributor release tooling: semver-based pre-releases, commit message checks, and contributor setup scripts.
- Centralized desktop OS permission handling and tightened macOS entitlements (removed JIT/unsigned-memory/library-validation allowances).
- Decoupled release version from the Tauri app version; this release is tagged `v0.0.12`.
- Updated the provider status table and refreshed product documentation.

## [desktop-unstable-8-1](https://github.com/md2it/ai-limits/releases/tag/desktop-unstable-8-1) — 2026-07-07

Desktop build, notification, and UI polish update.

- Added Apple notarization support for macOS desktop builds.
- Improved macOS build scripts and release workflow documentation.
- Routed desktop notifications through Tauri.
- Unified user-facing time display across the desktop UI.
- Improved provider card layout and source labels.
- Updated the provider status table.
- Refined DevOps and desktop build documentation.

## [desktop-unstable-5-1](https://github.com/md2it/ai-limits/releases/tag/desktop-unstable-5-1) — 2026-07-06

Desktop beta usability and provider display update.

- Improved the Tauri desktop UI theme.
- Added remaining credits to provider cards.
- Improved local CLI discovery through configured PATH handling.

## [desktop-unstable-4-1](https://github.com/md2it/ai-limits/releases/tag/desktop-unstable-4-1) — 2026-07-06

Initial unstable desktop pre-release of AI Limits.

- Added the first Tauri desktop app connected to the core limits engine.
- Shows usage limits, reset times, provider status, and data sources in a desktop UI.
- Added manual refresh, loading states, and per-provider refresh controls.
- Added provider settings, notification settings, and CLI fallback controls.
- Added system notifications for limit updates.
- Added local provider integrations for Codex, Claude, and Cursor usage data.
- Added CLI watch mode.
- Added desktop app icons, application logo assets, and the initial desktop build workflow.
- Updated documentation for desktop builds, release flow, smoke testing, and beta downloads.
