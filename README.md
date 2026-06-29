# ai-usage-mit

A small local tracker for AI tool usage and subscription limits on models.

## How it works

For the user, the app acts as a local assistant: it collects available usage and limit data, normalizes it, and shows a clear summary.

1. **user** → **app**: requests limits
2. **app** → **source**: fetches available data
3. **source** → **app**: returns usage/limits/status
4. **app**: normalizes the result
5. **app** → **user**: shows the summary

## Supported features

- **`ai-usage` command** — queries Codex, Claude, and Cursor in one run and prints a normalized usage/limit summary.
- **Codex** — reads limits via CLI `/status`.
- **Claude** — reads limits via CLI `/usage`.
- **Cursor** — reads usage from `api2.cursor.sh` using a token from `cursor agent login`; if the API is unavailable, falls back to `cursor agent about/status`.

Run from the repository:

```sh
./bin/ai-usage
```

Show CLI help:

```sh
./bin/ai-usage --help
```

Query only selected sources by passing source flags:

```sh
./bin/ai-usage --codex-cli --cursor-api2
```

Supported source flags are:

- `--codex-cli`
- `--claude-cli`
- `--cursor-api2`

`--all` and `-a` force all current sources, even when the config defines a narrower default. When no source is selected, the command uses config defaults or, if no config exists, all three current sources in the fixed output order: Codex, Claude, Cursor.

Optional config path:

```text
~/.config/ai-usage/config.toml
```

Example:

```toml
default_sources = [
  "codex_cli",
  "cursor_api2"
]
```

The command uses the standard `codex`, `claude`, and `cursor` CLIs. Install the provider CLIs for the tools you want to query.
