# Testing

This document is the entry point for testing guidance.

Detailed checks stay in the documents that own the relevant behavior. Keep this page focused on cross-cutting rules, links, and platform-level test setup.

## Test Areas

- Provider source contract tests: [get-info providers](get-info/providers/README.md#tests).
- Notification trigger and delivery tests: [notifications](notifications.md#testing).
- Local desktop dev run: [devops dev run](devops/dev.md).
- Local macOS debug build: [devops local build](devops/local-build.md).
- Release artifact verification: [temporary artifact verification](devops/artifact-verification-temp.md) and `scripts/verify-macos-app.sh`.
- Analog research hands-on checks: [references flow](references/_flow.md).

## Placement Rule

Put testing guidance where the tested behavior is defined when it is specific to one domain, provider, source, UI flow, or release artifact.

Use this document for:

- cross-cutting setup that affects several test areas;
- links to canonical detailed checks;
- shared manual-test conventions;
- OS-level reset or permission preparation.

## macOS Permission Reset

Use this procedure before a clean macOS permission check.

1. Install a signed and notarized GitHub build in a stable location, preferably:

```text
/Applications/AI Limits.app
```

2. Remove older app copies if they can affect which app macOS opens.

3. Reset TCC permissions for the desktop bundle:

```text
tccutil reset All com.ai-limits.desktop
```

4. Launch the app and run the permission scenario being checked.

5. For notifications, also check the system notification permission manually:

```text
System Settings -> Notifications -> AI Limits
```

6. For provider permission diagnostics, test one provider at a time and record which action triggers each macOS prompt.

7. If permission behavior remains inconsistent, log out and back in, or reboot macOS.
