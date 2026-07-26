# Desktop Architecture

## Settings

The desktop application owns user settings. The frontend stores current desktop settings in `localStorage` and passes them to Tauri commands as request parameters.

The CLI is stateless:

- it has no configuration file
- it does not read desktop settings
- built-in defaults apply when no explicit source flags are provided
- command-line arguments affect only the current single query

A shared desktop/CLI settings contract is not part of the architecture.

---

## Desktop

The desktop application uses Tauri as a desktop adapter to the existing Rust core.

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
    commands.rs
    notifications.rs
```

Purpose:

- `main.rs` — Tauri application bootstrap, window setup, plugins, and command registration
- `commands.rs` — desktop commands exposed to the frontend and delegated to the shared core
- `notifications.rs` — desktop notification bridge that delivers core notifications through Tauri

Boundaries:

- `src-tauri/` does not fetch provider data directly
- `src-tauri/` does not decide limit semantics
- `src-tauri/` does not own notification rules
- `src-tauri/` may provide desktop-specific notification transport when needed
- `src-tauri/` may provide desktop-specific window, tray, menu, and permission integration

IPC goals, constraints, and command rules are documented in [inter-process-communication.md](inter-process-communication.md).
