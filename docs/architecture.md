# Architecture

This document defines the target code structure and module boundaries. Runtime components, processes, state, and information flows are documented in [runtime-architecture.md](runtime-architecture.md).

---

## Goal

The code supports:

- a primary desktop interface
- a stateless CLI for automation and diagnostics
- a planned background agent for scheduled collection without an open desktop window
- planned macOS system widgets
- multiple providers
- multiple ways to fetch data for a single provider
- small files with a clear area of responsibility

The desktop and CLI share a common core, not separate business logic.

---

## `src/` Structure

Target structure for the near term:

```text
src/
  background/       # planned
  cli/
  infra/
  notifications/
  providers/
  snapshots/        # planned
  bin/
    ai-limits-agent.rs  # planned
  get_limits.rs
  lib.rs
  types.rs
src-macos-widgets/  # planned
```

Purpose:

- `background/` — planned scheduling and coordination for collection without an open desktop window
- `cli/` — terminal interface, arguments, retrieval scenario flags, output, exit codes
- `infra/` — technical primitives for processes, HTTP, and timeouts
- `notifications/` — shared notification service with platform adapters
- `providers/` — ways to fetch usage/limits from providers
- `snapshots/` — planned shared snapshot contract and storage service
- `bin/ai-limits-agent.rs` — planned headless background entry point
- `get_limits.rs` — limits-fetching scenario and provider method integration
- `lib.rs` — shared core available to different interfaces
- `types.rs` — shared types and the application's internal language
- `src-macos-widgets/` — planned SwiftUI and WidgetKit extension that reads shared snapshots

User-facing display rules that are shared across surfaces live in documentation, not as a separate `src/` architectural layer. Terminal block formatting is documented under [terminal/](terminal/); shared time display rules are in [presentation/time-display.md](presentation/time-display.md).

---

## Boundaries

Module rules:

- `cli/` does not fetch data from providers directly
- `cli/` calls the shared core and is responsible only for terminal behavior
- `cli/` parses `--best`/`-b` and passes the selected retrieval scenario to the shared core
- `cli/` formats terminal output from structured source reports and does not decide fallback order
- `get_limits.rs` coordinates provider method selection and fallback logic
- `get_limits.rs` owns provider fallback chains for default and best-source runs
- `get_limits.rs` owns desktop source priority chains for Fast, Full, and Best modes
- `get_limits.rs` does not run processes or HTTP directly when that can be delegated to provider/infra
- `get_limits.rs` does not format terminal output
- `providers/` does not format terminal output
- `providers/` returns normalized types from `types.rs`
- `providers/` follows [get-limits/providers/contract.md](get-limits/providers/contract.md)
- `infra/` does not know the business meaning of usage/limits
- `infra/` is responsible only for technical interaction with the outside world
- `background/` uses the shared limits-fetching scenario and does not duplicate provider logic
- `snapshots/` stores normalized results and does not fetch provider data
- `src-macos-widgets/` does not fetch local provider data or duplicate limit semantics
- `types.rs` must not depend on CLI, desktop, the file system, or external commands

Provider code and spec-doc structure is documented in [get-limits/providers/code-structure.md](get-limits/providers/code-structure.md). Desktop-specific architecture (settings, Tauri) is documented in [desktop/architecture.md](desktop/architecture.md).

---

The main runtime flow for limits fetching is documented in [get-limits/overview.md](get-limits/overview.md). Terminal output formatting is documented in [terminal/interface.md](terminal/interface.md) and [terminal/provider-block-format.md](terminal/provider-block-format.md).

---

## Rule for Agents

When making changes, first identify the business area of the task:

- terminal behavior — `cli/`
- desktop settings — Tauri frontend state
- data fetching — `providers/`
- limits-fetching scenario — `get_limits.rs`
- process execution, HTTP, timeouts — `infra/`
- background scheduling — `background/`
- persisted provider snapshots — `snapshots/`
- macOS widget presentation — `src-macos-widgets/`
- shared data structures — `types.rs`
- shared time display rules — [presentation/time-display.md](presentation/time-display.md)

If a task spans more than one area, describe the overlap explicitly before making changes.
