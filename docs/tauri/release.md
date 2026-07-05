# Tauri Release

Desktop builds should be produced in GitHub Actions, not only on a local machine.

The release pipeline should build each platform in its native environment:

- macOS build on a macOS runner.
- Windows build on a Windows runner.
- Linux build on a Linux runner.

The first release stage should focus on unsigned builds and downloadable artifacts. Signing and notarization should be added later, after the build pipeline is stable.
