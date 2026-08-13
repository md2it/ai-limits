# Codex RPC Usage

## Provider Method: `codex_rpc_usage`

`codex_rpc_usage` is the active Codex CLI-backed source. It replaces `codex_cli_usage` everywhere a Codex CLI source is used; `codex_cli_usage` stays documented as a legacy fallback path in [codex-cli-usage.md](codex-cli-usage.md).

Code layout (`src/providers/codex_rpc/`):

- `mod.rs` — thin public facade (`collect_usage`) and source identity constants
- `process.rs` — `codex app-server` child process, JSON-RPC framing over stdio, and request/response correlation
- `parse.rs` — response DTOs and normalization of percents, windows, timestamps, and credit balances
- `project.rs` — projection into `StructuredSourceInfo` / unavailable and authorization DTOs

Minimum commands:

- verify CLI availability: `command -v codex`
- verify CLI version: `codex --version`
- start the source transport: `codex app-server`
- regenerate the protocol schema for verification: `codex app-server generate-json-schema --out <DIR>` (writes `v2/*.json`)
- CLI documentation: https://developers.openai.com/codex/cli

## Transport

`codex app-server` speaks JSON-RPC 2.0 over stdio.

Verified on codex-cli 0.144.6:

- no TTY is required: `stdin`/`stdout` are plain pipes and no PTY adapter (`expect`) is involved
- the exchange completes in seconds, the process exits with code 0, and `stderr` stays empty
- the server exits on EOF, and it does so eagerly: closing stdin immediately after writing the messages makes it exit before answering, so no response line is ever read. Each response must be read while stdin is still open; see [Call sequence](#call-sequence).
- the protocol contract is machine-readable: `codex app-server generate-json-schema --out <DIR>` emits the request/response schemas under `v2/`
- the `app-server` command is marked `[experimental]` in the CLI. This is the main risk of this source: the method names and payload shapes documented here are the verified 0.144.6 contract, not a stability guarantee. A protocol change must degrade to `null` values plus diagnostics, never to guessed values.

## Call sequence

The session is strictly read-only and always follows this order:

1. request `initialize` with `params.clientInfo` (`name`, `version`)
2. send the `initialized` notification
3. request `account/read`
4. request `account/rateLimits/read`
5. request `account/usage/read`
6. close stdin and let the process exit

No other request is sent. Nothing is written to the Codex account, and no interactive session is started.

**Each response is read before stdin is closed.** This is a transport requirement, not a style choice: closing stdin right after writing all the messages makes the server exit on EOF *before* it has answered, and the source degrades to a transport failure with no data. Stdin stays open until the last response line has been read, and is closed only in step 6.

### `account/read`

Returns the account context:

- `account.type` — account kind, `"chatgpt"` on the verified account
- `account.planType` — plan tier, enum `PlanType`. The 0.144.6 schema declares `free`, `go`, `plus`, `pro`, `prolite`, `team`, `business`, `enterprise`, `edu`, `self_serve_business_usage_based`, `enterprise_cbp_usage_based`, and `unknown`. The list is open: a tier the schema does not yet declare is passed through as reported, exactly like a declared one, and only the literal `unknown` is treated as "not a plan name".
- `account.email` — the account email
- `requiresOpenaiAuth` — a boolean whose name does not mean "a login is required now"

`requiresOpenaiAuth` is `true` on a **fully authorized** account: it was verified as `true` on the verified machine while `account` was populated and every limit read succeeded. The obvious reading of the name is therefore wrong, and the flag alone must never drive an authorization message. The source reports the authorization state only when the account cannot be read **and** the flag is set; on its own, the flag is ignored.

`account.email` is never read into the internal model, never projected, and never written to raw data, structured data, diagnostics, or any message. See [Safety rules](#safety-rules).

### `account/rateLimits/read`

Returns `rateLimits` and `rateLimitsByLimitId`. The source reads `rateLimitsByLimitId`, keyed by limit id, and uses the `"codex"` entry. Each entry carries:

- `limitId`, `limitName`, `planType`. `limitName` is `null` on the verified account even though it is a declared member; it is an optional label, not a guaranteed one.
- `primary` — `usedPercent`, `windowDurationMins`, `resetsAt` (unix seconds). `usedPercent` is `int32` in the schema and is used exactly as reported: no rounding, no rescaling, and no clamping are applied to it.
- `secondary` — same shape as `primary`
- `credits` — `hasCredits`, `unlimited`, `balance` (a high-precision decimal string)
- `individualLimit`, `rateLimitReachedType`

The same response carries `rateLimitResetCredits`, the manually redeemable limit resets:

- `availableCount` — the number of resets available
- records with `id`, `resetType`, `status`, `grantedAt`, `expiresAt`, `title`, `description`

### `account/usage/read`

Returns server-side usage:

- `summary` — `lifetimeTokens`, `peakDailyTokens`, `longestRunningTurnSec`, `currentStreakDays`, `longestStreakDays`
- `dailyUsageBuckets[]` — `startDate` (`YYYY-MM-DD`) and `tokens`

## Projection into structured data

Field names below are from [structured-info-schema.md](../structured-info-schema.md).

| Structured field | Source |
|---|---|
| `account.plan` | `account/read` → `account.planType` |
| `account.credits_remaining` | `account/rateLimits/read` → `rateLimitsByLimitId["codex"].credits.balance` |
| `limits[].used_percent` | `primary.usedPercent` / `secondary.usedPercent` |
| `limits[].remaining_percent` | calculated from `used_percent` per [structured-info-rules.md](../structured-info-rules.md) |
| `limits[].resets_at` | `primary.resetsAt` / `secondary.resetsAt` (unix seconds) |
| `limits[].window_minutes` | `primary.windowDurationMins` / `secondary.windowDurationMins` |
| `limits[].name` | `limitName`, else the fixed literal `primary` / `secondary` |
| `limits[].window_label` | `limitName`, `null` when it is absent |
| `available_limit_resets` | `rateLimitResetCredits.availableCount` |
| `usage.tokens.total` | `account/usage/read` → `summary.lifetimeTokens` |
| `data_as_of` | the time the RPC responses were received |
| `collected_at` | the time the collection run started |

Projection rules:

- `primary` and `secondary` produce two separate `limits[]` records, told apart by `window_minutes` and `resets_at`. A missing `secondary` produces one record, not a record with `null` values.
- **the naming of those records does not depend on `limitName`.** `name` is required by the schema, and `limitName` is `null` on the verified account, so `name` falls back to the fixed literals `primary` and `secondary` — the same vocabulary `codex_local` uses for the same two windows. `window_label` is optional and simply stays `null`; nothing is invented to fill it. Surfaces lose nothing by it: the window label they render comes from `window_minutes` first, so a 10080-minute window is displayed as `7d` whether or not the server named it. When `limitName` is present it is used for both fields.
- `resets_at` is a numeric unix timestamp from the server and is normalized to ISO 8601 UTC (`YYYY-MM-DDTHH:MM:SSZ`). This is strictly better than the legacy TUI path, which could only recover a rendered local string such as `16:44 on 8 Aug`; no date reconstruction or timezone guessing is involved.
- `limits[].used_amount`, `remaining_amount`, `total_amount`, and `amount_unit` stay `null`: the source reports percentages and windows, not absolute quota sizes.
- `usage.tokens.total` is the server-reported `lifetimeTokens`. It is not a locally computed sum and must not be reconciled with, replaced by, or added to the `codex_local` token totals.
- the remaining `usage.tokens.*` fields stay `null`: the RPC reports a single lifetime total with no input/output/cache breakdown.
- `usage.activity.latest_activity_at` stays `null`. `dailyUsageBuckets[].startDate` is date-granular and is not an activity timestamp; deriving one from it would be the weak assumption forbidden by [structured-info-rules.md](../structured-info-rules.md).
- `account.plan` uses `planType` as reported. The enum value `unknown` is not a plan name: it leaves `account.plan` `null` plus a diagnostic.
- `account.credits_remaining` is parsed from the `balance` string. When `credits.unlimited` is `true`, the balance is not a remaining amount and the field stays `null` plus a diagnostic. When `balance` cannot be parsed as a number, the field stays `null` plus a diagnostic; the string is never emitted as a number-shaped guess.
- `account.credits_total` and `account.credits_used` stay `null`: the source reports a balance only.
- `available_limit_resets` is taken from `availableCount` as reported. The `rateLimitResetCredits` records — `resetType`, `status`, `grantedAt`, `expiresAt` — are read for raw data and diagnostics, where expiry and reset type come from; the count is never recomputed from them, because a record's status semantics are not part of the verified contract.
- `individualLimit` and `rateLimitReachedType` have no field in the schema. They stay in raw data; `rateLimitReachedType` may inform a diagnostic and nothing else.
- `raw_data_available` is `true`: the JSON-RPC responses can be exposed as raw data after the email is removed.

### What raw data contains

Raw data is a re-serialization of the parsed responses, never a copy of the wire payload, so it is defined by what the parser reads:

- all three responses are present, each degrading on its own; a response that could not be read is absent rather than partial.
- `account.email` is not a member of the parsed model at all, so it cannot appear.
- the top-level `rateLimits` member of `account/rateLimits/read` is kept. It is the backward-compatible single-bucket view of the same data the source reads from `rateLimitsByLimitId`, so raw data carries the figures twice: once keyed by limit id and once without any limit identifier. It is retained deliberately, as the legacy shape a future protocol change may fall back to, and no structured field is ever derived from it.
- for the reset-credit records, `resetType`, `status`, `grantedAt`, and `expiresAt` are kept; `id`, `title`, and `description` are excluded — an opaque backend identifier and backend copy, neither of which may be published.
- `individualLimit` and `rateLimitReachedType` are kept, as stated above.

## Forbidden method

`account/rateLimitResetCredit/consume` **must never be called.** It spends one of the account's limit resets and is an irreversible write on the user's account. It is not part of the call sequence, it must not appear in the code, and no code path — retry, fallback, diagnostic, or test — may reach it. Reset credits are read-only for this product: the source reports how many exist, and redeeming them stays a manual user action in Codex itself.

## Safety rules

These follow the pattern already used by `src/providers/codex_local/auth.rs`:

- `account.email` and any other account identifier never leave the parsing layer. They are not projected, not written into raw data, not written into `diagnostics`, `status.message`, stdout, or stderr.
- failures never carry source content. A missing CLI, a failed `initialize`, a protocol error, an unparseable response, an absent field, or a non-zero exit degrades to `null` plus a short fixed diagnostic literal. Response bodies, error payloads, and stderr text are never interpolated into user-visible strings.
- an account that reports `requiresOpenaiAuth` **and** cannot be read sets `status.access_available = false` and `status.data_available = false` with the authorization state as `status.message`, in the same shape the legacy CLI path used for `codex login status`. Both conditions are required: the flag is `true` on a fully authorized account, so acting on it alone would tell an authorized user to log in.
- the child process is bounded: the source always closes stdin and does not leave `codex app-server` running after collection.

## Confirmed source limits

These are verified absences in the 0.144.6 protocol schema, not gaps that are merely unchecked. The whole generated schema contains no `price`, `currency`, `billing`, `renew`, `invoice`, or `subscription` key, and the only URL keys concern authentication, not plan management. Therefore, for `codex_rpc`:

- `account.price_amount`, `account.price_currency`, `account.price_period`, `account.price_note` — `null`
- `account.renewal_at`, `account.subscription_started_at` — `null`
- `account.plan_management_url`, `account.billing_management_url` — `null`

Public plan prices must not be hardcoded to fill these. Subscription dates for Codex come from the local auth token instead, documented in [codex-local-usage.md](codex-local-usage.md).

Also absent: absolute quota sizes (`used_amount`/`total_amount`), per-token-kind usage breakdown, and per-session or per-file activity counts. Activity counts for Codex come from `codex_local`.

## Not verified

- `account/workspaceMessages/read` — present in the protocol, not called and not inspected; its payload and whether it carries anything relevant are unknown
- contract stability between codex-cli versions — the whole surface is verified on 0.144.6 only, and the command is marked experimental
