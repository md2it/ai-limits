# Desktop Build and Release Plan

This document is the top-level plan for desktop build, artifacts, and release work.
It is the source of truth for planning GitHub Actions and GitHub Releases until the
desktop release pipeline is stable.

Detailed Tauri integration rules remain in `docs/architecture.md` and `docs/tauri/`.

---

## Goal

Build and publish unsigned desktop artifacts for the Tauri app across supported
platforms:

- macOS
- Windows
- Linux

The current unsigned CI workflow proves that every platform can produce a
downloadable artifact in GitHub Actions. The next release-readiness step is
manual smoke testing of the downloaded artifacts. Signing, notarization, and
polished installer distribution are future stages.

---

## Current Status

Confirmed:

- The app is a Rust + Tauri desktop application.
- `src/` is the shared Rust core and CLI.
- `src-tauri/` is the Tauri desktop adapter.
- Tauri uses structured data from the Rust core through commands.
- Tauri must not duplicate provider logic, limit semantics, configuration, or
  notification rules.
- Local macOS production `.app` build is confirmed.
- The produced macOS `.app` was manually launched and checked by the user.
- GitHub Actions unsigned desktop build workflow is implemented and verified.
- CI produces unsigned artifacts for macOS, Windows, and Linux.
- CI artifacts were downloaded and inspected by file name/path.

Known local macOS build result:

```text
npm exec tauri -- build --bundles app
```

This produces:

```text
target/release/ai-limits-desktop
target/release/bundle/macos/AI Limits.app
```

Known issue:

- Default `npm exec tauri -- build` currently reaches DMG packaging and fails on
  the DMG bundling step.
- DMG is not a blocker for the first unsigned `.app` artifact stage.
- DMG packaging should be handled as a later, separate task.

Verified CI run:

```text
URL: https://github.com/md2it/ai-limits/actions/runs/28758826398
Run ID: 28758826398
Trigger: workflow_dispatch
Status: success
```

---

## Scope

The desktop build/release work covers:

- GitHub Actions workflow design.
- Native OS builds on GitHub-hosted runners.
- Uploading build artifacts from CI.
- A later GitHub Releases workflow after CI artifacts are stable.
- Documentation of build commands, produced paths, and known platform-specific
  issues.

Out of scope for the first stages:

- Signing.
- Apple notarization.
- macOS DMG as a required artifact.
- Store distribution.
- Reworking Rust core logic.
- Reworking Tauri commands.
- Reworking frontend UI.
- Duplicating provider, config, limit, or notification logic in `src-tauri/`.

---

## Global Plan

### 1. Confirm Local macOS Build

Status: done.

Outcome:

- Local unsigned macOS `.app` build is confirmed.
- Default DMG packaging is not confirmed.
- First CI stage should use `.app`, not DMG, as the required macOS artifact.

### 2. Build Unsigned CI Artifacts

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

The `.app` bundle is archived with `ditto` before upload to preserve the bundle
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

### Desktop Icon Generation

Status: active.

Icon source files:

```text
src-tauri/icons/icon-master.svg
src-tauri/icons/icon-desktop.svg
```

Rules:

- `icon-master.svg` is the master artwork. It intentionally has no internal
  padding.
- `icon-desktop.svg` is the desktop icon source derived from the master artwork.
  It keeps the black background and adds desktop-safe internal padding.
- Desktop PNG, `.ico`, and `.icns` files must be generated from
  `icon-desktop.svg`, not directly from `icon-master.svg`.
- Android and iOS icon folders are not part of this desktop application.

Required local tools:

```text
qlmanage
magick
iconutil
sips
```

Tool roles:

- Use macOS QuickLook through `qlmanage` to render SVG into PNG. This is the
  confirmed renderer for the current SVG artwork.
- Do not use ImageMagick as the SVG renderer for this icon. During verification,
  ImageMagick rendered the small sparkles but dropped the main logo shape.
- Use ImageMagick only to assemble `icon.ico` from already rendered PNG files.
- Use a direct ICNS container build from already rendered PNG files, then verify
  the result with `iconutil`.
- Use `sips` for dimension checks.

Expected desktop icon files:

```text
src-tauri/icons/32x32.png
src-tauri/icons/64x64.png
src-tauri/icons/128x128.png
src-tauri/icons/128x128@2x.png
src-tauri/icons/icon.png
src-tauri/icons/icon-1024.png
src-tauri/icons/icon.ico
src-tauri/icons/icon.icns
```

Windows logo PNG files in `src-tauri/icons/Square*Logo.png` and
`src-tauri/icons/StoreLogo.png` are also generated from `icon-desktop.svg`.

