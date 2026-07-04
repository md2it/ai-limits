# Claude Code statusline setup for ai-limits

Give this prompt to Claude Code once on the machine where `ai-limits` is installed:

```text
Configure Claude Code statusLine.command so it runs:

ai-limits --claude-statusline

Use the correct Claude Code settings file for this machine. It is usually one of:

- ~/.claude/settings.json
- ~/.config/claude/settings.json

Do not remove existing settings. If a statusLine command already exists, preserve it when possible or explain the conflict before changing anything.

Also create the ai-limits statusline cache directory and marker file:

mkdir -p ~/.config/ai-limits
touch ~/.config/ai-limits/claude-statusline.json

The file may be empty immediately after setup. An empty file means the statusline integration is configured, but Claude Code has not yet sent a statusline payload with rate_limits.

After the setup, I will send one normal Claude Code request so ai-limits can capture the latest rate_limits payload.
```

After Claude Code updates its settings and creates the marker file, send any normal Claude Code request once. Then run `ai-limits` again.

Expected state after setup but before the first captured limits payload:

```text
~/.config/ai-limits/claude-statusline.json
```

The file can be empty at this point. `ai-limits --claude-statusline` should treat that as "configured, data not captured yet", not as "not configured".

This setup enables Claude Code live limits/reset through `claude_statusline`. It does not confirm coverage for Claude Desktop, Claude web, or browser-extension usage.
