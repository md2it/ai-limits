# Tauri UI macOS Menu Bar

On macOS the native `Help` menu lists the same sections as the [Help page](help.md) left menu.

The menu is built in `src-tauri/src/main.rs` from a `HELP_CHAPTERS` list that must stay in sync with the frontend `HELP_CHAPTERS` list in `frontend/modules/help-chapters.js`. The items are appended to the default macOS `Help` submenu, so the standard app, Edit, View, and Window menus are preserved.

Selecting an item routes to the web view, which owns the Help UI, and opens the matching section.

Non-macOS platforms have no native Help menu and use the in-app [entry points](help.md) only.
