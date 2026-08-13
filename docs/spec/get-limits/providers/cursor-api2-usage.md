# Cursor API2 Usage

## Provider Method: `cursor_api2_usage`

`cursor_api2_usage` is the only Cursor source. It reads plan, limit, and usage data from Cursor's internal dashboard backend `api2.cursor.sh` with the access token created by `cursor agent login`.

Code layout (`src/providers/cursor_api2/`):

- `mod.rs` — thin public facade (`collect_usage`) and re-export of `build_source_data`
- `fetch.rs` — Keychain token, the whole call sequence over `infra/os_access` (five methods for an individual account, up to seven with the team branch), event-page paging, and the page cap
- `parse.rs` — path-based reads of the named responses into the internal `CursorFields` model, page accumulation, and raw-data sanitization
- `helpers.rs` — private price, date, amount, and percentage helpers for projection
- `project.rs` — projection into `SourceData` (limits, billing, money) and package tests

## Transport

Every call is Connect-RPC over JSON:

- `POST https://api2.cursor.sh/aiserver.v1.DashboardService/<Method>`
- headers `Authorization: Bearer <token>`, `Content-Type: application/json`, `Connect-Protocol-Version: 1`
- request body is JSON; an empty request is `{}`

All methods used by this source are `Get*` methods and are semantically read-only. Nothing is written to the Cursor account.

The endpoint is a stable internal Cursor endpoint used by Cursor itself. It has no publicly documented contract: the method names and payload shapes recorded here are what was verified against a live individual Pro account, not a stability guarantee. A contract change must degrade to `null` values plus diagnostics, never to guessed values.

## Why HTTP is the only path

There is no local Cursor source. This is a confirmed architectural property of Cursor, not an unchecked gap:

- `cursor-agent` exposes an undocumented `acp` command (Agent Client Protocol, JSON-RPC over stdio). It works, but by ACP specification it carries only `session/*`, `fs/*`, and `terminal/*` methods. Account, subscription, and limit data are not part of that protocol.
- the `cursor-agent-svc` daemon exposes only `health`, `register`, `heartbeat`, `drain`, `flush`, `snapshot`, and `updateAuth`.
- Cursor.app itself calls the same `api2.cursor.sh` backend.
- no local listening socket carries account data, and `state.vscdb` holds no plan cache.

`cursor-agent about --format json` is not a local source either: it performs network calls to `GetMe` and `GetPlanInfo`, so as a plan source it duplicates `GetPlanInfo` and is not used.

`cursor-agent status --format json` is genuinely local and returns identity and auth state only. It may be used as an optional cheap pre-check for "is the user logged in" before any network call. It is an optimization, not a data source: it reports no plan, no price, no limits, and no usage.

## Call sequence

The sequence is strictly read-only and always follows this order:

1. `GetPlanInfo` — body `{}`
2. `GetCurrentPeriodUsage` — body `{}`
3. `GetHardLimit` — body `{}`
4. `GetAggregatedUsageEvents` — body `{"teamId":0,"startDate":<ms>,"endDate":<ms>}`
5. `GetFilteredUsageEvents` — body `{"teamId":0,"startDate":<ms>,"endDate":<ms>,"page":<n>,"pageSize":1000}`, repeated per page

Steps 4 and 5 use the current billing cycle window: `startDate` and `endDate` are `billingCycleStart` and `billingCycleEnd` from `GetCurrentPeriodUsage`. Date filtering is verified to work; a call without dates returned the same figures as the current-cycle window, but the source always passes the window explicitly rather than relying on that default.

A failure of any step after the first degrades only the fields that step feeds. A failed `GetAggregatedUsageEvents` leaves `usage.tokens.*` `null` plus a diagnostic and does not invalidate plan or limit data.

**A failed first step is different in kind.** `GetPlanInfo` is the source's access probe: it is the simplest call, with an empty body, and it is where a missing, expired, or rejected token shows up. When it fails, the run stops there and the whole source reports denied access — `status.access_available = false` with a fixed literal telling the user to run `cursor agent login` if the token was rejected, and a generic transport literal otherwise. No further method is attempted. Continuing would mean issuing four more calls that are certain to fail the same way, and reporting an empty result as if the account genuinely had no data.

