# Codex CLI Usage

## Provider Method: `codex_cli_usage` (legacy)

This method is legacy. It is not part of any source chain and the system does not use it. The active Codex CLI-backed source is `codex_rpc_usage`, documented in [codex-rpc-usage.md](codex-rpc-usage.md).

Why it was replaced:

- TUI parsing is fragile by construction: the source is a rendered terminal stream with control sequences, and every value depends on current CLI wording and layout
- it needs a PTY (`expect`) and a TTY-shaped environment, plus a two-pass `/status` dance for a limit refresh
- `codex_rpc` returns strictly more: exact unix reset timestamps instead of rendered strings such as `16:44 on 8 Aug`, the plan tier as a protocol enum, a server-side lifetime token total, and the reset-credit records with type and expiry
- the RPC path answers in seconds with an empty `stderr` and exit code 0

The description below is kept so the path can be restored if the experimental `codex app-server` surface is removed from the CLI.

Code layout (`src/providers/codex_cli/`):

- `mod.rs` — thin public facade (`collect_usage`) and re-export of `build_structured`
- `process.rs` — PTY expect script and `codex login status` checks
- `parse.rs` — TUI line normalization and limit/credits/reset parsing
- `project.rs` — projection into `StructuredSourceInfo` / unavailable and authorization DTOs

Minimum commands:

- verify CLI availability: `command -v codex`
- verify CLI version: `codex --version`
- check authorization without starting the interactive CLI: `codex login status`
- official website: https://openai.com/codex
- CLI documentation: https://developers.openai.com/codex/cli

Verified PoC details:

- launches the standard `codex` command without a custom path to the CLI
- checks `codex login status` before launching the interactive CLI; when not authorized, returns the authorization problem state without opening a browser
- Codex CLI refuses to launch the interactive TUI if `stdin`/`stderr` are not TTYs
- for PoC, the system `expect` command is used as a minimal PTY adapter
- runtime sets `TERM=xterm-256color`, `COLUMNS=120`, `LINES=40` and runs `stty cols 120 rows 40`
- PoC sends `/status` via bracketed paste
- the first `/status` call sometimes triggers a limit refresh
- a second `/status` call returns the actual breakdown
- the parser waits for response indicators: startup screen, `refresh requested`, limit lines, or `Credits`
- user-facing output shows only the found summary: `5h limit`, `Weekly limit`, and `Credits`

### Plan name in the TUI dump

The `/status` dump contains an account line of the form `Account: <email> (Plus)`, so the plan tier is parsable from the TUI. It is deliberately not used: `codex_rpc` returns the tier as the `PlanType` enum from `account/read`, which needs no parsing and no assumption about the parenthesized suffix. The line also carries the account email, which must never leave the parsing layer under the rules in [codex-rpc-usage.md](codex-rpc-usage.md#safety-rules); another reason not to depend on it.

### Manual Limit Resets

Verified CLI behavior:

- `/usage` reports the number of available manual resets, for example `You have 1 usage limit reset available`;
The collector read `/usage` after `/status` in the same PTY session, then interrupted the CLI. It never confirmed or redeemed a reset. The source was the rendered `/usage` TUI stream; no local JSON object or array for these records was verified. The implementation extracted only `available_limit_resets`; it never entered the redemption path. The normalized field is defined in [structured-info.md](../structured-info.md), and its active source is now `rateLimitResetCredits.availableCount` from `codex_rpc`.
