# Local Build

## Local macOS Debug Build

Command:

```text
npm run tauri:build:debug
```

Purpose:

- create a local `.app` artifact on a developer machine;
- check the built app outside `tauri dev`;
- support local testing before or outside a GitHub Actions run.

Output:

```text
target/debug/ai-limits-desktop
target/debug/bundle/macos/AI Limits.app
```

Distribution meaning:

- this is a local build, not a GitHub build;
- how it is used depends on the task, but it is not the primary distribution channel;
- for prod, pre-prod, and shared test builds, use GitHub Actions;
- the published macOS distributable is the signed and notarized disk image built by GitHub Actions, not this bundle;
- local macOS builds do not replace GitHub signing or notarization.

## Local Disk Image Preview

Command:

```text
scripts/build-macos-dmg.sh "target/debug/bundle/macos/AI Limits.app" /tmp/preview.dmg
```

Purpose:

- preview the disk image window (background, icon layout) on a developer machine, unsigned;
- this is the same script GitHub Actions runs, so a local preview matches what a release will look like;
- avoids Tauri's own dmg bundler, whose Finder-styling step needs the Automation permission and is unreliable outside a fresh grant — see [macOS signing](../../devops/macos-signing.md).

Related documents:

- [Dev run](dev-run.md)
- [Desktop builds](../../devops/builds.md)
- [macOS signing](../../devops/macos-signing.md)
