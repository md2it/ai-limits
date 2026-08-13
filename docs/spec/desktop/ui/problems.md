# Tauri UI Problems

This document defines user-facing desktop UI states where the requested limit data cannot be shown normally. Each problem must explain the situation and provide a useful next step without exposing technical implementation details.

## No Fresh Data

The shared [limit data-state semantics](../../presentation/data-states.md) define this as distinct from a source error. If checked sources return no fresh usable limit records, the provider block must show an empty state instead of a technical error like `No usable limit records from this source`.

Copy:

```text
No fresh limits' data.
```

Below the empty-state message, show a text button:

```text
More details
```

The button opens the [Help page](help.md) on its Data availability section, which explains the provider access and refresh steps.

The setup links must open externally from the Tauri app:

- Claude setup guide: <https://code.claude.com/docs/en/setup>
- Codex CLI guide: <https://developers.openai.com/codex/cli>

On the Main Window, this state also shows a `Retry` text button next to `More details` that refreshes that provider. The Popover does not show `Retry` — see [provider-blocks.md](provider-blocks.md).

## CLI Authorization

When an installed Codex CLI or Claude CLI is not authorized, show the provider-specific authorization state. Do not start authorization or open a browser when the state appears. The sign-in action is the user's explicit consent to start the provider login flow and may open a browser.

Codex CLI copy:

```text
You’re not signed in to Codex CLI.
[Sign in to Codex]
Or run manually: `codex login`
```

Claude CLI copy:

```text
You’re not signed in to Claude CLI.
[Sign in to Claude]
Or run manually: `claude login`
```

On the Main Window, this state also shows a `Fix access` text button that opens the [Help page](help.md)'s `permissions` section. The Popover does not show `Fix access` — see [provider-blocks.md](provider-blocks.md).
