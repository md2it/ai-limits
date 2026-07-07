# Dev Run

## Local Tauri Dev Run

### Command

```text
npm run tauri:dev
```

Equivalent command:

```text
npm run tauri
```

### Runtime

- [Tauri dev asset cleanup](../../scripts/tauri-clean-dev-assets.sh) removes stale generated Tauri codegen assets from `target/debug/build` and `target/release/build`;
- Tauri starts in development mode;
- [Tauri dev server](../../scripts/tauri-dev-server.sh) serves `frontend/` for the desktop WebView.
- Dev server: http://127.0.0.1:1420
- Config: [Tauri config](../../src-tauri/tauri.conf.json)

Relevant fields:

```text
beforeDevCommand: sh scripts/tauri-dev-server.sh
devUrl: http://127.0.0.1:1420
frontendDist: ../frontend
```

### Output

- Dev mode opens the local Tauri desktop app;
- It does not create release artifacts;
- Local build artifacts are created by `tauri build`, not by the dev run.
- GitHub build artifacts are created by GitHub Actions.

### Related

[Local build](local-build.md)

Use it when a ready local app artifact is needed for testing.
