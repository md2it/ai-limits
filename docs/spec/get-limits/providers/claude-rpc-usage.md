# Claude RPC Usage

## Provider Method: `claude_rpc_usage`

`claude_rpc_usage` is the active Claude CLI-backed source. It replaces `claude_cli_usage` everywhere a Claude CLI source is used; `claude_cli_usage` stays documented as a legacy fallback path in [claude-cli-usage.md](claude-cli-usage.md).

Code layout (`src/providers/claude_rpc/`):

- `mod.rs` — thin public facade (`collect_usage`) and source identity constants
- `process.rs` — `claude` child process, control-protocol framing over stdio, and request/response correlation
- `parse.rs` — response DTOs and normalization of percents, windows, timestamps, and monetary amounts
- `project.rs` — projection into `StructuredSourceInfo` / unavailable and authorization DTOs

Minimum commands:

- verify CLI availability: `command -v claude`
- verify CLI version: `claude --version`
- start the source transport:
  `claude -p --verbose --input-format stream-json --output-format stream-json --no-session-persistence`
- official site: https://www.anthropic.com/claude-code
- CLI documentation: https://code.claude.com/docs/en/setup

## Transport

The source is not a dedicated subcommand. It is the SDK **control protocol** carried over stdio by the CLI's print mode: the process is started with the flags above, one JSON line is written to `stdin`, and one JSON line is read from `stdout`.

Request:

```json
{"type":"control_request","request_id":"1","request":{"subtype":"get_usage"}}
```

Response:

```json
{"type":"control_response","response":{"subtype":"success","request_id":"1","response":{ }}}
```

Verified on claude 2.1.220:

- no handshake is required: an `initialize` control request is **not** sent, and the control handler answers `get_usage` directly
- `--verbose` is **mandatory**, not cosmetic: without it the CLI refuses the flag combination outright, writing `Error: When using --print, --output-format=stream-json requires --verbose` to `stderr` and exiting with code `1`. No response line is written. For the source the outcome is the same either way — no data — but the failure is an explicit CLI error, not silence, and a diagnostic must not describe it as a hang or a timeout.
- no TTY is required: `stdin`/`stdout` are plain pipes and no PTY adapter (`expect`) is involved
- the process exits cleanly once `stdin` is closed
- the answer does **not** race EOF. Closing stdin immediately after writing the request still yields the response line; the trap found on `codex_rpc`, where the server exits on EOF before answering, does not reproduce here. The source nevertheless reads the response before closing stdin, so that neither CLI-backed transport depends on a behavior only one of them was verified to have.
- the exchange completes in 1.6–2.0 s
- the call consumes no account quota: the response reports `total_cost_usd` of `0` and no request to `/v1/messages` is made
- `--no-session-persistence` keeps the run from writing a transcript, so the source does not pollute the data that `claude_local` reads

The main risk of this source is the contract itself. The method is officially experimental: its schema description states that the response shape may change, and the TypeScript SDK deliberately names it `usage_EXPERIMENTAL_MAY_CHANGE_DO_NOT_RELY_ON_THIS_API_YET`. Unlike Codex, there is **no schema dump**: the shapes are compiled into the CLI binary as zod objects and no command emits them, so the contract below is what was observed on 2.1.220, verified by inspection of those objects, and nothing more.

The parser is therefore defensive by requirement:

- it reads only the keys documented here and ignores every other key, at every nesting level
- an unknown or changed shape degrades the affected fields to `null` plus a diagnostic; it never falls back to a guess, a positional read, or a "closest matching" key
- a missing top-level section leaves its whole field group `null` rather than partially filling it from another section

## Call sequence

The session is strictly read-only and always follows this order:

1. spawn `claude` with the flags above
2. write one `control_request` line with `request.subtype = "get_usage"`
3. read the matching `control_response` line, correlated by `request_id`
4. close stdin and let the process exit

