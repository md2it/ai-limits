# Architecture

This document defines the target structure of `src/` after moving from a PoC monolith to a maintainable application.

---

## Goal

The code should support:

- a CLI interface
- a future desktop interface
- multiple providers
- multiple ways to fetch data for a single provider
- small files with a clear area of responsibility

The CLI and the future desktop should share a common core, not separate business logic.

---

## `src/` Structure

Target structure for the near term:

```text
src/
  cli/
  config/
  infra/
  notifications/
  presentation/
  providers/
  get_limits.rs
  lib.rs
  types.rs
```

Purpose:

- `cli/` — terminal interface, arguments, retrieval scenario flags, output, exit codes
- `config/` — user settings, defaults, and paths to config files
- `infra/` — technical primitives for processes, HTTP, and timeouts
- `notifications/` — shared notification service with platform adapters
- `presentation/` — user-facing display models built from structured data
- `providers/` — ways to fetch usage/limits from providers
- `get_limits.rs` — limits-fetching scenario and provider method integration
- `lib.rs` — shared core available to different interfaces
- `types.rs` — shared types and the application's internal language

---

## Boundaries

Module rules:

- `cli/` does not fetch data from providers directly
- `cli/` calls the shared core and is responsible only for terminal behavior
- `cli/` parses `--best`/`-b` and passes the selected retrieval scenario to the shared core
- `cli/` renders presentation output but does not decide user-facing limit semantics
- `presentation/` converts structured data into user-facing provider blocks
- `presentation/` selects display labels, limit rows, bar values, and fallback messages
- `presentation/` renders the selected source report and does not decide fallback order
- `presentation/` does not fetch data, read files, run commands, or call provider methods
- `get_limits.rs` coordinates provider method selection and fallback logic
- `get_limits.rs` owns provider fallback chains for default and best-source runs
- `get_limits.rs` does not run processes or HTTP directly when that can be delegated to provider/infra
- `get_limits.rs` does not format terminal output
- `providers/` does not format terminal output
- `providers/` returns normalized types from `types.rs`
- `providers/` follows [get-info/providers/README.md](get-info/providers/README.md)
- `infra/` does not know the business meaning of usage/limits
- `infra/` is responsible only for technical interaction with the outside world
- `types.rs` must not depend on CLI, desktop, the file system, or external commands

---

## Providers

Initially, `providers/` remains a flat directory.

Example:

```text
providers/
  mod.rs
  codex_cli_usage.rs
  claude_cli_usage.rs
  cursor_api2_usage.rs
```

Rules:

- one file describes one way to fetch data
- each data-fetching method must be independent of the others
- removing one method must not break the rest
- shared technical logic goes in `infra/`
- shared business types go in `types.rs`

If a single provider grows to many files, you can move to a nested structure by provider.

---

## `get_limits` Scenario

`get_limits.rs` follows the document [get-info/methods/README.md](get-info/methods/README.md).

Purpose:

- select enabled provider methods
- call provider methods in the right order
- apply provider fallback-chain logic for default and best-source runs
- assemble a shared result for the CLI and the future desktop

Boundaries:

- does not contain terminal output
- does not contain low-level process execution
- does not contain low-level HTTP primitives
- does not parse provider-specific output when that is a provider method's responsibility

---

## Presentation

`presentation/` is responsible for the default user-facing output model.

It receives structured data from the shared core and prepares provider blocks for the CLI. The default terminal presentation is documented in [terminal-ui.md](terminal-ui.md).

Responsibilities:

- group source reports into provider blocks;
- choose user-facing provider labels;
- convert limits into fixed-width rows;
- build 25-character remaining-limit bars;
- choose `Source {source}` text from structured `source` and `data_as_of`;
- prepare unavailable or no-data messages from structured status data.
- render the selected source report; fallback order is decided before presentation.

Boundaries:

- does not call providers;
- does not parse raw source data;
- does not own raw or structured serialization;
- does not draw terminal frames or loaders.

---

## Provider Specs

Provider documentation is grouped by provider:

```text
docs/get-info/providers/
  codex.md
  claude.md
  cursor.md
```

Rules:

- one spec file describes one provider
- a spec file may describe multiple provider methods
- provider method sections are named like future code files without `.rs`
- code may be more detailed than the documentation and split provider methods into separate files
- if a spec file becomes too large, it can be split by provider method

---

## Configuration

User settings must not be baked into the compiled binary.

Model:

- defaults live in code
- user config is stored in a separate runtime file
- the CLI and the future desktop use the same config
- platform-specific config file paths are defined inside `config/`

---

## Desktop

The desktop application uses Tauri as a desktop adapter to the existing Rust core.

Rules:

- the shared core must live in `lib.rs` and the `src/` modules
- the CLI must be only one interface to the core
- Tauri is a separate interface to the same core
- `src-tauri/` is a desktop adapter, not a separate business core
- Tauri must use structured data returned by the existing Rust core
- provider logic, limit semantics, configuration, and notification rules stay in `src/`
- Tauri commands delegate to core functions instead of duplicating application logic

Structure:

```text
src-tauri/
  src/
    main.rs
    commands.rs
```

Purpose:

- `main.rs` — Tauri application bootstrap, window setup, plugins, and command registration
- `commands.rs` — desktop commands exposed to the frontend and delegated to the shared core

Boundaries:

- `src-tauri/` does not fetch provider data directly
- `src-tauri/` does not decide limit semantics
- `src-tauri/` does not own notification rules
- `src-tauri/` may provide desktop-specific notification transport when needed
- `src-tauri/` may provide desktop-specific window, tray, menu, and permission integration

Current desktop command and response contract is factual and documented in [tauri/commands.md](tauri/commands.md) and [tauri/provider-contract.md](tauri/provider-contract.md).

Contract boundaries:

- `get_provider_limits` returns all enabled providers for the passed query.
- `get_single_provider_limits` returns one enabled provider for the passed provider id and query.
- `open_external_url` opens only allowlisted setup guide URLs.
- provider response fields are display-oriented and camelCase in the frontend.
- provider source, data timestamp, reset time, error state, and no-fresh-data state come from the backend response.
- provider update interval, pending state, global last-updated text, provider status badges, and saved UI settings are frontend state.
- frontend settings are passed to commands as request parameters; they are not currently read from a shared backend config file.

---

## Notifications

The `notifications/` directory contains the shared notification service.

Target structure:

```text
notifications/
  mod.rs
  macos.rs
  windows.rs
  linux.rs
  noop.rs
```

Rules:

- notifications should be a shared service, not part of desktop only
- the CLI can use notifications if the platform supports it and it is enabled in config
- platform differences must be isolated inside the notifications module
- the application should call one common notification interface, not platform-specific adapters directly
- notification requirements are documented in [notifications.md](notifications.md)

---

## Rule for Agents

When making changes, first identify the business area of the task:

- terminal behavior — `cli/`
- settings — `config/`
- data fetching — `providers/`
- presentation — `presentation/`
- limits-fetching scenario — `get_limits.rs`
- process execution, HTTP, timeouts — `infra/`
- shared data structures — `types.rs`

If a task spans more than one area, describe the overlap explicitly before making changes.