### Paging `GetFilteredUsageEvents`

- pages are numbered **from 1** — verified; page 0 is not used
- paging stops when the collected event count equals `totalUsageEventsCount` from the first page, when a page returns no events, or at the page cap
- the page cap is **60**; at `pageSize` 1000 that bounds one run at 60,000 events
- the page set counts as **complete** only when `totalUsageEventsCount` was read, the collected count equals it, no page failed, no page was unreadable, and the cap was not reached
- an incomplete page set leaves `usage.activity.sessions_count` `null` plus a diagnostic, per [Activity](#activity). `events_count` and `turns_count` still carry `totalUsageEventsCount`, which the server reported for the whole window and which paging cannot understate.

### `GetPlanInfo`

Returns `planInfo` and `nextUpgrade`.

- `planInfo.planName` — plan tier name, `"Pro"` on the verified account
- `planInfo.price` — a **string** carrying amount, currency symbol, and period together, `"$20/mo"` on the verified account. There is no separate currency field anywhere in the response; the currency exists only as a symbol inside this string. Per the proto, this field is optional.
- `planInfo.includedAmountCents` — integer, the monetary allowance included in the plan
- `planInfo.billingCycleEnd` — epoch milliseconds as a string; optional per the proto
- `planInfo.planOwner` — enum `PLAN_OWNER_UNSPECIFIED` | `PLAN_OWNER_STRIPE` | `PLAN_OWNER_APPLE`
- `nextUpgrade` — `tier`, `name`, `includedAmountCents`, `price`, `description` for the next plan up

`nextUpgrade` describes a plan the user is not on. It has no field in the schema, is never projected into `account.*`, and must never be mistaken for the current plan's price.

### `GetCurrentPeriodUsage`

The method already called by the source. Fields used:

- `planUsage.totalPercentUsed`, `planUsage.autoPercentUsed`, `planUsage.apiPercentUsed`
- `planUsage.includedSpend`, `planUsage.limit` — both in cents
- `billingCycleStart`, `billingCycleEnd` — epoch milliseconds
- `spendLimitUsage.individualUsed`, `individualLimit`, `pooledLimit`, `limitType`

`planUsage.remaining` does **not** exist in the current response. This is a confirmed fact about the response shape, not an unread field: an earlier parser read that key and it is simply not there any more. The remaining allowance is derived from `includedSpend` and `limit` instead.

`spendLimitUsage.individualUsed`, `individualLimit`, and `pooledLimit` did not appear on the verified account at all, so the whole on-demand spend window is unobserved end to end; see [Not verified](#not-verified).

### `GetHardLimit`

Body `{}`. Returns the on-demand spend ceiling:

- `noUsageBasedAllowed` — a **negative** flag, as its name states: `true` means usage-based spend beyond the plan allowance is *not* permitted. The source reads it as such and inverts it internally into "usage-based spend is allowed"; the live account agrees with that reading.
- `hardLimit` — the ceiling amount, of unstated scale. It was **not present** in the verified response; see [Not verified](#not-verified).

### `GetAggregatedUsageEvents`

Body `{"teamId":0,"startDate":<ms>,"endDate":<ms>}`. Returns totals for the window:

- `totalInputTokens`, `totalOutputTokens`, `totalCacheWriteTokens`, `totalCacheReadTokens`
- `totalCostCents`
- `percentOfBurstUsed`, `totalRequestCost` — optional
- `aggregations[]` — `modelIntent`, `inputTokens`, `outputTokens`, `cacheWriteTokens`, `cacheReadTokens`, `totalCents`, `requestCost`, `tier`

### `GetFilteredUsageEvents`

Body `{"teamId":0,"startDate":<ms>,"endDate":<ms>,"page":<n>,"pageSize":1000}`. Returns:

- `totalUsageEventsCount` — the number of billable usage events in the window
- `usageEventsDisplay[]` — `timestamp`, `model`, `kind`, `maxMode`, `requestsCosts`, `usageBasedCosts`, `isTokenBasedCall`, `tokenUsage{inputTokens, outputTokens, cacheReadTokens, totalCents}`, `owningUser`, `cursorTokenFee`, `isChargeable`, `serviceAccountId`, `isHeadless`, `chargedCents`, `conversationId`, `subscriptionProductId`

`owningUser` and `serviceAccountId` are account identifiers and are handled under [Safety rules](#safety-rules).

## Projection into structured data

Field names below are from [structured-info-schema.md](../structured-info-schema.md).

| Structured field | Source |
|---|---|
| `account.plan` | `GetPlanInfo` → `planInfo.planName` |
| `account.price_amount`, `account.price_currency`, `account.price_period` | parsed from the `planInfo.price` string |
| `account.price_note` | fixed disclaimer literal, see [Price](#price) |
| `account.renewal_at` | `GetPlanInfo` → `planInfo.billingCycleEnd` (epoch ms) |
| `limits[]` percent windows | `GetCurrentPeriodUsage` → `planUsage.totalPercentUsed`, `autoPercentUsed`, `apiPercentUsed` |
| `limits[]` monetary window | `GetCurrentPeriodUsage` → `planUsage.includedSpend` / `planUsage.limit`, `amount_unit` `usd` |
| `limits[]` on-demand window | `GetHardLimit` → `hardLimit` with `spendLimitUsage.individualUsed`, `amount_unit` `usd` |
| `limits[].resets_at` | `GetCurrentPeriodUsage` → `billingCycleEnd` |
| `usage.tokens.input` | `GetAggregatedUsageEvents` → `totalInputTokens` |
| `usage.tokens.output` | `GetAggregatedUsageEvents` → `totalOutputTokens` |
| `usage.tokens.cache_read` | `GetAggregatedUsageEvents` → `totalCacheReadTokens` |
| `usage.tokens.cache_write` | `GetAggregatedUsageEvents` → `totalCacheWriteTokens` |
| `usage.tokens.total` | sum of the four token fields above |
| `usage.activity.turns_count` | `GetFilteredUsageEvents` → `totalUsageEventsCount` |
| `usage.activity.sessions_count` | count of distinct `conversationId` across all pages of `usageEventsDisplay[]` |
| `usage.activity.latest_activity_at` | maximum `timestamp` in `usageEventsDisplay[]` |
| `usage.activity.events_count` | `GetFilteredUsageEvents` → `totalUsageEventsCount` |
| `usage.money.used_amount`, `total_amount`, `remaining_amount` | `GetCurrentPeriodUsage` → `planUsage.includedSpend` / `planUsage.limit`, converted from cents, remainder derived |
| `usage.money.currency` | the fixed code `USD`, set only when a total amount is known |
| `data_as_of` | the time the responses were received |
| `collected_at` | the time the collection run started |

### Price

`planInfo.price` is one string that packs amount, currency, and period. The parser accepts only the exact shape it actually understands:

- a **required** leading currency symbol from an explicit symbol-to-code table (`$`, `€`, `£`, `¥`, `₹`)
- a decimal amount consisting of digits with at most one decimal point
- a `/` separator followed by a period token that the schema recognizes (`mo`, `yr`)
- no other characters, before, between, or after

`account.price_period` may be populated here because the source states the period explicitly in the string — the `/mo` suffix is source data, not an inference from the amount, the plan name, or the distance between two dates. That is what [structured-info-rules.md](../structured-info-rules.md) requires before this field may be filled.

The symbol is required because the currency exists nowhere else in the response: a string such as `20/mo` states an amount with no currency at all, and there is no second field to recover it from. Taking the amount alone would leave `price_amount` populated with `price_currency` `null`, which is the partial parse the rule below forbids.

Parsing is all-or-nothing. If the string does not match the accepted shape — a missing or unknown currency symbol, an unknown period token, a missing amount, a range, a localized or annotated form, surrounding whitespace, an empty or absent field — then `price_amount`, `price_currency`, and `price_period` all stay `null` and a short diagnostic is added. Partial parsing is forbidden: an amount is never taken without its currency, and a currency is never taken without its amount. A public plan price must never be hardcoded to fill the gap; the value comes only from the source response for the account being read.

`account.price_note` is always filled when a price is reported. Cursor does not state that the returned string is the price every user on that plan pays — it can vary by country, currency, tax, and promotion — so the disclaimer rule in [structured-info-rules.md](../structured-info-rules.md) applies. The note is a fixed short literal; it never interpolates response content. Surfaces are free to render the disclaimer as a `≈` sign instead of the note text; the desktop card does exactly that, see [desktop/ui/provider-block-content.md](../../desktop/ui/provider-block-content.md).

### Tokens

`usage.tokens.total` is the sum of `totalInputTokens`, `totalOutputTokens`, `totalCacheReadTokens`, and `totalCacheWriteTokens`. It is a computed sum, not a server-reported total: the response carries no total token field.

If any of the four is absent or unparseable, the sum is not formed. `total` stays `null` plus a diagnostic, and the components that were read keep their own values. A sum over an unknown subset would understate the figure, which is the weak assumption forbidden by [structured-info-rules.md](../structured-info-rules.md).

`usage.tokens.cached_input` and `usage.tokens.reasoning_output` stay `null`: the response has no such breakdown, and `cache_read` is not the same fact as `cached_input`.

### Activity

`usage.activity.turns_count` is `totalUsageEventsCount`, which is the number of **billable usage events** in the billing cycle, not the number of conversational turns shown in the Cursor interface. One is not a proxy for the other: non-chargeable interactions do not appear, and a single interface turn may produce several events. The field is used because it is the closest honest match in the schema, and the semantics are recorded here so that neither the implementation nor the card claims something the source does not report.

`usage.activity.sessions_count` is the number of distinct `conversationId` values. It therefore requires every page of `GetFilteredUsageEvents` to be retrieved. If paging is incomplete for any reason — a failed page, a page cap, a truncated response — the field stays `null` plus a diagnostic. A count over the pages that happened to arrive is an understated number presented as a fact, and is not allowed.

`usage.activity.latest_activity_at` is the maximum `timestamp` across the retrieved events. It is a business fact about user activity and is not used as the `Source {source}` timestamp; `data_as_of` is the response time.

`usage.activity.events_count` carries the same `totalUsageEventsCount` value, which is literally what that figure is. `turns_count` repeats it because the target field set asks every source for a turn count and this is the closest the source has; the duplication is intentional and is not a sign that one of the two was derived.

`usage.activity.files_count` stays `null`, see [Confirmed source limits](#confirmed-source-limits).

`usage.models.top_model` stays `null`. `aggregations[].modelIntent` groups events by intent, and it is not established that its value is a model name; treating it as one would be a weak assumption. The field stays in raw data.

### Money

`usage.money` carries the plan's included monetary allowance: `planUsage.includedSpend` as used, `planUsage.limit` as total, both converted from cents, and the remainder derived from the two. The currency is the fixed code `USD` and is set only when a total is known — the response states no currency anywhere outside the price string, so a bare amount is never labelled.

These are the same two figures as the `included_spend` limit record. The duplication is intentional: the record answers "how much of the allowance is left", the `usage.money` block answers "how much has this cost", and surfaces select one or the other. Neither is derived from the other.

`rate_limits.spend`-style account balances have no counterpart here; Cursor reports no separate spend state beyond the allowance and the on-demand ceiling.

### Limits

Records are emitted with these fixed names, in this order, each only when its source values are present:

| `limits[].name` | Content |
|---|---|
| `plan_usage` | `planUsage.totalPercentUsed`, with the billing cycle as `window_label` |
| `auto` | `planUsage.autoPercentUsed` |
| `api_models` | `planUsage.apiPercentUsed` |
| `included_spend` | `planUsage.includedSpend` / `planUsage.limit` in USD, with percentages derived from the two |
| `on_demand_spend` | `hardLimit` as total and `spendLimitUsage.individualUsed` as used, in USD |

The names are fixed literals chosen by this source, not strings taken from the response; the response names no window. `window_minutes` is `null` for all of them: a billing cycle is not a fixed-length window, and its start and end are already reported as `window_label` and `resets_at`.

The on-demand ceiling from `GetHardLimit` and the plan allowance from `planUsage` are monetary, not percentage-based. They are projected with `amount_unit` `usd` and amounts filled from `used_amount` / `total_amount`, with `remaining_percent` derived per the limit rules. They are not converted into an invented percentage scale of their own, and they are not merged with the percent windows from `planUsage`.

The `on_demand_spend` record is emitted only when usage-based spend is permitted — that is, when `noUsageBasedAllowed` is `false`. When it is `true`, or when the flag is absent, there is no on-demand window to show and no record is emitted for it.

**The scale of `hardLimit` is not verified.** The response states no unit for it, and the field did not appear at all on the verified account, so no live value was ever available to compare against a known ceiling. The source treats it as whole dollars — the same unit the ceiling is expressed in throughout Cursor's own interface — and attaches a diagnostic to every `on_demand_spend` record stating that the scale is unverified. If the value turns out to be cents, the record is wrong by a factor of 100, which is exactly why the diagnostic travels with it rather than being recorded here only. Note that `planUsage.includedSpend` and `planUsage.limit` are in cents, so the two amount families in this source do **not** share a scale.

`limits[].resets_at` and `account.renewal_at` both come from a live response describing the current period, so no past-date guard is applied; this is the same boundary `src/get_limits/freshness.rs` draws when it rejects expired resets for local sources only.

## Enterprise and team branch

The official client branches the same way, and the implementation must follow it, or the source stays silent on team and enterprise accounts:

1. if `planUsage` in `GetCurrentPeriodUsage` is empty, the individual path does not apply
2. read `GetMe` → `isEnterpriseUser` and `teamId`
3. call `GetMonthlyBillingCycle` (`startDateEpochMillis`, `endDateEpochMillis`) for the window
4. call `GetAggregatedUsageEvents` for that window, with the real `teamId`

This branch is derived from the official client bundle only. It has **not** been exercised against a live team or enterprise account, and every field it depends on is unverified until it has been. Until then it must fail into `null` plus diagnostics rather than into fabricated figures.

## Parse by path

Every value is read from a named path inside a named response — `GetPlanInfo` → `planInfo.price`, `GetCurrentPeriodUsage` → `planUsage.limit`, and so on. Locating a value by scanning the whole body for a key name is forbidden.

The reason is that these payloads reuse key names across unrelated objects: `limit` occurs in `planUsage`, in `spendLimitUsage`, and per aggregation; `price` occurs in both `planInfo` and `nextUpgrade`. A whole-body search returns whichever one happens to come first in the current response, which makes the value depend on key order rather than on meaning — and would quietly report the next plan's price as the current one.

A path that is absent yields `null` plus a diagnostic. It never falls back to a same-named key found elsewhere in the body.

## What raw data contains

Raw data is an object keyed by method name — `GetPlanInfo`, `GetCurrentPeriodUsage`, `GetHardLimit`, `GetAggregatedUsageEvents`, and `GetFilteredUsageEvents` as an array of page bodies. A method that failed or returned an unparseable body is absent rather than partial. When nothing could be collected, `raw_data_available` is `false`.

`GetMe` is **never** included, in whole or in part. It is called only in the team branch, its useful content is the team identity, and the rest of it is account identity; there is no subset of it worth publishing.

Every included body passes through one recursive sanitizer before it can be published:

- a key is dropped, with its entire subtree, when its lowercased name ends with `id` or `ids`, or contains `email`, `user`, `auth`, `secret`, or `password`, or ends with `token` — but **not** `tokens`, so the token counters `inputTokens`, `outputTokens`, `totalCacheReadTokens`, and the rest survive
- any remaining string value that has the shape of an email address becomes `null`, which catches identifiers carried inside free-form backend text rather than in a named field
- everything else passes through unchanged

The rule is deliberately over-broad: it removes `conversationId`, `owningUser`, `serviceAccountId`, `teamId`, and `subscriptionProductId` without needing a list of them, and it will remove identifier-shaped keys that Cursor adds later without this document being updated first. The cost is that a harmless key matching one of those patterns is also removed; that trade is accepted, because raw data is a diagnostic aid and account identifiers must not leave the machine.

## Confirmed source limits

These are verified absences, not unchecked gaps.

- `account.plan_management_url` and `account.billing_management_url` stay `null`. The only portal method is `GetTeamCustomerPortalUrl`, which requires a `team_id` and returned `401 unauthenticated` for an individual account. The application bundle contains static constants such as a `cursor.com/dashboard` usage-tab URL and a `cursor.com/settings` URL, but a constant compiled into a client is not a link the source issued for the authenticated account — it is exactly the generic provider URL that [structured-info-rules.md](../structured-info-rules.md) forbids constructing a management link from. This question is settled: these fields are `null` until Cursor returns an account-specific portal link.
- `account.subscription_started_at` stays `null`. No method reports when the current plan began. `GetMe.createdAt` is the account creation date, which is a different fact and must not be substituted.
- `usage.activity.files_count` stays `null`. No usage message carries a file counter.
- there is no currency field. Currency exists only as a symbol inside the `planInfo.price` string, and `account.price_currency` is populated only when that string parses as a whole.

## Not verified

- `GetCurrentBillingCycle` returns `startDateEpochMillis` and `endDateEpochMillis` that **differ** from `billingCycleStart` / `billingCycleEnd` in `GetCurrentPeriodUsage`. The reason for the discrepancy is unknown, so the method is not used for `account.renewal_at`; `GetPlanInfo.billingCycleEnd` is used instead, and it matched `GetCurrentPeriodUsage.billingCycleEnd` exactly. Until the semantics are established, no field may be derived from `GetCurrentBillingCycle`.
- the enterprise/team branch above, verified by client code reading only.
- **the whole on-demand spend window.** `hardLimit` was absent from the `GetHardLimit` response, and `spendLimitUsage.individualUsed`, `individualLimit`, and `pooledLimit` were absent from `GetCurrentPeriodUsage`. Not one of the values that would populate the `on_demand_spend` record was observed, so its amounts are unexercised outside tests, and the scale it is built on is an assumption, not a reading. The dollars-versus-cents question is unanswerable from the verified data and stays open until a live account returns a ceiling.
- `planInfo.planOwner`, `spendLimitUsage.limitType`, `percentOfBurstUsed`, `totalRequestCost`, and `aggregations[].tier` are read into raw data; their exact semantics are not established and no structured field is derived from them.
- contract stability across Cursor versions. The surface here was verified against `cursor-agent` 2026.07.23-e383d2b and a live individual Pro account.

## Safety rules

- the access token is read from the macOS Keychain and passed to `curl` as a config on stdin (`curl -K -`), so it never appears in `argv`, in a process listing, in a shell history, or in an environment variable. It is never written to raw data, `diagnostics`, `status.message`, stdout, or stderr, and never interpolated into an error message.
- `owningUser`, `serviceAccountId`, and any account email or identifier never leave the parsing layer. `conversationId` is used only to count distinct values and is never projected or emitted.
- failures never carry source content. A missing token, a rejected token, a transport error, an unparseable response, or an absent field degrades to `null` plus a short fixed diagnostic literal. Response bodies and error payloads are never interpolated into user-visible strings.
- no token value, email, or identifier appears in this document or any other document, including in examples.
- `raw_data_available` is `true`: responses can be exposed as raw data after account identifiers are removed.
