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

## Cursor Session Recovery

Routine Cursor session expiry stays within one backend collection and does not show a sign-in state if recovery succeeds. The provider-specific state and retry rules are defined in [Cursor session recovery](../../get-limits/providers/cursor-auth.md#user-visible-outcomes).

After an unsuccessful transient update, keep the last successful Cursor values and their original timestamp visible with `Could not update Cursor. Showing previous data.` and Retry. Retry respects the provider recovery cooldown. Without a previous snapshot, show `Could not update Cursor. Try again.`

A confirmed missing or rejected session shows `Sign in to Cursor, then retry.` with instructions to run `cursor agent login`. Signing in to the IDE alone does not enable limits. If IDE is explicitly selected, choose Use CLI in Settings after signing in. Do not open the browser or run login automatically. Unreadable credentials show `Could not read Cursor sign-in information.` A permission denial shows `This Cursor account cannot access usage data.` Different discovered accounts show `Choose which Cursor account to use.` with IDE and CLI source choices; do not display secret tokens or automatically switch accounts.

Explicit account choice applies to this application session and uses the existing selected account's cache only. A sign-out or account change removes the previous account's displayed values. No new structured provider fields are prescribed; backend/UI mapping must preserve the existing common schema.
