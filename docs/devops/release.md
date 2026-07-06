# Release

## Unstable GitHub Pre-Releases

Status: active.

Purpose:

- Give collaborators and early users one GitHub place to download the latest
  working desktop build.
- Make the build clearly usable but unstable.
- Avoid asking people to find a specific GitHub Actions run and download CI
  artifacts manually.

Workflow:

```text
.github/workflows/desktop-build.yml
```

Trigger:

```text
workflow_dispatch
```

Current behavior:

- Build macOS, Windows, and Linux files in separate jobs.
- Upload GitHub Actions artifacts with 14-day retention.
- After all platform jobs pass, create an unstable GitHub pre-release.
- Attach separate files for each platform to that pre-release.
- Use an automatic tag based on the workflow run number and attempt:

```text
desktop-unstable-<run-number>-<attempt>
```

Release asset naming:

```text
AI-Limits-desktop-unstable-<run-number>-<attempt>-macos-app.zip
AI-Limits-desktop-unstable-<run-number>-<attempt>-windows-setup.exe
AI-Limits-desktop-unstable-<run-number>-<attempt>-windows.msi
AI-Limits-desktop-unstable-<run-number>-<attempt>-linux.deb
AI-Limits-desktop-unstable-<run-number>-<attempt>-linux.AppImage
```

User-facing meaning:

- These are working desktop builds.
- They are unstable and may contain bugs.
- They are unsigned.
- macOS and Windows may show security warnings.
- They are not stable releases and are not notarized or store-ready.

Download path:

```text
GitHub repository -> Releases -> latest unstable pre-release
```

Do not require users to build the app manually.

## Add Installers and Signed Distribution

Status: future.

Plan:

- Revisit macOS DMG packaging after `.app` CI builds are stable.
- Add Apple signing and notarization as a separate project phase.
- Add Windows signing later if distribution needs it.
- Add Linux packaging refinements after confirming the first Linux artifact.
