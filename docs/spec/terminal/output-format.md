# Terminal Output Format

### General Format

Each `ai-limits` response is printed inside a common frame.

Top frame:

```text
=-=-=-=-=-=-= AI LIMITS =-=-=-=-=-=-=
```

Bottom frame:

```text
=-=-= DONE 2026-07-02 15:04:05 =-=-=
=-=-= PART 2026-07-02 15:04:05 =-=-=
=-=-= FAIL 2026-07-02 15:04:05 =-=-=
```

An empty line is printed before the top frame, after the top frame, before the bottom frame, and after the bottom frame.
The first provider header follows the top-frame gap directly, so there is one empty line between the top frame and the first provider header.
The bottom frame timestamp is the local date and time when the response completed, formatted as `YYYY-MM-DD HH:MM:SS`.

Statuses:

| Status | Meaning |
| --- | --- |
| `DONE` | All requested sources returned a result or a valid unavailable state. |
| `PART` | Some sources returned a result; some ended with an error. |
| `FAIL` | The command did not obtain a usable result. |

---

### Exit Codes

The process exits with code `0` for `DONE` and `PART`, because both statuses contain at least one usable source result. It exits with a non-zero code for `FAIL`, invalid arguments, and command-level errors.

The bottom-frame status and process exit code are part of the stable headless contract.

---

Provider block and usage block formats are documented in [provider-block-format.md](provider-block-format.md) and [usage-block-format.md](usage-block-format.md). User-facing error and recovery output is documented in [problems.md](problems.md).
