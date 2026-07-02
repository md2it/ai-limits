# Get Info TODO

## Problem

Claude local transcript scanning reconstructs token usage but does not reliably provide official reset timestamps. The current local reset estimate can diverge from Claude CLI, web, and desktop values.

## Goal

Improve `claude_local_usage` reset accuracy when local files contain or can be linked to prior server reset signals.

## Tasks

- Extend `claude_local_usage` server-reset anchoring beyond reset fields already embedded in scanned transcript JSONL.
- Check whether Claude local files contain CLI `/usage`, 429 payloads, or other reset-bearing records not covered by the current parser.
- Persist or expose the discovered server reset anchor in the raw `claude_local_usage` output for diagnostics.
- Validate local reset reconstruction against observed Claude CLI, web, and desktop reset timestamps.
- Document remaining Claude Desktop and browser-extension local-file gaps separately from Claude Code statusline behavior.

## Related Documents

- [providers/claude.md](providers/claude.md) — Claude provider methods and local reconstruction notes.
- [methods/statusline.md](methods/statusline.md) — statusline runtime capture model.
- [methods/README.md](methods/README.md) — method taxonomy and selection principles.
- [methods/from-local-files.md](methods/from-local-files.md) — static local-file retrieval model.
