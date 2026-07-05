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

The first release pipeline should prove that every platform can produce a
downloadable artifact in GitHub Actions. Signing, notarization, and polished
installer distribution are future stages.

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

### 2. Design Unsigned CI Builds

Status: ready for implementation.

Plan:

- Create a manual GitHub Actions workflow using `workflow_dispatch`.
- Build each platform on its native runner.
- Use separate jobs first. A matrix can be introduced later after platform
  commands and artifact paths are stable.
- Avoid running this workflow on every push because private repository minutes,
  especially macOS runner minutes, can be expensive.
- Consider adding tag-based runs only after manual CI builds are stable.

Initial platform strategy:

```text
macOS:   macos-latest   -> npm exec tauri -- build --bundles app
Windows: windows-latest -> unsigned Tauri build, exact bundle target to confirm
Linux:   ubuntu-latest  -> unsigned Tauri build, exact bundle target to confirm
```

The Windows and Linux artifact targets should be treated as CI discovery until
their build outputs are confirmed.

First workflow file:

```text
.github/workflows/desktop-build.yml
```

Workflow name:

```text
Desktop unsigned build
```

Required trigger:

```text
workflow_dispatch
```

Do not add these triggers in the first workflow:

- `push`
- `pull_request`
- tag-based release trigger

Recommended jobs:

```text
build-macos
build-windows
build-linux
```

Common setup for every job:

- checkout repository;
- install Node.js;
- install Rust stable;
- install npm dependencies with `npm install` or `npm ci`, depending on whether
  a lockfile is present and valid;
- run the Tauri production build command for the platform;
- upload the resulting artifact.

macOS job:

```text
runner: macos-latest
command: npm exec tauri -- build --bundles app
required artifact: target/release/bundle/macos/AI Limits.app
```

The `.app` bundle should be archived before upload so the directory structure is
preserved. The first implementation may use a zip artifact produced by the
workflow step.

Windows job:

```text
runner: windows-latest
command: npm exec tauri -- build
artifact: discover from actual CI output
```

The first Windows run should report which bundle files were produced under
`target/release/bundle/`. Do not add Windows signing.

Linux job:

```text
runner: ubuntu-latest
command: npm exec tauri -- build
artifact: discover from actual CI output
```

Linux may need system packages for WebKitGTK and related Tauri dependencies.
The first implementation should use Tauri 2 Linux CI requirements from the
current official documentation or established Tauri action examples. Any added
Linux packages must be documented in the workflow implementation report.

First artifact naming:

```text
ai-limits-macos-app
ai-limits-windows-unsigned
ai-limits-linux-unsigned
```

First workflow success criteria:

- the workflow can be started manually;
- macOS produces and uploads an unsigned `.app` archive;
- Windows either uploads an unsigned artifact or reports the missing path clearly;
- Linux either uploads an unsigned artifact or reports the missing dependency or
  path clearly;
- no signing or notarization settings are introduced;
- no GitHub Release is created.

Implementation guardrails:

- Do not change Rust core code in `src/`.
- Do not change Tauri command behavior in `src-tauri/src/`.
- Do not change frontend UI behavior.
- Do not change provider, limit, config, or notification logic.
- Do not add release publishing yet.
- Do not add signing, notarization, or secret requirements.

### 3. Upload CI Artifacts

Status: planned.

Plan:

- Upload artifacts from every successful platform build.
- Use clear artifact names that include app name, platform, and architecture when
  known.
- Keep artifact retention modest during discovery.
- Do not attach artifacts to GitHub Releases until the CI artifact stage is
  stable.

Expected first macOS artifact:

```text
target/release/bundle/macos/AI Limits.app
```

Windows and Linux artifact paths must be confirmed by the first CI runs.

### 4. Stabilize Platform Builds

Status: planned.

Plan:

- Run manual workflow.
- Inspect each platform's produced bundle paths.
- Document required Linux system dependencies if the Linux runner needs them.
- Document Windows artifact type and path.
- Adjust workflow only for build and artifact correctness.

Success criteria:

- macOS artifact uploads successfully.
- Windows artifact uploads successfully.
- Linux artifact uploads successfully.
- Workflow failures are actionable and documented.

### 5. Add GitHub Releases

Status: future.

Plan:

- Add a release workflow only after unsigned CI artifacts are stable.
- Prefer tag-based release creation.
- Attach confirmed artifacts from all platforms.
- Keep release notes simple and factual.
- Do not introduce signing or notarization as part of the first release workflow.

### 6. Add Installers and Signed Distribution

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
- Keep the first workflow focused on build artifacts, not release publishing.
- Do not require DMG for the first macOS CI success.
- Do not add signing or notarization secrets yet.
- Keep platform-specific commands explicit if a matrix makes the workflow hard to
  read.
- Cache dependencies only if it does not obscure the first working workflow.

---

## Artifact Principles

- First priority: prove that CI can produce downloadable unsigned artifacts.
- Artifact names should be stable and human-readable.
- Artifact paths must be based on actual CI output, not assumptions.
- macOS `.app` may need to be archived before upload so the bundle structure is
  preserved.
- DMG, MSI, NSIS, AppImage, deb, and rpm should not be assumed until confirmed.

---

## Open Questions

- Which Windows bundle target should be the first required artifact?
- Which Linux bundle target should be the first required artifact?
- Which Linux system dependencies are required on the GitHub runner?
- Should the first workflow use a matrix or separate jobs for clarity?
- Should tag-based runs be added immediately or only after manual builds pass?

---

## Recommended Next Task

Design the first unsigned GitHub Actions workflow in this document before
implementation.

The design should define:

- workflow triggers;
- runner list;
- build commands;
- artifact upload paths;
- artifact names;
- expected known risks;
- checks for the implementing agent.

After the design is approved, a separate implementation task can create the
workflow file under `.github/workflows/`.
