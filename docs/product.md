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
