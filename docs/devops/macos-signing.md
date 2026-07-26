# macOS Signing

## Goal

A macOS artifact intended for users must be trusted by macOS: Developer ID signed, Apple-notarized, and stapled. The final archived artifact must be verified after packaging, because packaging can invalidate the properties being protected.

## Modes

- `full` is the release-ready mode: signing, notarization, and stapling are complete.
- `submit-only` is for an early notarization submission; it is not a user-ready artifact.
- `sign-only` is for build or signing diagnostics; macOS may warn users about the artifact.

Notarization time is controlled by Apple and can be materially longer for a new team. A release must not be represented as notarized until Apple accepts it.

## Credential Policy

The protected build environment receives the signing certificate and Apple notarization credentials. The repository contains neither credentials nor their values. The platform implementation must derive the signing identity from the supplied certificate rather than from a hard-coded identity.

The current implementation is the [desktop workflow](../../.github/workflows/desktop-build.yml), [Tauri packaging configuration](../../src-tauri/tauri.conf.json), and [macOS verification script](../../scripts/verify-macos-app.sh). The [secrets example](../../scripts/macos-signing-secrets.example) lists the current integration variables.

The host app and Widget Extension both use the App Group `group.md2it.ai-limits.shared`. Developer ID signing for this restricted entitlement requires separate Developer ID provisioning profiles for bundle identifiers `com.ai-limits.desktop` and `com.ai-limits.desktop.widgets`. CI embeds the host profile, lets Xcode embed the extension profile, signs the extension before the containing app, and verifies both effective entitlement sets.