No other control request is sent, no slash command is sent, no prompt is submitted, and nothing is written to the Claude account. Slash commands do not work in print mode at all, and `/usage-credits` must never be sent from any code path; see [claude-cli-usage.md](claude-cli-usage.md#forbidden-commands).

## Response shape

The `get_usage` payload has five top-level members: `session`, `subscription_type`, `rate_limits_available`, `rate_limits`, and `behaviors`.

### `subscription_type`

A string, from the enum `pro` | `max` | `team` | `enterprise`, or `null`.

`null` is a normal value, not a failure: API-key, Bedrock, and Vertex accounts have no Claude subscription. In that case `account.plan` stays `null` without a diagnostic claiming a parse problem.

### `rate_limits_available` and `rate_limits`

`rate_limits_available` is a boolean. When it is `false`, `rate_limits` is `null` and no limit records are produced.

When available, `rate_limits` is a direct pass-through of the server payload from `https://api.anthropic.com/api/oauth/usage`. It carries:

**Named rate-limit windows.** Each is either `null` or an object with `utilization` (0–100), `resets_at` (ISO 8601), `limit_dollars`, `used_dollars`, and `remaining_dollars`. Observed keys: `five_hour`, `seven_day`, `seven_day_oauth_apps`, `seven_day_opus`, `seven_day_sonnet`, `seven_day_cowork`, plus code-named entries such as `tangelo`, `iguana_necktie`, `omelette_promotional`, `nimbus_quill`, `cinder_cove`, and `amber_ladder`.

The key set is **open and demonstrably growing**: the live account carried `seven_day_omelette`, which appears in no earlier inventory of this payload. Any list of keys in this document, including the one above, is a record of what has been seen and never a contract. A parser that accepts keys because they are "in the list" is wrong by construction; only the explicitly named windows below are read, and everything else — known code name or not — is dropped at parse time.

`limit_dollars`, `used_dollars`, and `remaining_dollars` were `null` on the verified Pro account for every window that was present. Their shape is declared and their meaning is clear, but the populated case is covered by tests only; see [Not verified](#not-verified).

**`extra_usage`** — the paid overflow allowance: `is_enabled`, `monthly_limit`, `used_credits`, `utilization`, `currency`, `decimal_places`, `disabled_reason`, `user_disabled`, `spend_limit_reached`, `credits_ever_enabled`, `daily`, `weekly`. Amounts are integers in minor units; `decimal_places` gives the scale. `daily`, `weekly`, and `disabled_reason` are declared but not read; see [Members that are deliberately not read](#members-that-are-deliberately-not-read).

**`limits[]`** — a flat list of active limit states: `kind` (observed `session` and `weekly_all`), `group`, `percent`, `severity` (`normal` | `critical`), `resets_at`, `scope`, `is_active`. These records carry no absolute amounts.

**`spend`** — `used` (`amount_minor`, `currency`, `exponent`), `limit`, `percent`, `severity`, `enabled`, `disabled_reason`, `cap`, `balance`, `auto_reload`, `disclaimer`, `can_purchase_credits`, `can_toggle`. `disabled_reason`, `disclaimer`, and `auto_reload` are declared but not read; see below.

**`member_dashboard_available`** — a boolean.

The zod schema also declares `model_scoped[]` with `display_name`, `utilization`, and `resets_at`. It was **not** present in the observed response. It is not projected; if it appears, it is read into raw data only.

### Members that are deliberately not read

These are declared by the schema and are absent from the parsed model entirely, so they can reach neither structured data, nor raw data, nor diagnostics. Two different reasons apply.

Never observed, and the keys cannot be guessed:

- `session.model_usage` — a record keyed by model whose value shape was never seen, and which describes the collector's own empty process anyway
- `extra_usage.daily`, `extra_usage.weekly`
- `spend.auto_reload`

Observed or observable, but excluded on safety grounds — they are free-form backend copy that can name the account's organization or mislead:

- `extra_usage.disabled_reason` and `spend.disabled_reason` — the *state* of the allowance is what the projection reports; the backend's prose explanation for it is not published in any form
- `spend.disclaimer` — carries a support-article link, which is a help page and would be mistaken for a management link for the authenticated account

A member becoming observable later does not make it readable: adding one requires its shape to be verified first, and for the second group, a decision that the text is safe to publish.

### `session`

`total_cost_usd`, `total_api_duration_ms`, `total_duration_ms`, `total_lines_added`, `total_lines_removed`, and `model_usage` (a record keyed by model).

This is the usage of **the current CLI process**, not the account's history. For a one-shot `get_usage` call the session has done no work, so these values are empty or zero by construction. They are not account totals, and `session.*` is never projected into `usage.tokens`, `usage.money`, or `usage.activity`. Account-level token totals for Claude come from `claude_local` ([claude-local-usage.md](claude-local-usage.md)).

### `behaviors`

Two windows, `day` and `week`, each with `request_count`, `session_count`, `behaviors[]` (`key`, `pct`, `count`), `agents[]`, `skills[]`, `plugins[]`, and `mcp_servers[]`.

This is **not** server-side data. It is a local scan of the transcripts present on **this machine only**, over the given window, and the CLI itself treats it as approximate. It is not a count of the account's activity across machines and it is not comparable with a lifetime total. See the `sessions_count` decision below.

**Only `request_count` and `session_count` are read.** The rest of the block is not published anywhere, including raw data:

- `agents[]`, `skills[]`, `plugins[]`, and `mcp_servers[]` are the names of the user's own local agents, skills, plugins, and MCP servers. They describe the user's private tooling and are exactly the class of identifier the [Safety rules](#safety-rules) keep inside the parsing layer.
- `behaviors[].key` classifies how the user works. A working-habits profile is not usage data, and publishing it is not something raw data is for.

They are therefore not members of the parsed model at all, which is what makes the guarantee structural rather than a rule someone has to remember while writing a projection.

## Projection into structured data

Field names below are from [structured-info-schema.md](../structured-info-schema.md).

| Structured field | Source |
|---|---|
| `account.plan` | `subscription_type` |
| `account.credits_total` | `rate_limits.extra_usage.monthly_limit`, scaled by `decimal_places` |
| `account.credits_used` | `rate_limits.extra_usage.used_credits`, scaled by `decimal_places` |
| `account.credits_remaining` | calculated from the two above when both are present |
| `limits[].name`, `limits[].window_label` | the window key (`five_hour`, `seven_day`) |
| `limits[].window_minutes` | `300` for `five_hour`, `10080` for `seven_day` |
| `limits[].used_percent` | the window's `utilization`, else the matching `limits[].percent` |
| `limits[].remaining_percent` | calculated per [structured-info-rules.md](../structured-info-rules.md) |
| `limits[].resets_at` | the window's `resets_at`, else the matching `limits[].resets_at` |
| `limits[].used_amount` | `used_dollars` |
| `limits[].total_amount` | `limit_dollars` |
| `limits[].remaining_amount` | `remaining_dollars` |
| `limits[].amount_unit` | `usd`, only when the window reports at least one amount |
| `usage.money.used_amount` | `rate_limits.spend.used.amount_minor`, scaled by `exponent` |
| `usage.money.total_amount` | `rate_limits.spend.limit`, scaled the same way |
| `usage.money.remaining_amount` | calculated when both are present |
| `usage.money.currency` | `rate_limits.spend.used.currency` |
| `data_as_of` | the time the RPC response was received |
| `collected_at` | the time the collection run started |

Projection rules:

- **only explicitly named windows are parsed.** `five_hour` and `seven_day` produce limit records. The code-named entries — `tangelo`, `iguana_necktie`, `omelette_promotional`, `nimbus_quill`, `cinder_cove`, `amber_ladder`, `seven_day_omelette`, and any future sibling — are **not** parsed into `limits[]`: their semantics, window length, and applicability are unknown, and naming one in the UI would be exactly the weak assumption forbidden by [structured-info-rules.md](../structured-info-rules.md). The model- and surface-scoped windows `seven_day_opus`, `seven_day_sonnet`, `seven_day_oauth_apps`, and `seven_day_cowork` are likewise not projected: they are readable and documented, but they are not the account's headline quota and their meaning is not verified.
- **the two groups differ in what happens to them afterwards.** The four documented scoped windows are read and kept for raw data, alongside `model_scoped[]`. The code-named entries are dropped entirely, at parse time, and appear nowhere. This is forced by the defensive-parsing rule above: their key set is open — `seven_day_omelette` appeared on the live account after the inventory was written — so "keep the unparsed ones in raw data" would mean accepting arbitrary unknown keys from an experimental payload, which is precisely what the parser must not do. Keeping a fixed set of known keys and discarding the rest is the only form of the rule that can actually be implemented.
- **each window produces one record.** `rate_limits.limits[]` overlaps the named windows: `kind = "session"` describes the same window as `five_hour`, and `kind = "weekly_all"` the same window as `seven_day`. They are merged, not emitted twice. The named window is authoritative for amounts and `utilization`; the matching `limits[]` entry supplies `severity` and `is_active`, which have no schema field and are used only for diagnostics. A `limits[]` entry whose `kind` matches no parsed window is not projected.
- **a named window that is `null` still produces a record when its `limits[]` entry exists.** The two carry different facts, and the entry is not a partial copy of the window: `kind` identifies which window it describes, and the window's length is fixed by that identity, so `name`, `window_label`, and `window_minutes` are all still known. The record is emitted with `used_percent` from `percent` and `resets_at` from the entry, and with no amounts at all — those live only on the named window. A record is emitted only when at least one of percent, reset time, or an amount is known; a window and an entry that are both absent, or both empty, produce nothing.
- `amount_unit` is `usd` because the server names these fields `*_dollars`, and it is set only on records that actually carry an amount. A percent-only record — the normal case on the verified Pro account, where all three `*_dollars` fields were `null` — leaves `amount_unit` `null` rather than labelling amounts that do not exist. No currency conversion is ever performed. When `spend.used.currency` is not `USD` — `EUR` on the verified account — a diagnostic records that the limit amounts are dollar-named while the account's own spend is billed in another currency; the value is never re-labelled or converted to match.
- `account.credits_*` are monetary for Claude, unlike the credit balance Codex reports. The schema has no currency field beside them, so the currency travels in `usage.money.currency`. They are populated only when `extra_usage.is_enabled` is `true`; when the allowance is disabled, a `monthly_limit` is not an available balance and all three stay `null` plus a diagnostic carrying nothing but the disabled state.
- `usage.tokens.*` stays `null` for this source. The only token data in the response is `session.model_usage`, which describes the collector's own empty process. Claude token totals come from `claude_local`.
- `usage.activity.sessions_count` stays **`null`**, and a diagnostic records why. `behaviors.week.session_count` and `behaviors.day.session_count` exist, but they are a windowed, approximate, single-machine local scan, while the same field for every other source — including `claude_local` — is an exhaustive count of distinct sessions. Filling it here would make one field name mean two different things inside the same provider, and a chain that fell back from `claude_rpc` to `claude_local` would show the number jump for no user-visible reason. The two counts stay in raw data, where the window and the approximation are visible; the rest of `behaviors` is not published at all. The activity counts for Claude come from `claude_local`.
- `usage.activity.turns_count`, `files_count`, `events_count`, and `latest_activity_at` stay `null` for the same reason: `behaviors.*.request_count` is a windowed local estimate, not an account count.
- `usage.models.top_model` stays `null`. `session.model_usage` describes the collector's process, and `behaviors` reports agents, skills, plugins, and MCP servers rather than a model mix.
- `rate_limits_available = false` sets `status.access_available = true` and `status.data_available = false` with a fixed reason; it is not an error state and does not suppress `account.plan`.
- `raw_data_available` is `true`: the response can be exposed as raw data after the account identifiers are removed.

### What raw data contains

Raw data is a re-serialization of the **parsed** response, never a copy of the wire payload. Everything the parser drops is therefore absent from raw data too, and the two cannot drift apart. It contains:

- `subscription_type`, `rate_limits_available`, and the session totals other than `model_usage`
- the named windows `five_hour` and `seven_day`, the four documented scoped windows `seven_day_opus`, `seven_day_sonnet`, `seven_day_oauth_apps`, and `seven_day_cowork`, `model_scoped[]` if it ever appears, the flat `limits[]`, `extra_usage`, `spend`, and `member_dashboard_available`
- from `behaviors`, only `day.request_count`, `day.session_count`, `week.request_count`, and `week.session_count`

It does not contain code-named windows, the members listed under [Members that are deliberately not read](#members-that-are-deliberately-not-read), the `behaviors` name lists, or any account identifier.

### Freshness versus `claude_local`

This is the decisive difference between the two active Claude sources. `get_usage` performs a **fresh server request** on every call, so `data_as_of` is the response time and the limits are current.

`claude_local` reads `~/.claude.json` → `cachedUsageUtilization`, which is the same payload shape but is a **cache refreshed only when the user opens `/usage` in the TUI**. It can be arbitrarily stale, and its `fetchedAtMs` — not the collection time — is what `claude_local` must report as `data_as_of` ([claude-local-usage.md](claude-local-usage.md)).

## Safety rules

These follow the pattern already used by `src/providers/codex_local/auth.rs`:

- account identifiers never leave the parsing layer. Email, `organizationName`, `accountUuid`, `organizationUuid`, referral codes, and any other identifier present in the response are not projected, not written into raw data, not written into `diagnostics`, `status.message`, stdout, or stderr.
- failures never carry source content. A missing CLI, a process that exits without a response line, a protocol error, an unparseable response, an absent field, or a non-zero exit degrades to `null` plus a short fixed diagnostic literal. Response bodies, error payloads, and stderr text are never interpolated into user-visible strings.
- all error paths return fixed literals. There is no error message built by formatting a value read from the source.
- the child process is bounded: the source always closes stdin, enforces a timeout, and does not leave a `claude` process running after collection.
- the run is started with `--no-session-persistence` so that collection writes no transcript.

## Confirmed source limits

Verified absences in the `get_usage` payload, not merely unchecked gaps. The response contains no plan price, no billing period, no renewal date, and no plan- or billing-management link. Therefore, for `claude_rpc`:

- `account.price_amount`, `account.price_currency`, `account.price_period`, `account.price_note` — `null`
- `account.renewal_at`, `account.subscription_started_at` — `null`
- `account.plan_management_url`, `account.billing_management_url` — `null`

Public plan prices must never be hardcoded to fill these. `spend.disclaimer` contains a support-article link; it is a help page, not a management link for the authenticated account, and must not be used as one.

`account.subscription_started_at` for Claude comes from `claude_local` instead ([claude-local-usage.md](claude-local-usage.md)).

Also absent: any account-level token usage, and any account-level activity count.

### No authorization signal

The `get_usage` payload carries **nothing** that states whether the CLI is signed in. There is no counterpart to the `requiresOpenaiAuth` flag `codex_rpc` returns, and `rate_limits_available = false` is not one either: it says the account has no rate limits to report, which an authorized account can also say.

The consequence is a real product gap, and it is a confirmed absence rather than an unread field: an unauthorized run cannot be reported as "not signed in". It surfaces instead as one of the fixed transport literals — no response line, an unreadable response, or no supported data — and the user is told the source returned nothing rather than what to do about it. `claude_rpc` therefore never sets `status.cli_authorization`, unlike `codex_rpc`.

Closing this needs a signal from somewhere other than `get_usage`: a separate read-only CLI call whose output distinguishes signed-in from signed-out, verified in both states, and routed through the same fixed-literal message path. Until such a call is verified, guessing the state from an empty payload is forbidden — it would tell a signed-in user with a quiet account to log in again.

## Not verified

- behavior without network access and with an expired token. The expectation is `rate_limits_available = false` with `rate_limits = null`, but this is an assumption and was not reproduced.
- contract stability between CLI versions. The whole surface is verified on claude 2.1.220 only, the method is marked experimental by its own schema and SDK naming, and no schema dump exists to diff against.
- `model_scoped[]` — declared in the zod schema, absent from the observed response.
- populated window amounts. `limit_dollars`, `used_dollars`, and `remaining_dollars` were `null` on the verified Pro account, so `limits[].used_amount`, `total_amount`, `remaining_amount`, and the `usd` value of `amount_unit` are exercised by tests only and have never been produced from a live payload. The `null` path — percent-only records with `amount_unit` `null` — is the one that is verified.
- the unauthorized run. The behavior described under [No authorization signal](#no-authorization-signal) follows from the payload having no such field; which transport literal an actually unauthorized CLI produces was not reproduced.

Checked and ruled out:

- `claude gateway` — the enterprise component requires Postgres and its own configuration and has no relation to the current user's plan. A verified dead end.
- `claude mcp serve` — verified to expose no usage-related resources.
