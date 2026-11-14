# Cursor

## Source contract

The app retrieves Cursor plan, limit, and usage data from authenticated `DashboardService` methods on `api2.cursor.sh`. An active Cursor CLI sign-in is required: install Cursor CLI and run `cursor agent login`. These calls collect account data and do not invoke models. The endpoint is internal and has no publicly documented stability guarantee.

Credential discovery, account continuity, background token refresh, platform validation, and recovery behavior are defined in [cursor-auth.md](cursor-auth.md). Routine token expiry is recovered before reporting a source failure. The implementation gap and verification evidence are recorded in [cursor-options.md](cursor-options.md#verification-and-implementation-gap).

---

## Provider Method: `cursor_api2_usage`

The method reads several read-only `DashboardService` methods over Connect-RPC and projects them into structured data:

- `GetPlanInfo` — plan name, price string, included allowance, billing cycle end
- `GetCurrentPeriodUsage` — plan usage percentages, included spend, billing cycle, spend-limit usage
- `GetHardLimit` — the on-demand spend ceiling
- `GetAggregatedUsageEvents` — token totals for the billing cycle
- `GetFilteredUsageEvents` — billable usage events, used for activity counts

It fills `account.plan`, the price fields, `account.renewal_at`, the percentage and monetary `limits[]`, the token breakdown and total, and the activity counts. `account.plan_management_url`, `account.billing_management_url`, `account.subscription_started_at`, and `usage.activity.files_count` stay `null` as confirmed source limits, not as unread gaps.

The full call sequence, field mapping, parsing rules, confirmed limits, and safety rules are specified in [cursor-api2-usage.md](cursor-api2-usage.md).

Code lives in `src/providers/cursor_api2/`:

- `mod.rs` — thin public facade (`collect_usage`) and re-export of `build_source_data`
- `credentials.rs` — platform credential discovery, read-only system sqlite3 and CLI store reads
- `session.rs` — account continuity, bounded session recovery, and in-memory credentials
- `transport.rs` — allowlisted dashboard/session HTTP transport and response classification
- `fetch.rs` — dashboard call sequence and event-page paging
- `parse.rs` — path-based reads into the internal `CursorFields` model, page accumulation, and raw-data sanitization
- `helpers.rs` — private price, date, amount, and percentage helpers for projection
- `project.rs` — projection into `SourceData` (limits, billing, money) and package tests

Changes to session recovery require the credential-safety and live validation checks in [cursor-auth.md](cursor-auth.md#acceptance-checks).

Other known retrieval options are documented in [cursor-options.md](cursor-options.md).
