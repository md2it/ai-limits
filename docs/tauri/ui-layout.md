# Tauri UI Layout

## Purpose

The Tauri UI shows current provider limits, refresh controls, settings, source metadata, and states where provider data is unavailable.

The UI should remain compact and operational rather than marketing-oriented.

## Main Content

The UI contains:

- top action row with global manual update and settings
- centered last update line below the top action row
- provider blocks as the main content
- three inline rounded squares for providers:
  - Codex
  - Cursor
  - Claude

Each provider square represents one provider and contains that provider's limit details.

The window must not show a visible `AI Limits` title in the content area.

## Top Actions

Global update controls live above provider blocks.

The top action row contains:

- `UPDATE ALL NOW` button
- settings button

The `UPDATE ALL NOW` button takes all available row width except the settings button area. Its label is centered.

The settings button is a small square button on the right side of the same row.

The last update text is shown under the top action row and above provider blocks, centered.

The settings dropdown opens from the top settings button and must remain visible and usable across supported window sizes, including narrow and short windows.
