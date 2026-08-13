# Terminal Usage Block Format

### Usage Block

`--usage` prints each provider as a separate block using the same provider header format as default limits output.

The usage block contains only available user-facing usage facts. Fields with `null` values are not printed.

Example:

```text
     --------- CODEX ---------
Tokens        input 120,000 | cached 80,000 | output 30,000 | total 230,000
Activity      14 sessions | 128 turns | latest Jul 3, 21:41
Models        top: gpt-5
Money         $12.40 used

Source codex-local: Jul 3, 21:41
```

Supported usage rows:

- `Tokens` — input, cached input, output, reasoning output, cache read/write, and total, when available;
- `Activity` — sessions, turns, files, events, and latest activity, when available;
- `Models` — top model, when available;
- `Money` — used, remaining, total, and currency, when available;
- `Source {source}` — structured `source` and `data_as_of`.

If `data_as_of` is unavailable, print:

```text
Source codex-local: unknown
```

If the source is unavailable, use the same unavailable format as default limits output.
