# Testing

This document is the entry point for testing guidance.

Detailed checks stay in the documents that own the relevant behavior. Keep this page focused on cross-cutting rules, links, and platform-level test setup.

## Quality Gate

Run every automated quality check manually from the project root:

```text
npm run quality
```

The command checks frontend ES modules and Markdown links, then runs Rust formatting, Clippy, and all workspace tests. It runs automatically before each supported local desktop build or dev run and before every GitHub Actions build. A failed check stops the build.

The quality gate does not replace the manual desktop smoke-check. Before a release, start `npm run tauri:dev` and verify that the main window opens, each provider card renders, settings and help open, and external setup links open.

## Test Areas

- Provider source contract tests: [get-limits/providers/contract.md](../get-limits/providers/contract.md#tests).
- Notification trigger and delivery tests: [notifications](../notifications/testing.md#testing).
- Local desktop dev run: [dev run](../setup/dev-run.md).
- Local macOS debug build: [local build](../setup/local-build.md).
- Release artifact verification: `scripts/verify-macos-app.sh`.

The macOS permission reset procedure used before a clean permission check is documented in [macos-permission-reset.md](macos-permission-reset.md).

## Placement Rule

Put testing guidance where the tested behavior is defined when it is specific to one domain, provider, source, UI flow, or release artifact.

Use this document for:

- cross-cutting setup that affects several test areas;
- links to canonical detailed checks;
- shared manual-test conventions;
- OS-level reset or permission preparation.
