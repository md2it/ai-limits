# Terminal UI

This document describes the stateless terminal interface of `ai-limits`.

The desktop application is the primary product interface. The terminal interface is a stable headless contract for automation, diagnostics, and source inspection. Each invocation performs one query and exits.

Output formatting is documented in [output-format.md](output-format.md), user-facing problem output in [problems.md](problems.md), the loader in [loader.md](loader.md), and the concurrency/module architecture in [architecture.md](architecture.md).

---

## Help

`ai-limits --help` uses the common frame.

Format:

```text

=-=-=-=-=-=-= AI LIMITS =-=-=-=-=-=-=

Usage:
  ai-limits [OPTIONS]

Options:
  --help, -h       Show this help
  --all, -a        Query all current sources
  --best, -b       Query best available source per provider
  --usage          Show user-facing usage summary
  --raw, -r        Return raw source data
  --structured, -s Return structured source data

Technical source options:
  --codex-local       Query Codex from local session JSONL files
  --codex-rpc         Query Codex through the Codex CLI app-server RPC
  --codex-cli         Query Codex through the Codex CLI TUI (legacy)
  --claude-rpc        Query Claude through the Claude CLI control request
  --claude-cli        Query Claude through the Claude CLI TUI (legacy)
  --claude-local      Query Claude from local transcripts and state files
  --cursor-api2       Query Cursor through api2.cursor.sh

Examples:
  ai-limits --all
  ai-limits --best
  ai-limits --all --usage
  ai-limits --all --raw
  ai-limits --all --structured

=-=-= DONE 2026-07-02 15:04:05 =-=-=

```

Default output is the user-facing limits presentation. `--usage` is the user-facing usage presentation. `--raw` returns captured source data, and `--structured` returns normalized source data as formatted JSON. These technical modes support automation, development, testing, diagnostics, and provider contract checks.

Without explicit source flags, default limits output uses the built-in `fast_free` source chain from [get-limits/source-chains.md](../get-limits/source-chains.md).

`--codex-cli` is a legacy diagnostic flag: the TUI source it queries is not part of any source chain and is not queried by `--all`. The Codex CLI-backed source used by the chains is `codex_rpc` ([get-limits/providers/codex-rpc-usage.md](../get-limits/providers/codex-rpc-usage.md)).

`--claude-cli` is a legacy diagnostic flag on the same terms: the TUI source it queries is not part of any source chain and is not queried by `--all`. The Claude CLI-backed source used by the chains is `claude_rpc` ([get-limits/providers/claude-rpc-usage.md](../get-limits/providers/claude-rpc-usage.md)).

The terminal interface has no configuration file and does not read desktop settings. Runtime behavior is determined only by built-in defaults and explicit command-line arguments.

`--best`/`-b` uses the `cli_fallback` source chain and prints one selected block per provider.

Fallback chains print only the selected source report for each provider. Fallback order is decided before formatting. Failed earlier attempts are not printed unless that source is selected directly or through `--all`.

Default limits and `--usage` output are built from structured source reports: provider labels, fixed-width limit rows, remaining-limit bars, `Source` lines, and unavailable or no-data messages. Frame and loader drawing are separate from that formatting and are documented in [output-format.md](output-format.md) and [loader.md](loader.md).

`--best` applies to limits output and can be combined with `--raw` or `--structured` for the selected source reports. It cannot be combined with `--usage`.

`--all` prints every current source separately and does not apply best-source fallback.

Technical source options are working source selectors, but they are primarily intended for intermediate source-level workflows.
