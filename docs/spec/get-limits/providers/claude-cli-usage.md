# Claude CLI Usage

## Provider Method: `claude_cli_usage` (legacy)

This method is legacy. It is not part of any source chain and the system does not use it. The active Claude CLI-backed source is `claude_rpc_usage`, documented in [claude-rpc-usage.md](claude-rpc-usage.md).

Why it was replaced:

- TUI parsing is fragile by construction: the source is a rendered terminal stream, and every value depends on current CLI wording and layout
- it needs a PTY (`expect`) and a TTY-shaped environment, and the request/parse cycle takes noticeable time
- `claude_rpc` returns strictly more: the plan tier, exact ISO reset timestamps, absolute window amounts in dollars, the extra-usage allowance, and the spend state — in 1.6–2.0 s, over plain pipes, with no PTY

The description below is kept so the path can be restored if the experimental `get_usage` control request is removed from the CLI.

Minimum commands:

- check that the CLI is installed: `command -v claude`
- check CLI version: `claude --version`
- official site: https://www.anthropic.com/claude-code
- CLI documentation: https://code.claude.com/docs/en/setup

Verified PoC details:

- the standard `claude` command is run with the `--no-chrome` flag to avoid opening the additional Chrome integration dialog
- `/usage` is used to retrieve limits
- `/status` opens the Status tab by default without limits
- the PoC waits for the prompt to be ready based on the bottom line `for shortcuts`
- `/usage` is sent as regular input without bracketed paste
- user-facing output shows the matched lines `Current session`, `Current week`, `Total cost`, and token usage
- structured limits map `Current session` to a 5-hour window (`window_minutes = 300`) and `Current week` to a 7-day window (`window_minutes = 10080`)
- the parser accounts for some lines arriving via bare carriage return, so cleaned/compacted output is split on `\n` and `\r`

Code layout under `src/providers/claude_cli/`:

- `mod.rs` — public facade (`get_usage` / `collect_usage`) and source identity constants
- `capture.rs` — expect script and process capture (`capture_provider_run`)
- `parse.rs` — TUI line normalization (including CR/LF) and parsing into an internal model
- `project.rs` — projection to `SourceData` / `StructuredSourceInfo`

## Plan name without the TUI

`claude auth status --json` reports `subscriptionType` as machine-readable JSON, without starting the TUI and without a PTY. It is deliberately not used: `get_usage` already returns `subscription_type` in the same response as the limits, so a second process launch would add latency and a second contract to maintain for a field the active source already provides.

## Forbidden commands

- **`/usage-credits` must never be sent.** It starts an interactive login flow. In an automated, non-interactive context this either hangs the collector or drives the user's account through an unattended authentication path. It must not appear in any expect script, retry path, fallback, diagnostic, or test.
- slash commands do not work in print mode (`-p`) at all, so there is no "safe headless" variant of the TUI path. Anything reached by sending a slash command requires a PTY session, which is the reason this whole method is legacy.
