# Desktop Builds

## Process

- A release build is started deliberately, not for every source-code change.
- Validate the intended version and release notes before allocating platform build resources.
- Build each supported platform in an appropriate native environment.
- Verify the distributable artifact, not only the build output.
- Publish only when every required platform artifact is available and valid.

The current implementation is [GitHub Actions](../../.github/workflows/desktop-build.yml). It is an implementation detail and may be replaced without changing this process.

## Current Security Policy

- macOS release artifacts must be signed, notarized, and stapled.
- Windows and Linux artifacts are currently unsigned; their status must be visible to users.
- Signing credentials must be held only by the protected build environment and never committed to the repository.

See [macOS signing](macos-signing.md) and [versioning](versioning.md).

## macOS Widget Validation

Run `scripts/build-macos-app-with-widgets.sh` on macOS to build the universal Tauri app, build the Widget Extension with code signing disabled, and embed it in the app bundle. This path requires Xcode but no certificate or provisioning profile.

The existing `Desktop build` workflow has a `macos_widget_validation` mode for a current branch. This mode does not publish a release or change the changelog. It builds the host app and Widget Extension with Developer ID signing, verifies the nested signatures and App Group entitlements before and after zip packaging, launches the host app, and checks Widget Extension registration through `pluginkit`.

Start the workflow with `macos_widget_validation=true`; the required release `version` input is ignored in this mode.
