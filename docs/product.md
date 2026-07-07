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

## Interfaces

The product ships as two interfaces sharing one core:

- a desktop app (macOS available, Windows and Linux in beta)
- a terminal CLI

## User capabilities

From the user's point of view, the system provides five core capabilities:

1. Get limits

   The user can see the current usage limits that apply to their AI tools, accounts, plans, or providers: Codex, Claude, and Cursor.

2. Get usage

   The user can see how much of the available limit has already been used for a selected tool or provider. This is currently available only in the terminal, not in the desktop app.

3. Check access

   The user can verify whether the system has enough access to read the required usage and limit information from the relevant source.

4. Configure defaults and repeat checks

   The user can set default sources and a check interval. In the terminal this is done through a config file or a repeat flag; the desktop app exposes the same settings in a user-friendly form.

5. Receive notifications

   The user receives native system notifications when remaining limits cross defined thresholds. Notifications are delivered through the desktop app; the terminal can request delivery from an installed, running desktop app.

Hard usage blocking, stopping usage automatically when a limit is reached, is a planned capability and not yet implemented.

## Business process

The product flow can be described as a business-readable process:

1. Get information from sources

   The system works with a defined set of information sources, one per provider and access method. Each source can have its own request format, access method, data location, and reliability constraints.

2. Normalize the information

   The system processes the raw information received from each source into a common, normalized form: available limits, used volume, reset periods, account context, provider context, and access status.

3. Provide user-facing results

   The system exposes the normalized information to the user as clear answers about limits and usage, in the desktop app and in the terminal.

4. Notify on threshold crossings

   The system turns the normalized information into notification candidates when remaining limits cross defined thresholds, and the desktop app delivers them as native system notifications. Hard usage blocking will reuse the same normalized information once implemented.
