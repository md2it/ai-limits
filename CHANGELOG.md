# Changelog

This file records user-visible changes. Its version sections are used automatically for Git tags and GitHub Release notes.

Format: each version groups its entries under plain-text labels, in this order when present — `New feature`, `Updated`, `Fixed`, `Infra`, `Doc`. Labels with nothing under them are omitted. Keep every bullet to one short line and end it without a period; when a change is too big for that, split it into several bullets or into a second-level sub-list. A bullet introducing a sub-list ends with a colon.

## Unreleased

Updated:
- The desktop app no longer opens a local notification server

## [v0.5.0](https://github.com/md2it/ai-limits/releases/tag/v0.5.0) — 2026-08-06

New feature:
- The macOS menu bar Popover
- Shared live cache and refresh schedule between Main Window and Popover
- One shared Update frequency dropdown in Settings → Other for all provider cards (default 10 min)

Updated:
- macOS opens the Main Window only on the first launch; later launches start from the menu bar
- Main Window reworked into Overview, Settings, and Help
- Provider cards no longer have their own refresh-frequency or per-card UPDATE NOW controls
- Redesign:
  - Main Window navigation and Help share quiet grouped controls in both themes
  - Main Window actions now share one glass state pattern for rest, hover and press

Fixed:
- Cmd+W closes the Main Window from fullscreen
- Popover no longer flickers when switching Spaces

## [v0.4.0](https://github.com/md2it/ai-limits/releases/tag/v0.4.0) — 2026-08-04

Updated:
- Release builds are size-optimized workspace-wide:
  - Link-time optimization, stripped symbols, abort on panic
  - Windows and Linux binaries shrink too
- macOS now ships as two downloads, Apple Silicon and Intel. The installed macOS app is much smaller:
  - Apple Silicon: 32.5 MB to 5.9 MB (-82%)
  - Intel: 32.5 MB to 6.5 MB (-80%)
- Every release asset shrank compared to v0.3.0, measured from the actual GitHub Actions build output:
  - -74.3% — macOS updater archive (Apple Silicon)
  - -73.8% — macOS .dmg (Apple Silicon)
  - -72.2% — macOS updater archive (Intel)
  - -71.2% — macOS .dmg (Intel)
  - -50.0% — Linux .deb
  - -42.1% — Windows .msi
  - -38.0% — Windows .exe
  - -3.4% — Linux .AppImage, mostly the bundled WebKitGTK runtime rather than app code
- The auto-update manifest points each architecture at its own signed archive:
  - Installs from the previous universal build need a one-time manual reinstall

## [v0.3.0](https://github.com/md2it/ai-limits/releases/tag/v0.3.0) — 2026-08-04

New feature:
- A "100% again" notification, next to the existing low-remaining alerts:
  - Fires when a provider's remaining limit returns to exactly 100% after being lower
  - A first reading or a partial rise (40% to 97%) does not notify
- Two Display settings splitting each card's source line in two:
  - `Source` — e.g. `RPC, as of 10:10`, off by default
  - `Update time` — `Last upd hh:mm, next hh:mm` or `Manual only`, on by default
- The app version in Help > About

Updated:
- Dropped the Fast/Full/Best source-priority setting:
  - The app always queries providers over RPC first, falling back to local data
  - RPC removed the speed cost the setting traded freshness against
- Renamed the Display settings `Show limits` to `Limits` and `Show plan` to `Subscription`
- Moved each card's update-frequency picker into a gear dropdown next to `UPDATE NOW`:
  - Replaces the permanent `Upd every` row
- Removed the transient refresh status overlay from the desktop cards
- Kept the Limits and Plan slots aligned across visible cards, reserving space for empty sections
- Codex limits now come from the Codex CLI's app-server, not its text interface:
  - Answers in seconds
  - Adds exact reset times, plan tier, credit balance, available resets, lifetime token total
- Claude limits via the CLI now come from a direct usage request:
  - Arrive in about two seconds, without a terminal emulator or account quota
  - Adds plan tier, exact reset times, extra usage allowance, current-period spend
- Claude limits without the CLI now come from the snapshot the Claude app writes on `/usage`:
  - The 5-hour and 7-day percentages and reset times are Claude's own, not a local estimate
  - Shown with the time it was taken, since it refreshes only when `/usage` is opened
  - Adds plan, subscription start date, usage credit spend
  - Falls back to Claude's usage totals when no transcripts remain on the machine
- Cursor now reports plan name and price, renewal date, and included spend allowance:
  - Plus the token breakdown and total, and session, turn, and event counts per cycle
- Paths in output, raw data, and the Help command now start at `~`
- Simplified the macOS menu bar: dropped File and View, replaced Services with Settings…

Fixed:
- The macOS disk image shipped without its styled window:
  - The background and Applications shortcut were missing since the first disk image release
  - Arranging them needs a Finder permission unavailable in GitHub Actions

