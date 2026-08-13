# Codex

## Current status

Codex has two active sources and one legacy source:

- `codex_rpc_usage` (active): runs `codex app-server` and reads the account, rate limits, and usage over JSON-RPC 2.0 on stdio.
- `codex_local_usage` (active): scans local JSONL history in `${CODEX_HOME:-~/.codex}`, aggregates token usage and activity, and reads local rate-limit snapshots and subscription claims.
- `codex_cli_usage` (legacy): launches `codex`, sends `/status`, parses TUI limit lines. It is not used by any source chain; it stays documented as a fallback path in case the experimental RPC surface disappears.

`codex_rpc` replaces `codex_cli` everywhere a Codex CLI source is actually used. It returns strictly more than the TUI path — exact unix reset times instead of rendered strings, the plan tier as an enum, a server-side lifetime token total, and the reset-credit records — without a PTY and without TUI parsing.

Codex also exposes manually redeemable limit resets. `/usage` is Codex terminology; in ai-limits these records are part of limits, not token usage, and they are separate from the automatic `resets_at` time of a rate-limit window. `codex_rpc` reads their count from `rateLimitResetCredits.availableCount`. The redeeming method is forbidden; see [codex-rpc-usage.md](codex-rpc-usage.md).

Provider method details are documented in [codex-rpc-usage.md](codex-rpc-usage.md), [codex-local-usage.md](codex-local-usage.md), and [codex-cli-usage.md](codex-cli-usage.md).

---

## Field coverage by Codex source

| Structured field | `codex_rpc` | `codex_local` |
|---|---|---|
| `account.plan` | `account.planType` | `rate_limits.plan_type`, else the `chatgpt_plan_type` JWT claim |
| `account.credits_remaining` | `credits.balance` | `rate_limits.credits` when present |
| `account.subscription_started_at` | `null` (confirmed absent) | `chatgpt_subscription_active_start` claim |
| `account.renewal_at` | `null` (confirmed absent) | `chatgpt_subscription_active_until` claim, with the past-date guard |
| `account.price_amount` / `price_currency` / `price_period` | `null` (confirmed absent) | `null` (confirmed absent) |
| `account.plan_management_url` / `billing_management_url` | `null` (confirmed absent) | `null` (confirmed absent) |
| `limits[].used_percent` / `remaining_percent` | `primary`/`secondary.usedPercent` | `rate_limits.primary`/`secondary.used_percent` |
| `limits[].resets_at` | exact unix seconds from the server | local snapshot, subject to the staleness guard |
| `limits[].window_minutes` | `windowDurationMins` | `window_minutes` |
| `limits[].used_amount` / `total_amount` / `amount_unit` | `null` | `null` |
| `available_limit_resets` | `rateLimitResetCredits.availableCount` | `null` (confirmed unavailable locally) |
| `usage.tokens.total` | `summary.lifetimeTokens` (server-side) | sum of `last_token_usage` events |
| `usage.tokens` breakdown (input, cached, output, reasoning, cache write) | `null` | `last_token_usage.*` |
| `usage.activity.sessions_count` / `turns_count` / `files_count` | `null` | unique session ids / turn ids / patched file paths |
| `usage.activity.latest_activity_at` | `null` | latest timestamped `token_count` event |
| `data_as_of` | RPC response time | latest relevant source record |

The two active sources are complementary: `codex_rpc` is authoritative for current limits, plan tier, reset credits, and the lifetime token total; `codex_local` is the only source of the token breakdown, activity counts, and subscription dates.

---

## Limitations

- `codex app-server` is marked `[experimental]` in the CLI; its contract is verified on codex-cli 0.144.6 only
- the RPC source needs an authorized Codex CLI; without it, it reports the authorization state
- Codex exposes no price, billing period, renewal, or plan-management link through either active source
- for the legacy `codex_cli` path, output remains a TUI stream that may contain terminal control sequences, depends on current TUI text, and is slow
- whether these reads consume user limits is still unverified for both CLI-backed paths

---

## Other options

| Option | Status | Comment |
|---|---|---|
| `codex app-server` JSON-RPC (`codex_rpc_usage`) | Active source | Read-only account, rate-limit, and usage methods over stdio; exact reset times and reset-credit count |
| Local telemetry files (`codex_local_usage`) | Active source | Reads `${CODEX_HOME:-~/.codex}` JSONL history, aggregates usage and activity, reads local `rate_limits` snapshots and subscription claims |
| CLI `/status` TUI (`codex_cli_usage`) | Legacy, not used | Kept documented as a fallback if the experimental RPC surface is removed |
| Official API | Not investigated | Requires separate verification of usage/limits availability for a Codex subscription |
| Frontend/dashboard API | Research-only | Possible only with a clear and safe approach to session data |
| Traffic observation | Research-only | Do not consider as a product mechanism |