Current desktop icon padding:

```text
source: src-tauri/icons/icon-desktop.svg
canvas: 1024x1024
background: black
artwork scale: 80%
internal padding: about 10% per side
```

Verified behavior:

- Local Tauri macOS build copies `src-tauri/icons/icon.icns` into the `.app`
  bundle without changing it.
- GitHub Actions macOS build also copied `icon.icns` without changing it.
- GitHub Actions Linux `.deb` package copied the checked PNG files without
  changing them.
- Therefore desktop icon padding is controlled by the source icon files, not by
  GitHub Actions or Tauri at build time.

### 3. Smoke-Test Downloaded Artifacts

Status: in progress.

Plan:

- Verify that downloaded artifacts can be opened or installed on target
  platforms.
- Keep this as a manual verification step before GitHub Releases.
- Do not add signing or notarization during smoke testing.
- Do not create GitHub Releases during smoke testing.

Minimum checks:

```text
macOS:
  unzip AI Limits.app.zip
  launch the .app
  record unsigned app / Gatekeeper UX

Windows:
  install or run NSIS setup
  optionally install MSI
  launch app after installation

Linux:
  run AppImage
  optionally install DEB
  launch app after installation
```

Smoke-test result should document:

- platform;
- artifact file used;
- installation/opening result;
- launch result;
- blocking UX or security warning;
- whether the artifact is acceptable for an unsigned preview release.

Current smoke-test results:

```text
macOS:
  artifact file: AI Limits.app.zip
  unzip result: passed
  launch result: passed
  installer flow: not applicable; artifact is a zipped .app, not a DMG
  Gatekeeper UX:
    app opened immediately
    no "Apple cannot check it for malicious software" warning reported
    no "unidentified developer" warning reported
    right click -> Open was not required
    System Settings -> Privacy & Security -> Open Anyway was not required
    launch was not blocked
  note:
    result was observed on the development device, so external macOS behavior
    may differ

Windows:
  status: pending external tester

Linux:
  status: pending external tester
```

### 4. Add GitHub Releases

Status: future.

Plan:

- Add a release workflow only after unsigned artifacts pass smoke verification,
  or after an explicit decision to publish untested unsigned artifacts.
- Prefer tag-based release creation.
- Attach confirmed artifacts from all platforms.
- Keep release notes simple and factual.
- Do not introduce signing or notarization as part of the first release workflow.

### 5. Add Installers and Signed Distribution

Status: future.

Plan:

- Revisit macOS DMG packaging after `.app` CI builds are stable.
- Add Apple signing and notarization as a separate project phase.
- Add Windows signing later if distribution needs it.
- Add Linux packaging refinements after confirming the first Linux artifact.

---

## GitHub Actions Principles

- Start with manual workflows.
- Use native runners per platform.
- Keep the current workflow focused on build artifacts, not release publishing.
- Do not require DMG for the first macOS CI success.
- Do not add signing or notarization secrets yet.
- Keep platform-specific commands explicit if a matrix makes the workflow hard to
  read.

---

## Artifact Principles

- CI already proves that downloadable unsigned artifacts can be produced.
- Artifact names should be stable and human-readable.
- Artifact paths must remain based on actual CI output, not assumptions.
- macOS `.app` is archived before upload so the bundle structure is preserved.
- GitHub Actions artifact retention is currently 14 days.
- Long-term release artifacts should be handled through GitHub Releases or a
  documented release staging directory, not `/private/tmp`.

---

## Known Warnings and Risks

- GitHub Actions reported that `actions/checkout@v4`, `actions/setup-node@v4`,
  and `actions/upload-artifact@v4` target a Node.js 20 runtime that is deprecated
  and currently forced to run on Node.js 24. This is not blocking now, but action
  versions should be revisited when newer versions are available.
- GitHub Actions reported that `macos-latest` will migrate to macOS 26. If
  release stability becomes sensitive to macOS runner changes, pin the runner to
  a specific macOS version.
- Linux artifact size is materially larger than macOS and Windows artifacts:
  `ai-limits-linux-unsigned` was 80,837,949 bytes in the verified run.
- Downloaded artifacts used for verification were stored in `/private/tmp`, which
  is not durable storage.
- Artifact install/open smoke testing has not been completed yet.

---

## Recommended Next Task

Smoke-test downloaded artifacts on target platforms before planning GitHub
Releases.

GitHub Releases should be planned only after smoke-test results are documented,
unless the project explicitly chooses to publish unsigned artifacts with known
untested runtime risk.
