# Local Build

## Local macOS Debug Build

Command:

```text
npm exec tauri -- build --debug --bundles app
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
- local macOS builds do not replace GitHub signing or notarization.

Related documents:

- [Dev run](dev.md)
- [GitHub builds](github-builds.md)
- [macOS signing](macos-signing.md)
