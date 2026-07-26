# Inter-Process Communication

Desktop IPC is the Tauri command boundary between the WebView frontend and the Rust desktop adapter in `src-tauri/`.

Command names, request and response shapes, allowlists, and invoke call sites are defined by code and tests. This document states goals, constraints, and architectural rules only.

Related:

- adapter boundaries: [architecture.md](architecture.md)
- source selection semantics: [../get-limits/source-chains.md](../get-limits/source-chains.md)

---

## Goals

- expose the shared Rust core to the desktop UI without duplicating provider or notification logic
- refresh providers independently so a slow source does not block other provider blocks
- support planned per-provider update frequency controls
- keep desktop-only utilities behind the same IPC boundary

---

## Constraints

- Tauri commands delegate to the shared core; they do not fetch provider data or decide limit semantics themselves
- frontend settings travel as command request parameters; they are not read from a shared backend config file
- provider response fields are display-oriented and camelCase for the frontend
- provider source, data timestamp, reset time, error state, and no-fresh-data state come from the backend response
- provider update interval, pending state, status badges, and saved UI settings are frontend state
- external URL opening is allowlisted in code
- opening the CLI in a terminal is supported on macOS only; other platforms must fail as unsupported

## Rules

### Provider limits

1. The UI passes request settings on each provider fetch; the backend does not read desktop settings from a shared config
2. Normal UI refresh uses one-provider fetches, started in parallel for each enabled provider
3. Each successful response updates only that provider block
4. An error or delay for one provider must not block or discard results for other providers
5. A batch all-providers command may exist for other callers, but normal UI refresh must not depend on one combined response
6. A disabled or unknown provider for the passed query fails the command

### Utility commands

1. External URLs open only when allowlisted
2. The CLI command string invokes the running desktop executable with `--cli` and remains valid if the macOS app bundle has been moved
3. Run-in-terminal executes that same CLI command where the platform supports it

### Sources of truth

- command registration and handlers: `src-tauri/src/`
- frontend invoke call sites: `frontend/`
- behavior locks: repository tests for desktop IPC
- source chain order and usable-data rules: [../get-limits/source-chains.md](../get-limits/source-chains.md)
