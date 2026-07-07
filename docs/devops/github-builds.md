# GitHub Builds

## Desktop build workflow

Status: active.

GitHub Actions workflow:

[Desktop build workflow](../../.github/workflows/desktop-build.yml)

Workflow name:

```text
Desktop build
```

Trigger:

```text
workflow_dispatch
```

No automatic `push`, `pull_request`, or tag trigger is included.

When starting the workflow, choose **macOS notarization mode**:

| Mode | What happens | CI time | Release-ready macOS |
|------|----------------|---------|---------------------|
| `full` (default) | Sign, notarize, wait for Apple, staple | minutes to hours | yes |
| `submit-only` | Sign, submit notarization, do not wait | ~5 min | no (check Apple later) |
| `sign-only` | Developer ID sign only | ~3 min | no |

Use `full` for pre-releases that users download from GitHub. Use `submit-only` or `sign-only` only when iterating on CI and you do not need a finished macOS artifact yet.

### macOS notarization notes

- First notarization for a new Apple Developer team can stay `In Progress` at Apple for hours or longer while the app is held for in-depth analysis.
- After the first `Accepted` result, later `full` runs are usually much faster.
- Check submission status locally:

```sh
xcrun notarytool history --key ... --key-id ... --issuer ...
```

- If you used `submit-only` and Apple later reports `Accepted`, either rerun the workflow with `full` or staple the same `.app` locally with `xcrun stapler staple`.

See [macOS signing](macos-signing.md) for secrets and signing details.

## Jobs

Verified jobs:

```text
build-macos:   signed macOS app, notarization verified when full, artifact uploaded
build-windows: passed, artifact uploaded
build-linux:   passed, artifact uploaded
```

Common GitHub job setup:

- checkout repository;
- install Node.js 22;
- install Rust stable through `dtolnay/rust-toolchain@stable`;
- install npm dependencies with `npm ci`;
- upload artifacts with `actions/upload-artifact@v4`;
- keep artifact retention at 14 days.

### macOS job

```text
runner: macos-latest
command: npm exec tauri -- build --bundles app --target universal-apple-darwin
artifact name: ai-limits-macos-app
artifact path: target/release/bundle/macos/AI Limits.app.zip
```

The workflow imports a Developer ID Application `.p12`, writes the App Store Connect API key, and lets Tauri sign the universal `.app` bundle. In `full` mode, Tauri also notarizes and staples before the zip is uploaded.

Entitlements: `src-tauri/Entitlements.plist` with hardened runtime enabled in `tauri.conf.json`.

The workflow verifies the final `.app` before archive upload:

```text
codesign --verify --deep --strict --verbose=4 "target/universal-apple-darwin/release/bundle/macos/AI Limits.app"
```

In `full` mode, the workflow also verifies notarization and stapling in a separate macOS job step:

```text
stapler validate "target/universal-apple-darwin/release/bundle/macos/AI Limits.app"
```

The `.app` bundle is archived with `ditto` after signing to preserve the bundle structure.

### Windows job

```text
runner: windows-latest
command: npm exec tauri -- build --bundles nsis,msi
artifact name: ai-limits-windows-unsigned
artifact paths:
  target/release/bundle/nsis/*.exe
  target/release/bundle/msi/*.msi
```

Windows signing is not included.

### Linux job

```text
runner: ubuntu-latest
command: npm exec tauri -- build --bundles deb,appimage
artifact name: ai-limits-linux-unsigned
artifact paths:
  target/release/bundle/deb/*.deb
  target/release/bundle/appimage/*.AppImage
```

Linux system dependencies added to the workflow:

```text
libwebkit2gtk-4.1-dev
libgtk-3-dev
libayatana-appindicator3-dev
librsvg2-dev
patchelf
```

## Verification result

- Workflow starts manually from GitHub Actions.
- macOS, Windows, and Linux jobs pass when secrets and runners are available.
- Artifacts are created and uploaded for all three platforms.
- Release publishing creates an unstable GitHub pre-release after all platform jobs pass.
- macOS signing is used.
- macOS notarization is controlled by workflow input.
- Windows and Linux signing are not used.

## Implementation guardrails

- Do not change Rust core code in `src/`.
- Do not change Tauri command behavior in `src-tauri/src/`.
- Do not change frontend UI behavior.
- Do not change provider, limit, config, or notification logic.
- Keep Windows and Linux signing out of the current workflow unless explicitly requested.
- Keep macOS signing secrets in GitHub Actions only; do not commit them.
- Keep unstable release publishing clear about the selected macOS notarization mode.
