# Desktop DevOps

This directory is the source of truth for desktop build, artifacts, and release
work until the desktop release pipeline is stable.

Detailed Tauri integration rules remain in `../architecture.md` and
`../tauri/`.

## Documents

- [CI artifacts](ci-artifacts.md)
- [Icons](icons.md)
- [Temporary artifact verification](artifact-verification-temp.md)
- [Release](release.md)
- [Principles and risks](principles-and-risks.md)

## Goal

Build and publish unsigned desktop artifacts for the Tauri app across supported
platforms:

- macOS
- Windows
- Linux

The current unsigned CI workflow proves that every platform can produce a
downloadable artifact in GitHub Actions. The next release-readiness step is
manual verification of the downloaded artifacts. Signing, notarization, and
polished installer distribution are future stages.

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

## Global Plan

### 1. Confirm Local macOS Build

Status: done.

Outcome:

- Local unsigned macOS `.app` build is confirmed.
- Default DMG packaging is not confirmed.
- First CI stage should use `.app`, not DMG, as the required macOS artifact.

### 2. Build Unsigned CI Artifacts

Status: done.

See [CI artifacts](ci-artifacts.md).

### 3. Generate Desktop Icons

Status: active.

See [Icons](icons.md).

### 4. Verify Downloaded Artifacts

Status: in progress.

See [Temporary artifact verification](artifact-verification-temp.md).

### 5. Add GitHub Releases

Status: future.

See [Release](release.md).

### 6. Add Installers and Signed Distribution

Status: future.

See [Release](release.md).

## Recommended Next Task

Verify downloaded artifacts on target platforms before planning GitHub
Releases.

GitHub Releases should be planned only after artifact verification results are documented,
unless the project explicitly chooses to publish unsigned artifacts with known
untested runtime risk.
