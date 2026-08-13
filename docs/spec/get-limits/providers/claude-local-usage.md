# Claude Local Usage

## Provider Method: `claude_local_usage`

Code layout (`src/providers/claude_local/`):

- `mod.rs` — thin `collect()` orchestration
- `io.rs` — transcript root discovery, recursive JSONL scan, and reads of the local state files
- `parse.rs` — assistant turn usage and server reset anchors from JSON records, profile and cached-limit parsing
- `model.rs` — accumulated usage and 5-hour session-limit math
- `project.rs` — raw JSON payload and structured `SourceData` projection

The `utilization` parser in `parse.rs` serves a shape that is not local to this source: it is the same payload `claude_rpc` receives as `rate_limits`. It is a candidate for a shared module; the reasoning is recorded in [code-structure.md](code-structure.md).

Minimal sources:

- transcripts: `~/.config/claude/projects`, `~/.claude/projects`, `~/Library/Developer/Xcode/CodingAssistant/ClaudeAgentConfig/projects`
- profile and cached limits: `~/.claude.json`
- usage aggregates: `~/.claude/stats-cache.json`

What is extracted from transcripts:

- `assistant` records with non-zero `message.usage`
- deduplicated turns by `message.id` (latest record wins in file)
- latest server reset anchor found in local JSONL records when a reset timestamp appears inside rate-limit, usage-limit, quota, or 429 payload context
- scope summary: sessions, turns, and the number of scanned transcript files
- token totals: input/output/cache-read/cache-write/total
- top model and latest activity timestamp

