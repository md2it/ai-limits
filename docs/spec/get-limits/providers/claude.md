# Claude

## Current status

Claude has two active sources and one legacy source:

- `claude_rpc_usage` (active): runs the Claude CLI in print mode and reads the account, rate limits, and spend through the SDK control request `get_usage` over stdio.
- `claude_local_usage` (active): scans local transcript JSONL files, reads the profile and cached limit snapshot from `~/.claude.json`, and reads usage aggregates from `~/.claude/stats-cache.json`.
- `claude_cli_usage` (legacy): launches `claude`, sends `/usage`, parses TUI limit lines. It is not used by any source chain; it stays documented as a fallback path in case the experimental control request disappears.

`claude_rpc` replaces `claude_cli` everywhere a Claude CLI source is actually used. It returns strictly more than the TUI path — the plan tier, ISO reset timestamps, absolute window amounts, the extra-usage allowance, and the spend state — in 1.6–2.0 s, over plain pipes, without a PTY, and without consuming account quota.

Provider method details are documented in [claude-rpc-usage.md](claude-rpc-usage.md), [claude-local-usage.md](claude-local-usage.md), and [claude-cli-usage.md](claude-cli-usage.md).

---

## Field coverage by Claude source

| Structured field | `claude_rpc` | `claude_local` |
|---|---|---|
| `account.plan` | `subscription_type` (`pro`/`max`/`team`/`enterprise`) | `oauthAccount.organizationType`, as reported (`claude_pro`) |
| `account.subscription_started_at` | `null` (confirmed absent) | `oauthAccount.subscriptionCreatedAt` |
| `account.renewal_at` | `null` (confirmed absent) | `null` (confirmed absent) |
| `account.credits_total` / `credits_used` / `credits_remaining` | `extra_usage.monthly_limit` / `used_credits` / calculated | same fields from the cached snapshot |
| `account.price_amount` / `price_currency` / `price_period` | `null` (confirmed absent) | `null` (confirmed absent) |
| `account.plan_management_url` / `billing_management_url` | `null` (confirmed absent) | `null` (confirmed absent) |
| `limits[].used_percent` / `remaining_percent` | `five_hour` / `seven_day` `utilization` | same, from the cached snapshot; else the 5h transcript reconstruction |
| `limits[].resets_at` | server ISO 8601, fresh on every call | cached snapshot, subject to the staleness guard; else an estimate |
| `limits[].used_amount` / `total_amount` / `amount_unit` | `used_dollars` / `limit_dollars` / `usd`, all `null` on the verified Pro account | same from the snapshot; tokens for the reconstruction |
| `usage.money.*` | `spend.used`, `spend.limit` | same from the snapshot |
| `usage.tokens` breakdown and total | `null` | transcript scan, else `stats-cache.json` `modelUsage` |
| `usage.activity.sessions_count` / `turns_count` | `null` (deliberate, see below) | transcript scan, else `totalSessions` / `totalMessages` |
| `usage.activity.files_count` | `null` | `null` (confirmed absent) |
| `usage.models.top_model` | `null` | transcript scan, else `modelUsage` |
| `available_limit_resets` | `null` (Claude has no manual resets) | `null` |
| `data_as_of` | RPC response time | `cachedUsageUtilization.fetchedAtMs`, else the latest transcript record |

The two active sources are complementary: `claude_rpc` is authoritative for current limits, the plan tier, and spend; `claude_local` is the only source of token usage, activity counts, and the subscription start date.

`claude_rpc` deliberately leaves the activity counts `null` even though the response carries a `behaviors` block: those counts are a windowed, approximate, single-machine local scan and are not comparable with the exhaustive counts every other source reports. The reasoning is recorded in [claude-rpc-usage.md](claude-rpc-usage.md#projection-into-structured-data).

---

## Freshness

`claude_rpc` makes a fresh server request on every call. `claude_local` reads `cachedUsageUtilization`, which the CLI refreshes only when the user opens `/usage` in the TUI and which can therefore be arbitrarily stale. The cache always reports its own `fetchedAtMs` as `data_as_of`; it is never presented as live.

---

## Limitations

- the `get_usage` control request is experimental by its own schema description and SDK naming, its shape is compiled into the CLI binary with no schema dump available, and it is verified on claude 2.1.220 only
- `--verbose` is mandatory for the RPC transport; without it the CLI rejects the flag combination with an error on `stderr` and exit code 1
- neither active Claude source can detect the authorization state: the `get_usage` payload carries no signal for it, so an unauthorized run surfaces as a generic no-data result instead of a "run `claude` and sign in" message, unlike Codex ([claude-rpc-usage.md](claude-rpc-usage.md#no-authorization-signal))
- Claude exposes no price, billing period, renewal date, or plan-management link through either active source
- several rate-limit windows returned by the server are code-named and are deliberately not projected, because their meaning is unknown; their key set is open and grew during verification, so they are dropped at parse time rather than kept in raw data
- for the legacy `claude_cli` path, output remains a TUI stream that depends on current CLI text and layout, and is slow
- for Claude Desktop and browser-extension flows, local-file coverage remains unverified separately from Claude Code behavior

---

## Other options

| Option | Status | Comment |
|---|---|---|
| CLI control request `get_usage` (`claude_rpc_usage`) | Active source | Read-only account, rate-limit, and spend data over stdio; fresh server values, no PTY, no quota consumed |
| Local transcripts and state files (`claude_local_usage`) | Active source | Transcript JSONL for token usage and activity, `~/.claude.json` for the profile and the cached limit snapshot, `~/.claude/stats-cache.json` for aggregates |
| CLI `/usage` TUI (`claude_cli_usage`) | Legacy, not used | Kept documented as a fallback if the experimental control request is removed |
| `claude auth status --json` | Not used | Returns `subscriptionType` without a TUI, but `get_usage` already returns the plan alongside the limits |
| macOS Keychain | Forbidden | Its only useful field is duplicated in `~/.claude.json`, and reading it risks an interactive GUI prompt in headless runs |
| `claude gateway` | Verified dead end | Enterprise component requiring Postgres and its own configuration; unrelated to the user's plan |
| `claude mcp serve` | Verified dead end | Exposes no usage-related resources |
| Official API | Not investigated | May apply to API accounts, but not necessarily to Claude Code subscription limits |
| Local SQLite/cache | Auxiliary layer | e.g. `~/.claude/usage.db` from `claude-usage`: convenient for incremental scanning, but derived data, not a primary source |
| Frontend/dashboard API | Research-only | Possible only with a clear and safe way to handle cookie/session tokens |
| Traffic observation | Research-only | Not to be considered as a product mechanism |
