# Desktop Architecture

## Settings

The desktop application owns user settings. The frontend stores current desktop settings in `localStorage` and passes them to Tauri commands as request parameters.

The CLI is stateless:

- it has no configuration file
- it does not read desktop settings
- built-in defaults apply when no explicit source flags are provided
- command-line arguments affect only the current single query

A shared desktop/CLI settings contract is not part of the current architecture.

---

## Desktop

The desktop application uses Tauri as a desktop adapter to the existing Rust core.

The webview uses a restrictive Content Security Policy. It loads scripts, styles, fonts, and images only from the packaged application; the only additional resource source is `data:` for the local CSS icon. Tauri IPC is limited to `ipc:` and `http://ipc.localhost`. New frontend assets or network calls must update the policy deliberately and retain the least-privilege scope.

Rules:

- the shared core must live in `lib.rs` and the `src/` modules
- the CLI must be only one interface to the core
- Tauri is a separate interface to the same core
- `src-tauri/` is a desktop adapter, not a separate business core
- Tauri must use structured data returned by the existing Rust core
- provider logic, limit semantics, and notification rules stay in `src/`
- Tauri commands delegate to core functions instead of duplicating application logic

Structure:

```text
src-tauri/
  src/
    main.rs
    commands/
      mod.rs
      collect.rs
      provider_limits.rs
    notifications.rs
    platform.rs
    platform/
      terminal.rs
```

Purpose:

- `main.rs` — Tauri application bootstrap, window setup, plugins, and command registration
- `commands/` — desktop IPC facade and desktop-only orchestration/projection onto the shared core
  - `mod.rs` — thin `#[tauri::command]` wrappers exposed to the frontend
  - `collect.rs` — provider-limits collection orchestration and notification trigger
  - `provider_limits.rs` — camelCase DTO/projection for the frontend provider-limits contract
- `notifications.rs` — desktop notification delivery through Tauri
- `platform/` — desktop OS adapters (for example terminal launch helpers)

Boundaries:

- `src-tauri/` does not fetch provider data directly
- `src-tauri/` does not decide limit semantics
- `src-tauri/` does not own notification rules
- `src-tauri/` delivers desktop notifications directly through Tauri and does not expose a local notification listener
- `src-tauri/` may provide desktop-specific window, tray, menu, and permission integration

IPC goals, constraints, and command rules are documented in [inter-process-communication.md](inter-process-communication.md).

---

## Frontend modules

`frontend/index.html` holds markup only. Behaviour lives under `frontend/modules/`:

| File | Responsibility |
| --- | --- |
| `main.js` | DOM wiring and app startup |
| `constants.js` | shared ids, storage keys, accents |
| `settings.js` | saved settings and settings UI |
| `theme.js` | light/dark theme |
| `help-chapters.js` | Help section copy |
| `help.js` | Help view behaviour |
| `providers.js` | provider refresh orchestration |
| `provider-section-alignment.js` | equal Limits/Plan section slot heights across provider cards |
| `provider-rendering.js` | provider card markup and data projection into the DOM |
| `provider-refresh-intervals.js` | shared update-frequency setting and refresh timers |
| `provider-formatters.js` | display formatting helpers |
| `showcase.js` | browser screenshot showcase |
| `links.js` | allowlisted external links |

Styles enter through `frontend/styles.css`, which imports area stylesheets under `frontend/styles/` (tokens, base, showcase, toolbar, settings, providers, help) in cascade order.
