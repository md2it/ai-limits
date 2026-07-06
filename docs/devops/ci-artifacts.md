# CI Artifacts

## Build Unsigned CI Artifacts

Status: done.

Workflow:

```text
.github/workflows/desktop-build.yml
```

Workflow name:

```text
Desktop unsigned build
```

Trigger:

```text
workflow_dispatch
```

No automatic `push`, `pull_request`, or tag trigger is included.

Verified jobs:

```text
build-macos:   passed, artifact uploaded
build-windows: passed, artifact uploaded
build-linux:   passed, artifact uploaded
```

Common setup:

- checkout repository;
- install Node.js 22;
- install Rust stable through `dtolnay/rust-toolchain@stable`;
- install npm dependencies with `npm ci`;
- run a platform-specific unsigned Tauri production build;
- upload artifacts with `actions/upload-artifact@v4`;
- keep artifact retention at 14 days.

macOS job:

```text
runner: macos-latest
command: npm exec tauri -- build --bundles app
artifact name: ai-limits-macos-app
artifact path: target/release/bundle/macos/AI Limits.app.zip
```

The `.app` bundle is explicitly ad-hoc signed after `tauri build` and before
archive upload:

```text
codesign --force --deep --sign - "target/release/bundle/macos/AI Limits.app"
codesign --verify --deep --strict --verbose=4 "target/release/bundle/macos/AI Limits.app"
```

This does not add Developer ID signing or notarization. It fixes the local
bundle seal so macOS does not reject the unsigned preview app as damaged.

The `.app` bundle is archived with `ditto` after signing to preserve the bundle
structure.

Windows job:

```text
runner: windows-latest
command: npm exec tauri -- build --bundles nsis,msi
artifact name: ai-limits-windows-unsigned
artifact paths:
  target/release/bundle/nsis/*.exe
  target/release/bundle/msi/*.msi
```

Windows signing is not included.

Linux job:

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

Verification result:

- Workflow starts manually from GitHub Actions.
- macOS, Windows, and Linux jobs passed.
- Artifacts were created and uploaded for all three platforms.
- Artifacts were downloaded locally and file paths were confirmed.
- Release publishing was not added.
- Signing, notarization, secrets, and GitHub Releases were not used.

Confirmed artifacts:

```text
macOS:
  artifact name: ai-limits-macos-app
  artifact size: 4,988,236 bytes
  file: AI Limits.app.zip

Windows:
  artifact name: ai-limits-windows-unsigned
  artifact size: 5,365,018 bytes
  files:
    nsis/AI Limits_0.1.0_x64-setup.exe
    msi/AI Limits_0.1.0_x64_en-US.msi

Linux:
  artifact name: ai-limits-linux-unsigned
  artifact size: 80,837,949 bytes
  files:
    deb/AI Limits_0.1.0_amd64.deb
    appimage/AI Limits_0.1.0_amd64.AppImage
```

Local download location used during verification:

```text
/private/tmp/ai-limits-run-28758826398
```

This temporary directory is not a release storage location and may be cleaned by
the operating system.

Implementation guardrails:

- Do not change Rust core code in `src/`.
- Do not change Tauri command behavior in `src-tauri/src/`.
- Do not change frontend UI behavior.
- Do not change provider, limit, config, or notification logic.
- Do not add release publishing yet.
- Do not add signing, notarization, or secret requirements.
