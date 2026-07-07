# Tauri Release

Desktop builds should be produced in GitHub Actions, not only on a local machine.

The release pipeline should build each platform in its native environment:

- macOS build on a macOS runner.
- Windows build on a Windows runner.
- Linux build on a Linux runner.

macOS CI builds use Developer ID signing and notarization. See `docs/devops/github-builds.md` for workflow modes and required secrets.
