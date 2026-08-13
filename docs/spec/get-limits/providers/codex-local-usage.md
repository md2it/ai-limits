# Codex Local Usage

## Provider Method: `codex_local_usage`

Code layout (`src/providers/codex_local/`):

- `mod.rs` — public facade (`get_usage` / `collect`) and package tests
- `raw.rs` — raw DTO and internal accumulation model
- `scan.rs` — Codex home resolution and JSONL directory/file scan
- `parse.rs` — token-event / rate-limit / credits parsing
- `auth.rs` — subscription dates, the offline plan fallback, and the subscription freshness marker, all from the local auth token
- `project.rs` — structured projection and raw encode/decode

Minimal source:

- root: `${CODEX_HOME:-~/.codex}`
- scanned directories: `sessions/`, `archived_sessions/`
- scanned files: `**/*.jsonl`
- subscription dates: `auth.json`

What is extracted:

- events with `"type":"token_count"` and `"last_token_usage"`
- totals: input, cached input, output, reasoning output, cache write, total
- activity counts: sessions, turns, and changed files
- latest activity timestamp (ISO 8601 UTC from the latest `token_count` event with `rate_limits`)
- `rate_limits` snapshot when present: `primary.used_percent`, `primary.window_minutes`, `primary.resets_at`, `secondary.used_percent`, `secondary.window_minutes`, `secondary.resets_at`, `credits` (`balance` or scalar), `plan_type`
- subscription dates from `auth.json`

How to get these fields from local files:

1. read `${CODEX_HOME:-~/.codex}/sessions/**/*.jsonl` and `${CODEX_HOME:-~/.codex}/archived_sessions/**/*.jsonl`
2. keep only records where `type = "event_msg"` and `payload.type = "token_count"`
3. for usage, aggregate `payload.info.last_token_usage.*`
4. for limits/reset, read `payload.rate_limits.*` from the latest timestamped event that includes `rate_limits`
5. for activity counts, read the session, turn, and patch records described in [Activity counts](#activity-counts)
6. show `Latest activity` and `resets_at` as ISO 8601 UTC (`YYYY-MM-DDTHH:MM:SSZ`)

## Token fields

`usage.tokens.*` is aggregated from `payload.info.last_token_usage`. Beyond the input/cached-input/output/reasoning/total fields, `usage.tokens.cache_write` is filled from `payload.info.last_token_usage.cache_write_input_tokens`.

`cache_write_input_tokens` is optional per event. It is summed over the events that report it, and when **no** scanned event reports it the field stays `null` plus a diagnostic — not `0`. The two states are different facts: `0` would state that the account wrote nothing to cache, while the truth is that these records say nothing about cache writes at all. The same distinction applies to any other token field the scanned records omit.

## Activity counts

All three activity counts come from the same JSONL scan and are counts of distinct identifiers, never counts of records:

- `usage.activity.sessions_count` — the number of unique `session_id` values found in `session_meta` records
- `usage.activity.turns_count` — the number of unique `turn_id` values found in `event_msg` records of type `task_started` and `task_complete`. A turn seen in both records counts once.
- `usage.activity.files_count` — the number of unique file paths found in `event_msg` / `patch_apply_end` `changes`. This is the count of **changed user files**, which is what the Usage output kind in [output-kinds.md](../../presentation/output-kinds.md) means by "files".

The number of scanned JSONL files is **not** `files_count`. It is an internal scan metric about the source itself, it says nothing about the user's work, and it must not reach `usage.activity.files_count`. If it is exposed at all, it belongs in raw data or diagnostics.

## Subscription dates

`${CODEX_HOME:-~/.codex}/auth.json` holds `tokens.id_token`, a JWT whose payload carries the ChatGPT subscription claims under the namespaced object `https://api.openai.com/auth`:

- `chatgpt_subscription_active_start` -> `account.subscription_started_at`
- `chatgpt_subscription_active_until` -> `account.renewal_at`
- `chatgpt_plan_type` -> `account.plan`, offline fallback only (see below)
- `chatgpt_subscription_last_checked` -> the age of the subscription data, used for diagnostics and for `data_as_of` reasoning about the subscription fields

How this is read:

1. read `auth.json` and take `tokens.id_token`
2. base64url-decode the middle JWT segment only and parse it as JSON; the signature is never verified and no crypto dependency is used
3. read the listed claims from the `https://api.openai.com/auth` object and normalize the timestamps to ISO 8601 UTC (`YYYY-MM-DDTHH:MM:SSZ`)

Rules for this read:

- only the subscription timestamps, the plan name, and the last-checked timestamp leave the auth module; the access token, refresh token, id token, account identifiers, email, and every other claim never reach stdout, stderr, `diagnostics`, `raw`, or any error message
- failures never carry the offending content: a missing file, unreadable file, unparseable JSON, malformed JWT, absent claim, or unparseable timestamp degrades to `null` plus a short generic diagnostic
- `account.plan` comes from `rate_limits.plan_type` when a local snapshot provides it. Only when that is unavailable does the `chatgpt_plan_type` claim fill `account.plan`, as the offline fallback, with a diagnostic recording that the plan came from the cached auth token rather than from a limits snapshot.
- `chatgpt_subscription_last_checked` is the issuing-time freshness marker for the subscription claims. It is reported as the age of the subscription data in `diagnostics`; it is not `collected_at` and it does not override `data_as_of` for usage or limits, which describe different records.
- `chatgpt_subscription_active_until` is an active-until date, which for a cancelled subscription is the end of access rather than a renewal; whenever `renewal_at` is populated from it, a diagnostic records that ambiguity

### Stale token windows

The claims describe the subscription window as of token issuance, not as of collection time. `auth.json` is rewritten only when the token is refreshed, so a window that has already rolled over stays in the file: on a verified machine the id token was issued on 2026-07-30 (`iat`, matching `chatgpt_subscription_last_checked` and the file's own `last_refresh`) and still reported `chatgpt_subscription_active_until` as 2026-08-02 on 2026-08-03, while the account was actively consuming its weekly quota.

A past active-until date can mean either a stale token or a genuinely lapsed subscription, and local data cannot tell them apart. Presenting it as an upcoming renewal would be the weak assumption that [structured-info-rules.md](../structured-info-rules.md) forbids, so:

- when the parsed active-until date is earlier than collection time, `renewal_at` is `null` and a diagnostic records that the window has already ended and could not be confirmed as current
- the comparison uses the same collection timestamp that produces `collected_at`, so a single run stays internally consistent
- a 2-minute grace matches the local-reset expiry grace in `src/get_limits/freshness.rs`, which rejects local snapshots the same way when an automatic reset time has passed
- `subscription_started_at` is not checked this way; a start date in the past is correct and expected

Behavior:

- if root is missing, returns `not found`
- if `auth.json` is missing or unusable, subscription dates stay `null` and the source still reports usage and limits
- if no token events are found, returns `token events: not found`
- local Codex JSONL can provide current local snapshot for limit percent and reset time (5h/weekly windows when present)
- local Codex JSONL usually does not provide absolute quota size (`used_tokens`/`max_tokens`), only percent and reset window
- local Codex JSONL and the local Codex state database did not expose manual reset count, type, or expiry during verification; they must not be used as a source for `available_limit_resets`, whose active source is `codex_rpc` ([codex-rpc-usage.md](codex-rpc-usage.md))
- confirmed limit: local Codex data contains no price, currency, billing period, or billing/plan-management link. `account.price_amount`, `account.price_currency`, `account.price_period`, `account.plan_management_url`, and `account.billing_management_url` stay `null` for this source, and public plan prices must never be hardcoded to fill them.
