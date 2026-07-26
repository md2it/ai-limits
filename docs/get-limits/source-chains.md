# Source Chains

This document is the source of truth for provider source order.

A source chain is an ordered list of provider methods. The app tries the next method when the current method does not provide usable limit data as defined in [data-validation.md](data-validation.md).

Which interface mode uses which chain is documented in [source-chain-mapping.md](source-chain-mapping.md).

## Chains

### `fast_free` "fast"

Fast local/provider-native chain. It avoids provider CLI checks.

```text
Codex: codex_local
Claude: claude_local
Cursor: cursor_api2
```

### `cli_fallback` "full"

Local/provider-native chain with CLI fallback for Codex and Claude.

```text
Codex: codex_local -> codex_cli
Claude: claude_local -> claude_cli
Cursor: cursor_api2
```

### `cli_first` "best"

CLI-first chain for more accurate and current Codex and Claude data. CLI checks may take longer.

```text
Codex: codex_cli -> codex_local
Claude: claude_cli -> claude_local
Cursor: cursor_api2
```

## Snapshot Reuse Compatibility

Snapshot reuse does not change source-chain order. A fresh cached result may satisfy a scheduled refresh only when its source is compatible with the selected chain:

- Codex and Claude `fast` or `full` may reuse a local or CLI snapshot
- Codex and Claude `best` may reuse only a CLI snapshot
- Cursor may reuse only an API2 snapshot

A CLI snapshot may satisfy a request that would otherwise start from a local source. A local snapshot does not satisfy a CLI-first request.

Background Agent uses `best`, so its Codex and Claude checks may reuse only CLI snapshots. Snapshot age and Tauri timer behavior are documented in [../snapshot-store.md](../snapshot-store.md), and background timing is documented in [../background-agent.md](../background-agent.md).
