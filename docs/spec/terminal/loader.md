# Terminal Loader

## Loader

The loader shows active work for a source and does not show a progress percentage.

Format:

```text
⠋ waiting codex-rpc
⠙ waiting claude-rpc
```

Unicode spinner frames:

```text
⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏
```

ASCII spinner frames:

```text
- \ | /
```

The ASCII spinner is used when stdout is not a TTY or the environment does not appear to be UTF-8.

The loader starts displaying if a source runs longer than `350ms`.

If a source finishes before the loader is first shown, the loader is not printed.

After a source finishes, the loader is cleared, then the source result block is printed.

If at least one provider block has already been printed and other sources are still waiting, one empty line is printed between the completed provider output and the loader area.

---

## Color

Default output may use color for filled bar characters only.

Frames, headers, loader text, labels, percentages, reset text, and empty bar characters are not colored.

Color is based on remaining limit:

| Remaining limit | Color |
| --- | --- |
| `>= 75%` | green |
| `>= 50%` | yellow |
| `>= 25%` | orange |
| `>= 10%` | red |
| `< 10%` | bright red |

Color is optional. If stdout is not a TTY, the terminal does not support color, or color is disabled by environment settings such as `NO_COLOR`, output must remain readable without ANSI color codes.

---

## Loader Cleanup

In an interactive terminal, the loader is redrawn in place.

On each update:

1. previous loader lines are cleared;
2. current loader lines are printed again;
3. the cursor stays in the loader area.

When a source finishes, the loader is cleared before the result is printed.

When `TerminalUi` shuts down, the loader is cleared via `Drop`.