The number of scanned transcript files is an internal scan metric about the source itself. It is kept in raw data and **never** reaches `usage.activity.files_count`; see [Activity counts](#activity-counts).

Behavior:

- if no local roots are present **and** the state files hold no data either, returns `local transcript roots were not found`
- if roots exist but no token usage is found **and** the state files hold no data either, returns `no token usage found`
- local transcripts provide usage history; official remaining limit/reset may be unavailable

Both early returns are conditional on the state files, and that condition is what makes them correct. `~/.claude.json` and `~/.claude/stats-cache.json` are read independently of the transcript scan, and they carry the server-computed limit snapshot, the plan, the subscription start date, and the aggregate counts. An unconditional early return would discard all of it on a machine whose transcripts were deleted or never written — reporting "no data" while a usable server snapshot sat in the profile — which contradicts the rule below that a missing local file is not an error state for this source. The source reports nothing only when the transcripts and both state files are all empty.

## `~/.claude.json`

Three members of this file are read. Nothing else in it is parsed or projected.

### `oauthAccount`

- `organizationType` -> `account.plan`
- `subscriptionCreatedAt` -> `account.subscription_started_at`
- `profileFetchedAt` -> the age of these two fields, reported in `diagnostics`

Rules for this read:

- `organizationType` is used **as reported**, for example `claude_pro`. It is not rewritten into the `subscription_type` vocabulary that `claude_rpc` returns (`pro`, `max`, `team`, `enterprise`): only the one observed value can be mapped, and inventing the rest of the mapping is a weak assumption. A diagnostic records that the plan came from the local profile cache rather than from a live response.
- `profileFetchedAt` is the freshness marker for the profile fields only. It is not `collected_at` and it does not override `data_as_of`, which describes the limit snapshot.
- every other member of `oauthAccount` — `emailAddress`, `displayName`, `accountUuid`, `organizationName`, `organizationUuid`, `organizationRole`, `billingType`, `seatTier`, and the rate-limit tier fields — is not read into the model and never leaves the parsing layer.

### `cachedUsageUtilization`

`cachedUsageUtilization.utilization` has the **same payload shape** as `rate_limits` in the `get_usage` response, so one parser serves both sources: the named windows `five_hour` and `seven_day` with `utilization`, `resets_at`, `limit_dollars`, `used_dollars`, `remaining_dollars`; the flat `limits[]` records; `extra_usage`; and `spend`. The projection rules — including the rule that code-named windows are not parsed, and the rule that a named window which is `null` still yields a record when its `limits[]` entry exists — are defined once in [claude-rpc-usage.md](claude-rpc-usage.md#projection-into-structured-data) and apply here as written. The one difference is `data_as_of`, recorded below.

`severity` and `is_active` from the flat `limits[]` records have no schema field here either, and this is what "diagnostic only" means concretely: a `severity` of `critical` on a parsed window adds one fixed sentence naming that window, and nothing else. `normal` adds nothing. `is_active` is not used at all — `claude_rpc` uses it to sharpen the wording of the same diagnostic, but on a cached snapshot the flag describes the moment the cache was written, not the moment of collection, so it is not reported. Neither field ever changes a projected value.

What differs from `claude_rpc` is freshness, and the rule is mandatory.

**Freshness:**

- `cachedUsageUtilization.fetchedAtMs` **must** be reported as `data_as_of` for every field derived from this cache.
- the cache is refreshed only when the user opens `/usage` in the Claude TUI. It is not refreshed by collection, by the CLI starting, or on any schedule, so it can be arbitrarily stale — days old on a machine where `/usage` was never opened.
- presenting a cached snapshot with a collection-time `data_as_of` is forbidden. It would claim live limits the source does not have.
- a snapshot with **no usable `fetchedAtMs`** is discarded whole — windows, credits, and spend alike — plus a diagnostic. It is not a missing decoration: without it there is no honest `data_as_of` for anything derived from the snapshot, and the only alternatives would be presenting cached values as live or publishing them with no age at all. The source falls back to the transcript reconstruction, which does have a defensible timestamp of its own.

**Amounts.** `amount_unit` is set to `usd` only on a window that actually reports at least one of `used_dollars`, `limit_dollars`, or `remaining_dollars` — the same conditional rule `claude_rpc` applies to the same payload, and for the same reason: on the verified Pro account every `*_dollars` field was `null`, so an unconditional `usd` would label a record that carries no amount at all. Percent-only windows leave `amount_unit` `null`. The reconstructed 5-hour limit is unaffected — its unit is `tokens`.

`cachedUsageUtilization.accountUuid` is not read.

### Where the staleness check happens

There are two guards, at two levels, and they do different things.

**Inside the source.** Before the cached windows become limit records, their automatic reset times are checked against collection time, with the same 2-minute grace `src/get_limits/freshness.rs` uses. One expired reset rejects the **whole** snapshot as a current-limit source, because all its percentages were captured together and a snapshot that is stale in one window is stale in all of them. The rejection is not fatal: the source falls back to the transcript-based 5-hour reconstruction and reports limits from there, with a diagnostic naming which of the two produced them. Everything outside the windows — the plan, the subscription start date, the token totals, the activity counts — is unaffected.

**Above the source, in the chain.** The guard described in [source-chains.md](../source-chains.md) then applies to whatever the source returned, and it is a different judgement: it decides whether this source's limit data is usable at all, or whether the chain should move on to the next source.

The order matters. Because the in-source check runs first, an expired cache degrades to reconstructed limits rather than to no limits, and the chain guard usually has a usable record to look at. Reading the chain-level guard as the only guard would mean an expired cache blanks the source's limits entirely — losing a reconstruction the source is perfectly able to produce.

### Cached limits versus reconstructed limits

When `cachedUsageUtilization` yields usable window records, they are the source's limit data: they are server-computed values with real reset times. The transcript-based 5-hour reconstruction below is the fallback for when the cache is absent, unusable, or stale, and a diagnostic records which of the two produced the reported limits.

## `~/.claude/stats-cache.json`

- `totalSessions` -> `usage.activity.sessions_count`
- `totalMessages` -> `usage.activity.turns_count`
- `modelUsage.<model>.inputTokens` -> `usage.tokens.input`
- `modelUsage.<model>.outputTokens` -> `usage.tokens.output`
- `modelUsage.<model>.cacheReadInputTokens` -> `usage.tokens.cache_read`
- `modelUsage.<model>.cacheCreationInputTokens` -> `usage.tokens.cache_write`
- `lastComputedDate` -> the **age** of these aggregates, reported in `diagnostics`

Rules for this read:

- token fields are summed across all models in `modelUsage`; `usage.tokens.total` is the sum of the four, the same way the transcript scan computes it. `usage.models.top_model` is the model with the largest total.
- the transcript scan stays authoritative for `usage.tokens` and the activity counts. `stats-cache.json` fills a field only when the scan produced no value for it, with a diagnostic naming the cache and how old its aggregates are.
- **the diagnostic reports a computed age, not the raw `lastComputedDate` string.** `lastComputedDate` is parsed and validated first; the diagnostic then states the elapsed time in a fixed shape such as `1d ago`. An unparseable value produces the same sentence without the age clause, and the offending string is never quoted. This follows what `src/providers/codex_local/auth.rs` does with `chatgpt_subscription_last_checked`, and for the same two reasons: a date the reader has to subtract from today is not an answer to "is this stale", and echoing a file's contents into a user-visible string is what the safety rules forbid. The cache is a lazily recomputed derivative of the same transcripts and lags the figures shown in the Claude UI; its one advantage is that it retains aggregates for transcripts that have since been deleted.
- `costUSD`, `webSearchRequests`, `contextWindow`, and `maxOutputTokens` in `modelUsage` have no schema field and are not projected. `dailyActivity`, `dailyModelTokens`, `hourCounts`, `longestSession`, and `firstSessionDate` are not read.
- a missing, unreadable, or unparseable file leaves these fields to the transcript scan; it is never an error state for the source.

## Activity counts

- `usage.activity.sessions_count` — the number of distinct sessions found by the transcript scan, else `totalSessions` from `stats-cache.json`
- `usage.activity.turns_count` — the number of assistant turns found by the scan, else `totalMessages`
- `usage.activity.latest_activity_at` — the latest transcript record timestamp
- `usage.activity.files_count` — **always `null`** for this source
- `usage.activity.events_count` — `null`; no local Claude record is an event count

`files_count` means the number of **changed user files**, the same as it does for `codex_local` and in the Usage output kind ([output-kinds.md](../../presentation/output-kinds.md)). No local Claude file records changed files, so the field has no source and stays `null`.

The number of scanned transcript files is not that number and must never be projected into it. It counts JSONL files the collector happened to open — it changes when transcripts are pruned or a new root appears, and it says nothing about the user's work. It stays in raw data as a scan metric. This is the identical defect that was found and removed in `codex_local`, where the scanned-file count had likewise been standing in for a changed-file count.

## macOS Keychain is not a source

Reading the Claude credentials from the macOS Keychain is **forbidden** for this source.

- the only useful field it holds, `subscriptionType`, is already available from `~/.claude.json` (`oauthAccount.organizationType`) and, live, from `claude_rpc`. There is nothing to gain.
- a Keychain read can raise an interactive GUI authorization dialog. In a headless or background collection run that hangs the collector on a prompt the user never sees.
- the form `security find-generic-password -g ...` prints the secret to stderr. It is forbidden in any variant, in any code path, including tests and diagnostics. No collection path may cause a credential to be written to any stream.

## 5-Hour Limit Reconstruction

`claude_local_usage` also reconstructs a single active 5-hour limit record from local transcripts, used when no usable cached snapshot is available:

- the numerator is `input_tokens + output_tokens` summed over turns in the active window
- the denominator is a fixed local estimate of `88,000` tokens for the Max5 plan; this is a community-derived approximation, not a value read from an official Claude API
- if a server reset anchor was found in local data (a reset timestamp nested under a rate-limit, usage-limit, quota, or 429 payload) and it is in the future, the window is `[anchor - 5h, anchor)` and the reset source is reported as `server reset anchor`
- otherwise the window is reconstructed from transcript timing: a new window starts at the first turn after the previous window elapsed or after a gap of 5 hours or more since the last turn, and the reset source is reported as `estimated reset`
- the resulting limit record includes `used_percent`, `remaining_percent`, `used_amount`, `remaining_amount`, `total_amount` (tokens), and `resets_at`
- because the `88,000` denominator is an approximation, reported usage can diverge from the account's actual 5-hour limit, especially at high usage

## Confirmed source limits

Verified absences in local Claude data, not merely unchecked gaps:

- no plan price, no currency for the plan, no billing period, no renewal date, and no billing or plan-management link. `account.price_amount`, `account.price_currency`, `account.price_period`, `account.price_note`, `account.renewal_at`, `account.plan_management_url`, and `account.billing_management_url` stay `null`, and public plan prices must never be hardcoded to fill them.
- no `usage.activity.files_count`: `stats-cache.json` counts sessions and messages only, and no local Claude file records changed user files. See [Activity counts](#activity-counts).

## Safety rules

Identical in kind to the rules in [claude-rpc-usage.md](claude-rpc-usage.md#safety-rules), and following the pattern of `src/providers/codex_local/auth.rs`:

- email, display name, organization name, account and organization UUIDs, referral codes, and any other identifier read from `~/.claude.json` never reach structured data, raw data, `diagnostics`, `status.message`, logs, or error text
- failures never carry source content: a missing file, unreadable file, unparseable JSON, absent key, or unparseable timestamp degrades to `null` plus a short fixed diagnostic literal
- all error paths return fixed literals; no message is built by formatting a value read from a local file
