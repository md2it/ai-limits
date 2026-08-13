# Getting Limits

## Purpose

This document describes options for retrieving usage/limits from AI providers and serves as an entry point into provider-specific documentation.

The product goal is to show the user current limits locally with minimal setup. Different providers expose data through different channels, so for each provider we document several possible approaches: primary, fallback, and research options.

## Terms

- Usage — current consumption for the period.
- Limit — available quota for the plan or included usage.
- Reset — when the period resets.
- Quota/rate limit — technical API or service constraints.
- Provider method — a specific channel for retrieving data for a provider.

## Method selection principles

- Prefer the official, documented approach when it is available to the user without materially degrading UX.
- For a minimal scenario, prioritize an already installed and authorized local tool.
- Do not extract cookies, session tokens, or refresh tokens without explicit user consent and a separate threat model.
- Do not treat unofficial endpoints as a stable public contract.
- For each provider method, document data quality: which fields are available, how accurate they are, whether reset is included, and how often data can be refreshed.
- Implement fallback only if it improves the user scenario and does not disproportionately increase security/ToS risk.

## Provider fallback chains

Source chain order, usable-data rules, and interface mappings are defined in [../source-chains.md](../source-chains.md).

## Related documents

- [from-provider-cli.md](from-provider-cli.md) — technical model for provider methods that retrieve data via the provider CLI/TUI.
- [from-local-files.md](from-local-files.md) — technical model for provider methods that retrieve data from local transcript/telemetry files.
- [../providers/codex.md](../providers/codex.md) — ways to retrieve Codex limits.
- [../providers/claude.md](../providers/claude.md) — ways to retrieve Claude limits.
- [../providers/cursor.md](../providers/cursor.md) — ways to retrieve Cursor limits.
- [retrieval-options.md](retrieval-options.md) — data retrieval options and current status by provider.
