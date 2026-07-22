# ai-limits

English | [Русский](README.ru.md)

Check AI subscription limits easily. Codex, Claude, Cursor.

`ai-limits` is a local tool for viewing available AI usage and limit data from supported providers. It can be used as a desktop app or from the terminal.

## Interfaces

### Desktop App

The desktop app is currently in beta.

- macOS works as an app.
- Windows and Linux builds exist and are being tested with real users.
- The interface is functional, but still early.

Download: https://github.com/md2it/ai-limits/releases

Run the desktop app locally in development mode:

```sh
npm run tauri:dev
```

### Terminal UI

The terminal interface remains available.

Run from the repository:

```sh
./bin/ai-limits
```

Show help:

```sh
./bin/ai-limits --help
```

For terminal UI details, see [docs/terminal-ui.md](docs/terminal-ui.md).

## What It Shows

- Current limits for Codex, Claude, and Cursor when available.
- Usage information when a supported source can provide it.
- Results from local files, provider CLIs, and other supported sources.
- Default, structured, and raw output views in the terminal.
- Repeated checks with terminal watch mode.

## Current Limitations

- No macOS DMG installer yet.
- macOS releases are signed and notarized; Windows and Linux builds are unsigned.
- Desktop notifications currently work on macOS only.
- Windows and Linux desktop builds are still being tested.
- Some local Codex and Claude data sources may not work on Windows and Linux yet.
- CLI-backed data sources are expected to be the most portable option across platforms.

## Documentation

- [Terminal UI](docs/terminal-ui.md)
- [Configuration](docs/config.md)
- [Developer documentation](docs/)

## License

This project is licensed under the [MIT License](LICENSE).
