# Structured Info Schema

## Structured format

Structured data should use one common field contract for all providers and sources.

The current minimum structure is described below in a YAML-like schema. The field names, nesting, and meanings are mandatory. The final serialization format may be selected by the implementation as long as it remains machine-readable and preserves this structure.

```yaml
provider: string
source: string
source_link: docs/spec/get-limits
status:
  data_available: boolean
  access_available: boolean
  message: string | null
raw_data_available: boolean
collected_at: string | null
data_as_of: string | null
account:
  plan: string | null
  credits_total: number | null
  credits_used: number | null
  credits_remaining: number | null
  subscription_started_at: string | null
  renewal_at: string | null
  price_amount: number | null
  price_currency: string | null
  price_period: string | null
  price_note: string | null
  plan_management_url: string | null
  billing_management_url: string | null
limits:
  - name: string
    window_label: string | null
    window_minutes: number | null
    resets_at: string | null
    used_percent: number | null
    remaining_percent: number | null
    used_amount: number | null
    remaining_amount: number | null
    total_amount: number | null
    amount_unit: string | null
usage:
  tokens:
    input: number | null
    cached_input: number | null
    output: number | null
    reasoning_output: number | null
    cache_read: number | null
    cache_write: number | null
    total: number | null
  money:
    used_amount: number | null
    remaining_amount: number | null
    total_amount: number | null
    currency: string | null
  activity:
    events_count: number | null
    files_count: number | null
    sessions_count: number | null
    turns_count: number | null
    latest_activity_at: string | null
  models:
    top_model: string | null
available_limit_resets: number | null
diagnostics:
  - string
```

`available_limit_resets` is the count of manually redeemable resets of provider limits. It is separate from `limits[].resets_at`, which is the automatic reset time of a rate-limit window, and from `usage`, which records consumed tokens, money, and activity.

For Codex, the source is `rateLimitResetCredits.availableCount` from the read-only `account/rateLimits/read` method of `codex_rpc` ([providers/codex-rpc-usage.md](providers/codex-rpc-usage.md)). `usage` is Codex's own command and UI terminology for the same records; `available_limit_resets` is the ai-limits product field. The legacy `codex_cli` path read the count from the rendered `/usage` TUI stream instead. Sources that do not expose a manual reset count must return `available_limit_resets: null`.

The Rust structured model serializes `available_limit_resets`. Reading the count never redeems a reset; the redeeming RPC method is forbidden.

`account.subscription_started_at`, `account.renewal_at`, `account.price_amount`, `account.price_currency`, `account.price_period`, `account.price_note`, `account.plan_management_url`, and `account.billing_management_url` are the subscription fields that back the product's **Plan** output kind, defined in [output-kinds.md](../presentation/output-kinds.md). Population and disclaimer rules for these fields are documented in [structured-info-rules.md](structured-info-rules.md).

`account.price_period` is the billing period the price applies to, as a short lowercase token: `mo`, `yr`, or another period the source states explicitly. It exists so that a price can be displayed as a rate rather than a bare amount; without it, `20.00` is ambiguous between a monthly and an annual charge. It must never be assumed — a source that reports a price without stating its period leaves this `null`.
