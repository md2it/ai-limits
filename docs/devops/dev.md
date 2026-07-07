# Dev Run

## Local Tauri Dev Run

Command:

```text
npm run tauri:dev
```

Equivalent command:

```text
npm run tauri
```

What happens:

- `scripts/tauri-clean-dev-assets.sh` removes stale generated Tauri codegen
  assets from `target/debug/build` and `target/release/build`;
- Tauri starts in development mode;
- `scripts/tauri-dev-server.sh` serves `frontend/` for the desktop WebView.

Dev server:

```text
http://127.0.0.1:1420
```

Config:

```text
src-tauri/tauri.conf.json
```

Relevant fields:

```text
beforeDevCommand: sh scripts/tauri-dev-server.sh
devUrl: http://127.0.0.1:1420
frontendDist: ../frontend
```

Output:

- dev mode opens the local Tauri desktop app;
- it does not create release artifacts;
- local build artifacts are created by `tauri build`, not by the dev run.
- GitHub build artifacts are created by GitHub Actions.

Related document:

```text
local-build.md
```