## [v0.2.0](https://github.com/md2it/ai-limits/releases/tag/v0.2.0) — 2026-08-03

New feature:
- Self-updating macOS installations:
  - Checks at startup and every 12 hours, downloads in the background
  - Offers a restart once ready
  - Can be turned off in Settings

Updated:
- Gave the macOS app icon the rounded platform shape; Windows and Linux stay edge to edge

## [v0.1.0](https://github.com/md2it/ai-limits/releases/tag/v0.1.0) — 2026-08-02

Updated:
- macOS is installed from a signed and notarized disk image:
  - The app bundle archive is no longer a separate download

Doc:
- Refreshed the README showcase screenshots from the current UI
- Added a thin gray border to showcase frames so they stay visible on light and dark backgrounds

## [v0.0.15](https://github.com/md2it/ai-limits/releases/tag/v0.0.15) — 2026-08-02

Fixed:
- Codex CLI authorization detection when the CLI writes its signed-in status to diagnostics

Infra:
- Enabled a restrictive Content Security Policy for the desktop app

## [v0.0.14](https://github.com/md2it/ai-limits/releases/tag/v0.0.14) — 2026-07-30

New feature:
- A desktop Help page with a section sidebar:
  - Opened from a new info button, and on macOS from the Help menu
  - The source priority explanation moved here; its modal is gone
- An authorization message when Codex CLI or Claude CLI is installed but not signed in:
  - Shows the manual login command
  - A Sign in action starts login only when chosen
- The local CLI command in Help → CLI mode, with Copy and Run in Terminal actions
- Available Codex limit resets in the terminal and desktop UI

Updated:
- Made the CLI stateless: one query per run, built-in defaults, no config file or watch mode
- Outdated local Codex and Claude snapshots are rejected after their expected reset time:
  - The configured fallback source is used when available
- Renamed the desktop credit and reset availability labels and aligned their styling
- Clarified the desktop Help copy on privacy, source priority, and unavailable limit data
- Removed the unsupported Claude statusline source and setup

Infra:
- Disabled automatic desktop rebuilds; dev builds now run on explicit command

Doc:
- Added a browser-only screenshot showcase with macOS, Windows, and Linux window frames

## [v0.0.13](https://github.com/md2it/ai-limits/releases/tag/v0.0.13) — 2026-07-08

New feature:
- Desktop source priority modes: Fast, Full, and Best

Updated:
- UI polish: reworded copy, relaid out modal items, grouped the settings

Fixed:
- macOS signing now survives the release zip round-trip

Infra:
- Added contributor release tooling
- Enforced local commit prefixes

## [v0.0.12](https://github.com/md2it/ai-limits/releases/tag/v0.0.12) — 2026-07-08

Fixed:
- macOS signing and notarization now survive the release zip round-trip:
  - Removed `ditto --sequesterRsrc`
  - Added a shared verification script
  - Verify the archived artifact after extract, so stapled tickets survive download

Infra:
- Added contributor release tooling: semver pre-releases, commit checks, setup scripts
- Centralized desktop OS permission handling
- Tightened macOS entitlements: no JIT, unsigned-memory, or library-validation allowances
- Decoupled the release version from the Tauri app version

Doc:
- Updated the provider status table and refreshed the product documentation

## [desktop-unstable-8-1](https://github.com/md2it/ai-limits/releases/tag/desktop-unstable-8-1) — 2026-07-07

New feature:
- Apple notarization for macOS desktop builds
- Desktop notifications routed through Tauri

Updated:
- Unified the user-facing time display across the desktop UI
- Improved the provider card layout and source labels

Doc:
- Updated the provider status table
- Improved the macOS build script and release workflow documentation
- Refined the DevOps and desktop build documentation

## [desktop-unstable-5-1](https://github.com/md2it/ai-limits/releases/tag/desktop-unstable-5-1) — 2026-07-06

New feature:
- Remaining credits on provider cards

Updated:
- Improved the Tauri desktop UI theme
- Local CLI discovery now honors the configured PATH

## [desktop-unstable-4-1](https://github.com/md2it/ai-limits/releases/tag/desktop-unstable-4-1) — 2026-07-06

Initial unstable desktop pre-release of AI Limits.

New feature:
- The first Tauri desktop app, connected to the core limits engine
- Usage limits, reset times, provider status, and data sources in a desktop UI
- Manual refresh, loading states, and per-provider refresh controls
- Provider, notification, and CLI fallback settings
- System notifications for limit updates
- Local provider integrations for Codex, Claude, and Cursor usage data
- CLI watch mode

Infra:
- Desktop app icons, logo assets, and the initial desktop build workflow

Doc:
- Documentation for desktop builds, release flow, smoke testing, and beta downloads
