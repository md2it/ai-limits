# Codex

## Current status

PoC retrieves limits via Codex CLI. The application launches the standard `codex` command, sends `/status` to the interactive TUI, and extracts limit lines from the output.

---

## Provider Method: `codex_cli_status`

Minimum commands:

- verify CLI availability: `command -v codex`
- verify CLI version: `codex --version`
- official website: https://openai.com/codex
- CLI documentation: https://developers.openai.com/codex/cli

Verified PoC details:

- launches the standard `codex` command without a custom path to the CLI
- Codex CLI refuses to launch the interactive TUI if `stdin`/`stderr` are not TTYs
- for PoC, the system `expect` command is used as a minimal PTY adapter
- runtime sets `TERM=xterm-256color`, `COLUMNS=120`, `LINES=40` and runs `stty cols 120 rows 40`
- PoC sends `/status` via bracketed paste
- the first `/status` call sometimes triggers a limit refresh
- a second `/status` call returns the actual breakdown
- the parser waits for response indicators: startup screen, `refresh requested`, limit lines, or `Credits`
- user-facing output shows only the found summary: `5h limit`, `Weekly limit`, and `Credits`

---

## Limitations

- full output remains a TUI stream and may contain terminal control sequences
- the approach depends on the current CLI behavior and TUI text
- CLI requests can take a noticeable amount of time
- needs verification of whether such requests consume user limits

---

## Other options

| Option | Status | Comment |
|---|---|---|
| Official API | Not investigated | Requires separate verification of usage/limits availability for a Codex subscription |
| Local telemetry files | Candidate for usage history | By analogy with `ccusage`, Codex usage can be read from `${CODEX_HOME:-~/.codex}`; confirms tokens/cost/models, but not subscription limit/reset |
| Frontend/dashboard API | Research-only | Possible only with a clear and safe approach to session data |
| Traffic observation | Research-only | Do not consider as a product mechanism |
