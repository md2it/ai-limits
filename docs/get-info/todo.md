# Get Info TODO

Status: implemented in `claude_local_usage`; remaining items are validation gaps that require real Claude CLI/web/desktop observations.

## Problem

Claude local transcript scanning reconstructs token usage but does not reliably provide official reset timestamps. The current local reset estimate can diverge from Claude CLI, web, and desktop values.

## Goal

Improve `claude_local_usage` reset accuracy when local files contain or can be linked to prior server reset signals.

## Tasks

- Done: extended `claude_local_usage` server-reset anchoring beyond fixed transcript reset paths.
- Done: added coverage for nested rate-limit, usage-limit, quota, and 429 reset-bearing records when present in local JSONL.
- Done: exposed the discovered server reset anchor in raw `claude_local_usage` output as `usage.latest_server_reset_anchor`.
- Pending validation: compare local reset reconstruction against observed Claude CLI, web, and desktop reset timestamps.
- Done: documented remaining Claude Desktop and browser-extension local-file gaps separately from Claude Code statusline behavior.

## Related Documents

- [providers/claude.md](providers/claude.md) — Claude provider methods and local reconstruction notes.
- [methods/statusline.md](methods/statusline.md) — statusline runtime capture model.
- [methods/README.md](methods/README.md) — method taxonomy and selection principles.
- [methods/from-local-files.md](methods/from-local-files.md) — static local-file retrieval model.
