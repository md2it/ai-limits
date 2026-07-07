# Temporary Artifact Verification

## Verify Downloaded Artifacts

Status: in progress.

Plan:

- Verify that downloaded artifacts can be opened or installed on target platforms.
- Keep this as a manual verification step before GitHub Releases.
- Do not change signing or notarization during artifact verification.
- Do not create GitHub Releases during artifact verification.

## macOS release artifact checks

Use `ditto` to extract the zip. Do not use `unzip`; it does not preserve macOS extended attributes and can break signed/stapled `.app` bundles.

```sh
mkdir -p /tmp/ai-limits-verify
ditto -x -k "AI Limits.app.zip" /tmp/ai-limits-verify
```

Run the shared verification script on the downloaded zip or extracted `.app`:

```sh
scripts/verify-macos-app.sh --notarization full "AI Limits.app.zip"
```

Or on the extracted bundle:

```sh
scripts/verify-macos-app.sh --notarization full "/tmp/ai-limits-verify/AI Limits.app"
```

The script runs:

```text
codesign -dv
codesign -d --entitlements -
codesign --verify --deep --strict
spctl --assess
xcrun stapler validate   # only in full mode
```

Minimum manual checks:

```text
macOS:
  extract with ditto -x -k
  run scripts/verify-macos-app.sh
  launch the .app
  record signing and notarization mode / Gatekeeper UX

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
- whether the artifact is acceptable for an unstable preview release.

Current verification results:

```text
macOS:
  artifact file: AI Limits.app.zip
  extract command: ditto -x -k
  verification script: scripts/verify-macos-app.sh
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
    result was observed on the development device, so external macOS behavior may differ

Windows:
  status: pending external tester

Linux:
  status: pending external tester
```
