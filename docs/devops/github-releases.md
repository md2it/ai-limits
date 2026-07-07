# GitHub Releases

## Unstable GitHub Pre-Releases

Status: active.

Purpose:

- Give collaborators and early users one GitHub place to download the latest working desktop build.
- Make the build clearly usable but unstable.
- Avoid asking people to find a specific GitHub Actions run and download workflow artifacts manually.

GitHub Actions workflow:

[Desktop build workflow](../../.github/workflows/desktop-build.yml)

Trigger:

```text
workflow_dispatch
```

Current GitHub behavior:

- Build macOS, Windows, and Linux files in separate jobs.
- Upload GitHub Actions artifacts with 14-day retention.
- After all platform build jobs pass, create an unstable GitHub pre-release.
- Attach separate files for each platform to that pre-release.
- Use the version provided when the workflow starts.

See [Versioning](versioning.md).

### macOS notarization mode

The workflow input `macos_notarization` controls the macOS job:

| Mode | Default | Use when |
|------|---------|----------|
| `full` | yes | Publishing a pre-release users should install |
| `submit-only` | no | You want CI to finish quickly and will wait for Apple separately |
| `sign-only` | no | Testing signing only; Gatekeeper may still warn |

See [GitHub builds](github-builds.md) for secrets and timing notes.

Release description:

- Should be generated automatically from commit messages between the previous release and the current release.
- Should include only user-relevant commit types: `feat:` and `fix:`.
- Should exclude `docs:` and `chore:` by default.
- Can be edited manually after release creation to remove noise or clarify wording.
- Depends on the commit message rules in [Contributing](../../CONTRIBUTING.md).
- Commit message checks are advisory and should warn without blocking commits or builds.

User-facing meaning:

- These are working desktop builds.
- They are unstable and may contain bugs.
- macOS builds from `full` runs are signed, notarized, and stapled.
- macOS notarization depends on the selected workflow input: `sign-only`, `submit-only`, or `full` (default).
- Windows and Linux are unsigned.
- Windows may show security warnings.
- They are not stable releases or store-ready.

Download path:

```text
GitHub repository -> Releases -> latest unstable pre-release
```

Do not require users to build the app manually.

## Add Installers and Signed Distribution

Status: active for macOS, future for Windows and Linux.

Plan:

- Revisit macOS DMG packaging after `.app` GitHub builds are stable.
- Keep Apple signing and notarization documented as part of the macOS GitHub Actions path.
- Add Windows signing later if distribution needs it.
- Add Linux packaging refinements after confirming the first Linux artifact.
