# Terminal Architecture

## Parallel Model

Selected sources are started in parallel.

Execution model:

```text
provider worker threads
        ↓
channel events
        ↓
cli event loop
        ↓
terminal renderer
```

If multiple sources are waiting at the same time, multiple loader lines are displayed.

Format:

```text
⠋ waiting codex-rpc
⠙ waiting claude-rpc
```

When a source finishes, its loader is cleared and the result is printed as soon as it is ready.

---

## Architectural Boundaries

Layout:

```text
src/cli/
  mod.rs     — thin public facade (`run` / `run_with_args`) and top-level CLI flow
  args.rs    — argument parsing (`CliArgs`, `OutputMode`, `parse_args`) and source-plan resolution from flags
  run.rs     — parallel source orchestration: worker threads, channel event loop, loader state
  render.rs  — projection of source reports and failure blocks into terminal provider blocks
  help.rs    — help text content

src/infra/loader.rs
  - selects unicode/ascii spinner
  - draws loader lines
  - clears loader lines
  - prints frames and headers

src/get_limits/
  - calls provider methods
  - selects fallback chains for default and best-source runs
  - returns normalized SourceReport

src/providers/*
  - fetches source data
  - returns raw and structured data
  - does not render terminal UI
```

`cli/` keeps fallback/chain selection in `get_limits`; the package only chooses a plan from flags and runs it.
