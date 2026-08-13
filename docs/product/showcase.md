# Browser Screenshot Showcase

The browser showcase renders the production desktop frontend with fixed demonstration data and a decorative macOS Tahoe, Windows, or Linux window frame — or, for `popover`, the [macOS Menu Bar Popover](../spec/desktop/mac-popover.md) static layout. It is for creating screenshots and previewing layout only; it does not connect to Tauri, Rust, provider APIs, or local accounts.

From the project root, run `python3 -m http.server 1420 --bind 127.0.0.1 --directory frontend`, then open `http://127.0.0.1:1420/?showcase=macos`. Replace `macos` with `windows`, `linux`, or `popover`, or use the platform controls beside the frame.

To replace the README images in `docs/readmes/screenshots/` in one step, run `npm run screenshots:update` (or `sh scripts/update-showcase-screenshots.sh`). The script starts the local frontend server, captures the seven showcase shots (including Popover, settings, and help), overwrites those PNG files, and stops the server when finished.

## Running the Screenshot Script

Run the screenshot command manually in the user's normal macOS Terminal, outside a Codex workspace, agent sandbox, or other restricted execution environment. Playwright Chromium needs macOS process permissions that an agent sandbox may deny, even when the showcase itself opens correctly in a browser.

Copy and paste this command into Terminal:

```sh
cd /Users/alekseyterekhov/Projects/ai-limits
npm run screenshots:update
```

The command is successful only after it prints `Wrote` for all seven files. Do not treat a successful server start as a successful screenshot update.

The fixed data is defined in `frontend/modules/showcase.js` so its shape follows the rendered provider cards. Update it only when a new screenshot needs different content or the displayed data model changes.

The `macos`, `windows`, and `linux` frames start at a compact size and can be resized by dragging the invisible bottom-right corner; the platform controls change only the decorative window chrome. Showcase windows use a 1px gray border so README screenshots stay visible on light and dark GitHub backgrounds.

## Popover Variant

`?showcase=popover` previews the [macOS Menu Bar Popover](../spec/desktop/mac-popover.md), per [mac-popover.md#static-layout](../spec/desktop/mac-popover.md#static-layout). Unlike the other three, it is not a decorative OS window with traffic lights or a resize handle — it is not an OS window at all, so that chrome does not apply. It relocates the same live, real provider cards used by the other platforms (same `SHOWCASE_PROVIDERS` mock data, same interactivity) into a `.popover-root` panel: the top bar (app name plus `[update all]`, and the segmented `[All] [Codex] [Claude] [Cursor]` view control), the scrolling card area, and a footer row (`Open AI Limits` plus `[info]` and `[gear]`) — no Help entry point. Markup and styles both come from the files the real Popover window uses (`frontend/modules/popover-toolbar.js`, `frontend/styles/popover.css`), so the preview and the shipped panel cannot drift.

The panel itself gets no showcase-specific styling. What the stage around it adds is only what the real window gets from macOS and CSS cannot: a neutral backdrop to be translucent against, a stand-in for the native window shadow, and the panel width the native window would otherwise fix (`.showcase-capture-area--popover` in `frontend/styles/showcase.css`).

Native tray/popover/window logic now backs the real Popover window (see [mac-popover.md](../spec/desktop/mac-popover.md)); this variant remains the browser-side preview of the same surface. The fixed screenshot set includes dark and light Popover shots.

## Screenshot Requirements

Fixed rules for `scripts/update-showcase-screenshots.mjs`, so future updates don't have to re-derive them:

- Display toggles: `showPlan`, `showSource`, and `showUpdateTime` are OFF (only `showLimits` is on). Cards must show limit bars only, no PLAN section, no `Source ...` line, no `Last upd ...` line.
- Viewport `1400x900`, `deviceScaleFactor: 1`.
- Theme: `dark` for `macos.png`, `windows.png`, `linux.png`, `macos-help.png`; `light` for `macos-light-settings.png`.
- Window frame is resized to `1010px` wide with `box-shadow: none` before capture; page background is made transparent.
- The capture target is `.showcase-window` with `omitBackground: true` (PNG keeps transparency outside the frame).
- Seven fixed shots: `macos`, `windows`, `linux` (plain), `macos-light-settings` (settings dropdown open), `macos-help` (help view open), `macos-popover-dark`, and `macos-popover-light`.
- Popover shots use `.popover-root` as the capture target. The shared Popover styles always hide source and update-time lines, matching the real Popover window.
- Mock data always comes from `frontend/modules/showcase.js`; never hand-edit a screenshot or hardcode different numbers per shot.
