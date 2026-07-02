# Codex CLI setup for ai-limits

`ai-limits --codex-cli` needs the Codex CLI installed and authorized.

Check installation:

```text
codex --version
```

If the command is missing, install Codex CLI:

https://developers.openai.com/codex/cli

If the command exists but ai-limits says authorization is missing, run:

```text
codex login
```

Then run `ai-limits --codex-cli` again.
