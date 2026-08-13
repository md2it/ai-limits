# Cursor

## Current status

The app retrieves Cursor plan, limit, and usage data through the stable internal endpoint `api2.cursor.sh`, which Cursor itself uses, and an access token created by `cursor agent login`. The endpoint has no publicly documented contract. This is the only implemented Cursor source, and it is the only possible one: Cursor exposes no local protocol carrying account, subscription, or limit data, and the IDE itself calls the same backend.

If the token is not found, the request is rejected, or the response format has changed, the source reports the failure as unavailable data.

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
- `fetch.rs` — Keychain token, the call sequence via `infra/os_access`, and event-page paging
- `parse.rs` — path-based reads into the internal `CursorFields` model, page accumulation, and raw-data sanitization
- `helpers.rs` — private price, date, amount, and percentage helpers for projection
- `project.rs` — projection into `SourceData` (limits, billing, money) and package tests

The endpoint is Cursor's stable internal endpoint, whose contract is not publicly documented, and it requires a separate security review before production use.

Other known retrieval options are documented in [cursor-options.md](cursor-options.md).
