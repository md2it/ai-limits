# Terminal Problems

This document defines user-facing terminal output for cases where requested limit data cannot be shown normally. Each problem must explain the situation and provide a useful next step.

## Command Error

Command errors are printed inside the common frame:

```text
ai-limits: unknown argument `--bad`
```

## No Fresh Data

When a source is available but has no supported limit data, print the provider block with a short reason and the source timestamp. The exact provider-block format is defined in [provider-block-format.md](provider-block-format.md).

## Provider CLI Authorization

When an installed provider CLI is not authorized, print the provider-specific authorization message in the relevant provider result. The terminal interface never starts authorization and does not show a sign-in button.

Codex CLI:

```text
You’re not signed in to Codex CLI.
Run it: `codex login`
```

Claude CLI:

```text
You’re not signed in to Claude CLI.
Run it: `claude login`
```
