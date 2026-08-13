# Data Retrieval Options

## Data retrieval options

| Option | Status | Pros | Cons | Providers |
|---|---|---|---|---|
| Official API | Preferred when available | Stability, clear support, low risk | Often requires a key, enabling the API, or an Enterprise plan | Cursor Enterprise; potentially Codex/Claude/API providers |
| Local transcript/telemetry files | Implemented in PoC | Fast, local, no network requests and no quota consumption | Usually gives usage history, but not always the official remaining limit/reset | Claude (`~/.config/claude/projects`, `~/.claude/projects`, Xcode ClaudeAgentConfig), Codex (`~/.codex`) |
| Local derived DB/cache | Auxiliary layer | Speeds up the dashboard, incremental recalculation, and history after upstream cleanup | Not the primary source of truth; must stay in sync with source files | Claude (`~/.claude/usage.db`, tool-specific caches) |
| Provider CLI (TUI) | Legacy for Codex and Claude | Works in a minimal user scenario using an already configured CLI | Slow, fragile TUI parsing, requests may consume resources | Codex (legacy `codex_cli`), Claude (legacy `claude_cli`) |
| Provider CLI RPC server | Used for Codex and Claude | Machine-readable JSON over stdio, no PTY, exact numeric values | Marked experimental by both CLIs; contract stability between versions is unverified, and Claude publishes no schema dump | Codex (`codex_rpc`), Claude (`claude_rpc`) |
| Local token/client backend | Used for Cursor | Can work without a separate API key, using an existing login | Cursor's stable internal endpoint has no publicly documented contract and needs careful security review | Cursor |
| Frontend/dashboard API via cookie | Research-only | Often exposes the same data as the web UI | Cookie is a sensitive secret, high security and ToS risk | Potentially Codex, Claude, Cursor |
| Traffic observation | Research-only | Can help understand internal contracts | HTTPS, certificate pinning, high risk of fragility and misuse | Potentially all |

## Current status by provider

| Provider | Primary known option | Status | Documents |
|---|---|---|---|
| Codex | Local JSONL history and `rate_limits` snapshots from `${CODEX_HOME:-~/.codex}` (`codex_local`) | Default implemented source. Provides token usage, activity, and current local limit/reset snapshots when present; `codex app-server` JSON-RPC (`codex_rpc`) is the CLI-backed source used for `--best`. The `/status` TUI source (`codex_cli`) is legacy and unused. | [../providers/codex.md](../providers/codex.md) |
| Claude | Local transcript reconstruction (`claude_local`) | Default implemented source. Local transcripts, the `~/.claude.json` cached snapshot, and `~/.claude/stats-cache.json` provide usage, activity, and limits; the CLI `get_usage` control request (`claude_rpc`) is the CLI-backed source used for `--best`. The `/usage` TUI source (`claude_cli`) is legacy and unused. | [../providers/claude.md](../providers/claude.md) |
| Cursor | `api2.cursor.sh` `DashboardService` methods via Cursor Agent token (`cursor_api2`) | Only implemented source. Five read-only methods provide the plan name and price, the renewal date, the percentage and monetary limits, the token breakdown, and the activity counts. Uses Cursor's stable internal endpoint, but its contract is not publicly documented; there is no CLI-based fallback source in code. | [../providers/cursor.md](../providers/cursor.md) |
