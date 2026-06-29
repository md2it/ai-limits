# Claude

## Current Status

The PoC retrieves usage/limits via the Claude CLI. The application runs the standard `claude` command, opens the interactive TUI, and sends the slash command `/usage`.

---

## Provider Method: `claude_cli_usage`

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
- the parser accounts for some lines arriving via bare carriage return, so cleaned/compacted output is split on `\n` and `\r`

---

## Limitations

- full output remains a TUI stream
- the approach depends on current Claude CLI behavior and TUI text
- a CLI request may take a noticeable amount of time
- needs verification of whether such a request consumes user limits

---

## Other Options

| Option | Status | Comment |
|---|---|---|
| Official API | Not investigated | May apply to API accounts, but not necessarily to Claude Code subscription limits |
| Local transcript JSONL | Candidate for usage history | Check `~/.config/claude/projects/**/*.jsonl`, `~/.claude/projects/**/*.jsonl`, and Xcode ClaudeAgentConfig; good for tokens/cost/sessions, but not always for official remaining limit |
| Claude Code statusline `rate_limits` | Candidate for live limits | Hook receives JSON via stdin from Claude Code and can provide an official live signal for 5h/7d limits; requires statusline configuration |
| Local SQLite/cache | Auxiliary layer | e.g. `~/.claude/usage.db` from `claude-usage`: convenient for dashboard and incremental scanning, but this is derived data, not a primary source |
| Frontend/dashboard API | Research-only | Possible only with a clear and safe way to handle cookie/session tokens |
| Traffic observation | Research-only | Not to be considered as a product mechanism |
