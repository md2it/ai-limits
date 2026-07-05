# Tauri UI States

## No Fresh Data State

If checked sources return no fresh usable limit records, the provider block must show an empty state instead of a technical error like `No usable limit records from this source`.

Short copy:

```text
No fresh limits' data. Try this:
```

Show the existing CLI fallback toggle directly in this state, labeled:

```text
Use CLI too
```

Below the toggle, show a text button:

```text
More details
```

The button opens a modal instead of navigating away.

Modal copy:

```text
CLI data source

CLI fallback is off by default because it can take more time.

If you mostly use Claude or Codex through CLI, this source is usually the most relevant one.

If you do not use CLI but still have trouble getting current limit data, enabling CLI fallback can still help. It is used only as a fallback for providers that do not return data through faster sources.

Provider refreshes run asynchronously, so one slower provider should not block the others.

Setup guides:
Claude guide on GitHub
Codex guide on GitHub
```

The setup links must open externally from the Tauri app:

- Claude guide on GitHub: <https://github.com/md2it/ai-limits/blob/main/docs/setup/claude-cli.md>
- Codex guide on GitHub: <https://github.com/md2it/ai-limits/blob/main/docs/setup/codex-cli.md>

Optional modal helper copy:

```text
You can give this guide to your agent to set it up.
```

Backend state:

- selected by `noFreshData: true`.
- shown when `limits` is empty.
- does not use `errorMessage` text in this state.

Frontend-only state:

- inline `Use CLI too` toggle state comes from `appSettings.useCliFallback`.
- modal open/closed state is frontend state.
