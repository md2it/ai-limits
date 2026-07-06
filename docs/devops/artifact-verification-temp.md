# Temporary Artifact Verification

## Verify Downloaded Artifacts

Status: in progress.

Plan:

- Verify that downloaded artifacts can be opened or installed on target
  platforms.
- Keep this as a manual verification step before GitHub Releases.
- Do not add signing or notarization during artifact verification.
- Do not create GitHub Releases during artifact verification.

Minimum checks:

```text
macOS:
  unzip AI Limits.app.zip
  launch the .app
  record unsigned app / Gatekeeper UX

Windows:
  install or run NSIS setup
  optionally install MSI
  launch app after installation

Linux:
  run AppImage
  optionally install DEB
  launch app after installation
```

Artifact verification result should document:

- platform;
- artifact file used;
- installation/opening result;
- launch result;
- blocking UX or security warning;
- whether the artifact is acceptable for an unsigned preview release.

Current verification results:

```text
macOS:
  artifact file: AI Limits.app.zip
  unzip result: passed
  launch result: passed
  installer flow: not applicable; artifact is a zipped .app, not a DMG
  Gatekeeper UX:
    app opened immediately
    no "Apple cannot check it for malicious software" warning reported
    no "unidentified developer" warning reported
    right click -> Open was not required
    System Settings -> Privacy & Security -> Open Anyway was not required
    launch was not blocked
  note:
    result was observed on the development device, so external macOS behavior
    may differ

Windows:
  status: pending external tester

Linux:
  status: pending external tester
```
