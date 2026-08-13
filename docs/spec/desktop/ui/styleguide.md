# Tauri UI Style Guide

## Action Colors

Interactive controls that act (buttons, links) carry an accent color that signals the action's intent.

| Color | CSS variable | Meaning | Example |
| --- | --- | --- | --- |
| Green | `--accent-success` | Good and safe: a positive, low-risk action | Confirm / primary actions |
| Red | `--accent-danger` | Dangerous or risky: destructive or high-consequence | Destructive actions |
| Yellow | `--accent-warning` | Neutral: navigation and other actions that neither help nor endanger | Help page menu, back button, and links |

- Variables are defined in [../../../../frontend/styles/tokens.css](../../../../frontend/styles/tokens.css).
- Hover and active states are tinted from these variables (see the settings and help icon buttons for the yellow/neutral pattern).

## Other

- All external links should be marked by external-link icon
