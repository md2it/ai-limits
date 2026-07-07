# Claude CLI setup for ai-limits

You can give this guide to your agent to set it up.

`ai-limits --claude-cli` needs the Claude Code CLI installed and authorized.

Check installation:

```text
claude --version
```

If the command is missing, install Claude Code:

https://code.claude.com/docs/en/setup

If the command exists but ai-limits says authorization is missing, run:

```text
claude login
```

Then run `ai-limits --claude-cli` again.
