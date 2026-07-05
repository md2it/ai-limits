# Tauri

## Choice

Tauri is the planned desktop framework for this project.

The choice was made because Tauri produces lightweight desktop applications. The final app should be small, fast, and available for the three main desktop operating systems: Linux, macOS, and Windows.

Rust also fits this direction well, because Tauri uses Rust for the native application layer.

## Provider Data Contract

The desktop UI should not depend on one combined provider response for normal refresh behavior.

Tauri commands should support fetching one provider at a time, for example Codex, Claude, or Cursor. The frontend can then start provider refreshes in parallel and update each provider block as soon as that provider result is available.

This keeps slow sources from blocking unrelated provider blocks and matches the planned per-provider update frequency controls.

## Release Scheme

Desktop builds should be produced in GitHub Actions, not only on a local machine.

The release pipeline should build each platform in its native environment:

- macOS build on a macOS runner.
- Windows build on a Windows runner.
- Linux build on a Linux runner.

The first release stage should focus on unsigned builds and downloadable artifacts. Signing and notarization should be added later, after the build pipeline is stable.

## Initial Milestones

1. Add Tauri project structure.
2. Verify local development on macOS.
3. Add GitHub Actions for Linux, macOS, and Windows builds.
4. Publish build artifacts from GitHub Actions.
5. Add GitHub Releases.
6. Add macOS signing and notarization.
7. Add Windows signing when needed.
