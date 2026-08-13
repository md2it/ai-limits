# PRODUCT

## Problem

AI spending is hard to control when usage is spread across multiple CLIs, models, and providers. API billing dashboards only help when requests go through API accounts, while subscription plans usually show quotas indirectly, inconsistently, or only inside vendor interfaces.

This creates several practical risks:

- Usage only becomes noticeable after hitting the limit
- Different providers use different quota rules and reset windows
- Token and request consumption is hard to compare across tools
- Paid overages or forced upgrades can happen before the user sees a trend
- No working free solution was found:
   - Most tools show API spending, not subscription plan usage
   - Too heavy
   - Too expensive
   - Require routing traffic through another vendor
   - Many simply do not work
   - Many are difficult to run and configure

## Target solution

A lightweight local tracker focused on AI usage through CLIs.

## Market comparison

The feature comparison is available in [`analogues.tsv`](analogues.tsv).

> Note: The table was populated by AI agents and cross-checked several times by other AI agents. Detailed manual verification by a human has been carried out only for a subset of the applications and their capabilities.

## Interfaces

The product ships as two interfaces sharing one core:

- a desktop app (macOS supported; Windows and Linux builds are being tested)
- a terminal CLI

## User capabilities

From the user's point of view, the system provides six core capabilities:

1. Get limits

   The user can see the current usage limits that apply to their AI tools, accounts, plans, or providers: Codex, Claude, and Cursor.

2. Get usage

   The user can see how much of the available limit has already been used for a selected tool or provider.

3. Get plan details

   The user can see whatever subscription/tariff context a provider exposes: plan name, subscription start date, and next renewal date. This gives the user a compact way to keep subscriptions under control without opening the provider's own billing page.

   Coverage is uneven and depends entirely on what each source publishes. Today Codex reports a plan name and both dates; Cursor reports a plan name, a renewal date, and a price; Claude reports a plan name and the subscription start date, but no renewal date. Price is exposed by Cursor only, and always with the disclaimer required by [get-limits/structured-info-rules.md](../spec/get-limits/structured-info-rules.md), because the amount can vary by country, currency, tax, and promotion.

4. Check access

   The user can verify whether the system has enough access to read the required usage and limit information from the relevant source.

5. Configure defaults and repeat checks

   The user can select visible providers, notifications, a refresh interval, and which output kinds are shown in the desktop app. The desktop application uses the CLI-first source chain; the terminal uses built-in defaults and explicit command-line options for a single query.

6. Receive notifications

   The user receives native macOS system notifications for remaining-limit thresholds and for an exact return to 100% available. Notifications are delivered through the desktop app.

Limits, plan, and usage are the product's three user-facing output kinds. They are defined, independent of any single interface, in [output-kinds.md](../spec/presentation/output-kinds.md).

Hard usage blocking, stopping usage automatically when a limit is reached, is a planned capability and not yet implemented.

## Business process

The product flow can be described as a business-readable process:

1. Get information from sources

   The system works with a defined set of information sources, one per provider and access method. Each source can have its own request format, access method, data location, and reliability constraints.

2. Normalize the information

   The system processes the raw information received from each source into a common, normalized form: available limits, used volume, reset periods, account context, provider context, and access status.

3. Provide user-facing results

   The system exposes the normalized information to the user as clear answers about limits and usage, in the desktop app and in the terminal.

4. Notify for limit events

   The system turns the normalized information into notification candidates when remaining limits are within defined thresholds or when a limit returns to exactly 100% available after a stored lower reading, and the macOS desktop app delivers them as native system notifications. Hard usage blocking will reuse the same normalized information once implemented.
