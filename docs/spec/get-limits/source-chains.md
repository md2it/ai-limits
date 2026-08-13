# Source Chains

This document is the source of truth for provider source order.

A source chain is an ordered list of provider methods. The app tries the next method only when the current method does not provide usable limit data.

Usable limit data means:

- `access_available = true`
- `data_available = true`
- at least one limit record is present
- for local Codex and Claude snapshots, no reliably parsed automatic reset time is more than two minutes in the past

Which interface mode uses which chain is documented in [source-chain-mapping.md](source-chain-mapping.md).

If a local Codex or Claude snapshot contains an expired automatic reset time, the whole current-limit snapshot is rejected because all limit percentages were captured together. Historical usage remains source data, but the stale snapshot does not stop fallback. If no fallback succeeds, the source reports `Local provider data is outdated`.

If no source in a chain returns usable limit data and an installed provider CLI requires authorization, report the authorization state instead of a generic no-data result. A usable result from any source still takes priority over authorization.

## Chains

### `fast_free`

Fast local/provider-native chain. It avoids provider CLI checks.

```text
Codex: codex_local
Claude: claude_local
Cursor: cursor_api2
```

### `cli_fallback`

Local/provider-native chain with CLI fallback for Codex and Claude. The CLI-backed sources are `codex_rpc` and `claude_rpc`; the legacy `codex_cli` and `claude_cli` TUI sources are not part of any chain.

```text
Codex: codex_local -> codex_rpc
Claude: claude_local -> claude_rpc
Cursor: cursor_api2
```

### `cli_first`

CLI-first chain for more accurate and current Codex and Claude data. CLI checks may take longer.

```text
Codex: codex_rpc -> codex_local
Claude: claude_rpc -> claude_local
Cursor: cursor_api2
```
