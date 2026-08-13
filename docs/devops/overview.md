# Desktop DevOps

This directory defines the goals, constraints, and release policy for desktop distribution.

Detailed Tauri integration rules remain in [architecture](../spec/architecture.md) and [desktop docs](../spec/desktop/).

## Goal

Build and publish trustworthy desktop artifacts for supported platforms:

- macOS — signed, notarized, and stapled for a release-ready distribution, and updated in place after installation;
- Windows and Linux — distributed without signing until signing is introduced;
- every platform — receives its own downloadable artifact.

The release process must make the artifact's platform and security state clear to users. Windows signing and store distribution are future work.

## Current Implementation

The current implementation uses [GitHub Actions](../../.github/workflows/desktop-build.yml) to build and publish an unstable pre-release. The [Tauri configuration](../../src-tauri/tauri.conf.json) remains the source of truth for application packaging.

Installed macOS copies update themselves from that pre-release; see [auto-update](auto-update.md).

## Scope

The desktop build/release work covers artifact creation, validation, signing policy, versioning, and distribution.

Out of scope for now:

- Windows code signing.
- Store distribution.
- Reworking Rust core logic.
- Reworking Tauri commands.
- Reworking frontend UI.
- Duplicating provider, config, limit, or notification logic in `src-tauri/`.
