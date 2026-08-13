# Architecture

This document defines the target structure of `src/` after moving from a PoC monolith to a maintainable application.

---

## Goal

The code supports:

- a primary desktop interface
- a stateless CLI for automation and diagnostics
- multiple providers
- multiple ways to fetch data for a single provider
- small files with a clear area of responsibility

The desktop and CLI share a common core, not separate business logic.

---

## `src/` Structure

Target structure for the near term:

```text
src/
  cli/
  get_limits/
  infra/
  notifications/
  presentation/
  providers/
  lib.rs
  types.rs
```

Purpose:

- `cli/` — terminal interface, arguments, retrieval scenario flags, output, exit codes
- `get_limits/` — limits-fetching scenario and provider method integration; package layout is documented in [get-limits/overview.md](get-limits/overview.md)
- `infra/` — technical primitives for processes, HTTP, and timeouts
- `notifications/` — shared notification rules and content model; package layout is documented in [notifications/overview.md](notifications/overview.md)
- `presentation/` — shared display projection (limits, usage, time); time rules are also documented in [presentation/time-display.md](presentation/time-display.md)
- `providers/` — ways to fetch usage/limits from providers
- `lib.rs` — shared core available to different interfaces
- `types.rs` — shared types and the application's internal language

---

## Boundaries

Module rules:

- `cli/` does not fetch data from providers directly
- `cli/` calls the shared core and is responsible only for terminal behavior
- `cli/` parses `--best`/`-b` and passes the selected retrieval scenario to the shared core
- `cli/` formats terminal output from structured source reports and does not decide fallback order
- `get_limits/` coordinates provider method selection and fallback logic
- `get_limits/` owns provider fallback chains for default and best-source runs
- `get_limits/` selects the desktop CLI-first provider chains
- `get_limits/` does not run processes or HTTP directly when that can be delegated to provider/infra
- `get_limits/` does not format terminal output
- `providers/` does not format terminal output
- `providers/` returns normalized types from `types.rs`
- `providers/` follows [get-limits/providers/contract.md](get-limits/providers/contract.md)
- `presentation/` formats shared display strings and does not fetch provider data
- `infra/` does not know the business meaning of usage/limits
- `infra/` is responsible only for technical interaction with the outside world
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
- limits-fetching scenario — `get_limits/`
- process execution, HTTP, timeouts — `infra/`
- shared data structures — `types.rs`
- shared display projection — `presentation/`
- shared time display rules — `presentation/time/` and [presentation/time-display.md](presentation/time-display.md)

If a task spans more than one area, describe the overlap explicitly before making changes.
