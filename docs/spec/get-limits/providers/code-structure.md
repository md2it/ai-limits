# Providers Code Structure

## Code

Current provider methods are nested packages under `providers/`. A new method may start as one file and nest later when a single file becomes hard to reason about. Nested packages keep a thin public facade in `mod.rs` and private internals split by responsibility (I/O, parsing, projection).

Current layout:

```text
providers/
  mod.rs
  claude_rpc/
    mod.rs
    process.rs
    parse.rs
    project.rs
  claude_cli/
    mod.rs
    capture.rs
    parse.rs
    project.rs
  claude_local/
    mod.rs
    io.rs
    parse.rs
    model.rs
    project.rs
  codex_rpc/
    mod.rs
    process.rs
    parse.rs
    project.rs
  codex_cli/
    mod.rs
    process.rs
    parse.rs
    project.rs
  codex_local/
    mod.rs
    raw.rs
    scan.rs
    parse.rs
    auth.rs
    project.rs
  cursor_api2/
    mod.rs
    fetch.rs
    parse.rs
    helpers.rs
    project.rs
```

Rules:

- one package or file describes one way to fetch data
- each data-fetching method must be independent of the others
- removing one method must not break the rest
- the package root keeps the public facade; internals stay private
- shared technical logic goes in `infra/`
- shared business types go in `types.rs`

`claude_rpc/` layout:

- `mod.rs` — thin public facade (`collect_usage`) and source identity constants
- `process.rs` — `claude` print-mode child process and control-protocol framing over stdio
- `parse.rs` — response DTOs and normalization of percents, windows, timestamps, and monetary amounts
- `project.rs` — projection into `StructuredSourceInfo` / unavailable and authorization DTOs

`claude_cli/` layout (legacy, not used by any source chain):

- `mod.rs` — public facade (`get_usage` / `collect_usage`) and source identity constants
- `capture.rs` — expect script and process capture (`capture_provider_run`)
- `parse.rs` — TUI line normalization (including CR/LF) and parsing into an internal model
- `project.rs` — projection to `SourceData` / `StructuredSourceInfo`

`claude_local/` layout:

- `mod.rs` — thin `collect()` orchestration
- `io.rs` — transcript root discovery, recursive JSONL scan, and reads of the local state files
- `parse.rs` — turn usage and server reset anchors from JSON, profile and cached-limit parsing
- `model.rs` — usage accumulation and 5h session-limit math
- `project.rs` — raw JSON and structured `SourceData` projection

The `utilization` payload parser lives in `claude_local/parse.rs`, but the shape it parses is not local: `cachedUsageUtilization.utilization` and the `rate_limits` member of the `claude_rpc` `get_usage` response are the same payload. Two packages therefore read one contract from two independent implementations, which is the situation the "each method must be independent" rule is not meant to protect — a server-side change to that payload has to be applied twice. It is a candidate for a shared module the two sources both call, once a second divergence makes the duplication concrete; until then it stays where it is rather than being moved on the strength of one shared shape.

`codex_rpc/` layout:

- `mod.rs` — thin public facade (`collect_usage`) and source identity constants
- `process.rs` — `codex app-server` child process and JSON-RPC framing over stdio
- `parse.rs` — response DTOs and normalization of percents, windows, timestamps, and credit balances
- `project.rs` — projection into `StructuredSourceInfo` / unavailable and authorization DTOs

`codex_cli/` layout (legacy, not used by any source chain):

- `mod.rs` — thin public facade (`collect_usage`) and re-export of `build_structured`
- `process.rs` — PTY expect script and `codex login status` checks
- `parse.rs` — TUI line normalization and limit/credits/reset parsing
- `project.rs` — projection into `StructuredSourceInfo` / unavailable and authorization DTOs

`codex_local/` layout:

- `mod.rs` — public facade (`get_usage` / `collect`) and package tests
- `raw.rs` — raw DTO and internal accumulation model
- `scan.rs` — Codex home resolution and JSONL directory/file scan
- `parse.rs` — token-event / rate-limit / credits parsing
- `auth.rs` — subscription dates, the offline plan fallback, and the freshness marker from the local auth token
- `project.rs` — structured projection and raw encode/decode

`cursor_api2/` layout:

- `mod.rs` — thin public facade (`collect_usage`) and re-export of `build_source_data`
- `fetch.rs` — Keychain token, the read-only `DashboardService` call sequence via `infra/os_access` (five methods for an individual account, up to seven with the team branch), and `GetFilteredUsageEvents` paging with its page cap
- `parse.rs` — path-based reads of the named responses into the internal `CursorFields` model, usage-event page accumulation, and raw-data sanitization
- `helpers.rs` — private price, date, amount, and percentage helpers for projection
- `project.rs` — projection into `SourceData` (limits, billing, money) and package tests

## Documentation

Provider documentation is grouped by provider; large providers split method details into separate files:

```text
docs/spec/get-limits/providers/
  code-structure.md
  contract.md
  claude.md
  claude-rpc-usage.md
  claude-cli-usage.md
  claude-local-usage.md
  codex.md
  codex-rpc-usage.md
  codex-cli-usage.md
  codex-local-usage.md
  cursor.md
  cursor-api2-usage.md
  cursor-options.md
```

Rules:

- one top-level spec file describes one provider
- a spec file may describe multiple provider methods
- provider method docs name the method like the source id (for example `claude_cli_usage`)
- code may be more detailed than the documentation and split a method into private modules
- if a spec file becomes too large, it can be split by provider method
