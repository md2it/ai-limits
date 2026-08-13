# Tauri UI Layout

## Purpose

The Tauri UI shows current provider limits, refresh controls, settings, source metadata, and states where provider data is unavailable.

The UI should remain compact and operational rather than marketing-oriented.

## Main Content

The UI contains:

- a top nav bar with Overview, Settings, and Help tabs
- on the Overview tab: provider blocks as the main content, then a centered last-update line, then the global manual update action below the cards
- three inline rounded squares for providers, in order:
  - Codex
  - Claude
  - Cursor

Each provider square represents one provider and contains that provider's limit details.

The window must not show a visible `AI Limits` title in the content area.

## Top Nav

A persistent nav bar (`#app-nav`) sits above all page content, with three tabs: Overview (`#nav-overview`), Settings (`#nav-settings`), and Help (`#nav-help`). Each tab shows a real page — `#overview-view`, `#settings-view`, `#help-view` — rather than a floating panel; `main.js`'s `switchView(view)` shows the matching section, hides the other two, and marks the active tab's `aria-current`. `Escape` returns to Overview from either of the other tabs.

Provider blocks, the global manual update action, and the last-update status line live on the Overview page. Settings and Help are their own pages instead of a dropdown/overlay on top of Overview.

## Update Actions

The Overview page's action row sits below the provider blocks and contains the `UPDATE ALL DATA NOW` button, which takes the available row width. Its label is centered.

The last update status line is shown between the provider blocks and the action row, centered.
