# Tauri UI Help Page

## Purpose

The Help page collects short reference sections for the desktop app. Each section fits within the standard window frame without scrolling.

## Layout

The Help page is one of the three top-level pages reached from the [top nav](layout.md#top-nav). It contains:

- a header with a centered `Help` title
- a narrow left menu listing the available sections
- a content panel on the right showing the selected section

The Help tab in the nav bar is how the user returns to the other pages; `Escape` also switches back to Overview.

On narrow windows the menu stacks above the content panel.

The available sections are listed in [help-sections.md](help-sections.md).

## Entry Points

The Help page opens from:

- the Help tab in the [top nav](layout.md#top-nav), which opens the first section
- the no-fresh-data provider state, which opens the `data-errors` section
- the CLI-not-authorized provider state's "Fix access" button, which opens the `permissions` section (Main Window only — see [provider-blocks.md](provider-blocks.md))
- on macOS, the `Help` menu bar items, which mirror the section list

## macOS Menu Bar

On macOS the native `Help` menu mirrors the section list. See [macos-menu-bar.md](macos-menu-bar.md).

Non-macOS platforms use the in-app entry points only.
