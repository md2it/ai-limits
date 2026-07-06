# Tauri Release

Desktop builds should be produced in GitHub Actions, not only on a local machine.

The release pipeline should build each platform in its native environment:

- macOS build on a macOS runner.
- Windows build on a Windows runner.
- Linux build on a Linux runner.

The first release stage should focus on unsigned builds and downloadable artifacts. Signing and notarization should be added later, after the build pipeline is stable.

## Local Development

Local Tauri dev runs must start from fresh generated frontend assets. The npm dev commands run `scripts/tauri-clean-dev-assets.sh` before `tauri dev` to remove stale `target/*/build/*/out/tauri-codegen-assets` directories.

This cleanup is intentionally scoped to generated Tauri codegen assets, not the whole Rust `target` directory. It prevents the desktop WebView from showing stale HTML/CSS during local development while keeping normal incremental Rust builds intact.
