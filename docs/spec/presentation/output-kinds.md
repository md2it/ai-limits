# User-Facing Output Kinds

## Purpose

This document defines the output kinds the product presents to the user, independent of which interface renders them and independent of how the underlying data was collected.

It applies to every current and future interface built on the shared core: the desktop app, the terminal CLI, macOS widgets, the menu bar, and any interface added later. It is not scoped to any single interface's implementation.

## The three output kinds

The system presents normalized structured data (see [get-limits/structured-info.md](../get-limits/structured-info.md)) to the user as three output kinds:

1. **Limits** — the remaining and used share of each rate-limit window that applies to a provider, account, or plan, with its reset time. Sourced from `limits[]` and `available_limit_resets`.
2. **Plan** — the user's subscription/tariff context: which plan they are on, when it started, when it renews, and what it costs. Sourced from `account`.
3. **Usage** — how much of the provider's tracked consumption has been used, in whatever shape that provider reports it (tokens, money, activity, model mix). Sourced from `usage`.

These are the three highest-value ways to answer "where do I stand with this provider," in that order of everyday usefulness: limits tell the user if they are about to be blocked, plan tells the user what they are paying for and when it changes, usage gives supporting detail.

## Cross-interface expectation

Any interface may implement any subset of these three output kinds, and may implement none of them for a given provider if the provider's structured data does not support it. Showing all three is not a requirement placed on every interface.

What is fixed is the categorization itself: an interface that shows subscription context does so as **Plan**, an interface that shows consumption detail does so as **Usage**, and an interface that shows quota headroom does so as **Limits**. Interfaces do not invent a fourth category or blend these three into an undifferentiated feed, so that the same mental model transfers between the terminal, the desktop app, and any interface added later.

The Tauri desktop app implements Limits and Plan side by side, one section per provider block, and deliberately shows no Usage; the terminal CLI is where Usage output lives, under `--usage`. The desktop layout, rendering rules, and visibility toggles are documented in [desktop/ui/provider-blocks.md](../desktop/ui/provider-blocks.md), [desktop/ui/provider-block-content.md](../desktop/ui/provider-block-content.md), and [desktop/ui/settings.md](../desktop/ui/settings.md).

## Plan output: goal and content

The goal of the Plan output is to give the user a compact, at-a-glance way to keep subscriptions under control, without needing to open the provider's own billing page.

Plan output answers three questions, in this order of usefulness: what am I on, when am I next charged, and where do I change it.

- the tariff/plan name and what it costs
- when it next renews
- where to manage the plan and the billing

Price is always presented as approximate. A plan's real cost varies by country, currency, tax, and promotional terms, and the product reads what a source reports rather than what a given user is actually billed. Interfaces signal this on the price itself — the desktop card uses a `≈` sign — instead of claiming an exact figure. `account.price_note` in [get-limits/structured-info-schema.md](../get-limits/structured-info-schema.md) carries the long-form explanation for surfaces that have room for it.

When the current subscription started is collected but is not part of the standard displayed set. It is the least actionable of the subscription facts for day-to-day cost control, and interfaces with a tight space budget omit it.

A renewal date is only shown when it is still in the future. Sources that read a cached local credential can carry a subscription window that has already elapsed; per the "no weak assumptions" rule in [get-limits/structured-info-rules.md](../get-limits/structured-info-rules.md), an elapsed renewal date is reported as unknown with a diagnostic rather than presented as an upcoming charge.

## Usage output: goal and content

Sources report consumption in different shapes, but the user's question is the same everywhere: how much have I done through this provider. Usage output answers it with one standard set of metrics, in a fixed order, so the same figure sits in the same place for every provider and can be compared at a glance:

- total tokens
- sessions
- turns
- files

Every provider and source aims to supply these; see the target field set in [get-limits/structured-info-rules.md](../get-limits/structured-info-rules.md). Whatever a source cannot supply is simply absent, not padded or substituted.

Sources expose far more than this — token breakdowns, cached-input ratios, monetary spend, event counts, model mixes, per-project attribution. All of it is collected into structured data regardless of what any interface displays, and is available in raw and structured output.

Usage is the least time-critical of the three kinds: it describes what has already been spent rather than what is left, so an interface built around "can I keep working right now" gains little from it. This is why the desktop card omits it entirely (see [desktop/ui/provider-block-content.md](../desktop/ui/provider-block-content.md#usage-is-not-shown)) and the terminal keeps it behind an explicit `--usage` flag rather than in default output; the terminal's row format is documented in [terminal/usage-block-format.md](../terminal/usage-block-format.md).

## Coverage is uneven by design

Each source exposes a different subset, and partial coverage is the normal case rather than an error state. A value that is absent produces no line, no placeholder, and no dash — an interface shows what is known and stays silent about the rest.

This is why the output kinds are defined as categories rather than as fixed layouts: a card showing three subscription lines for one provider and none for another is behaving correctly, and an interface must look deliberate in both cases.
