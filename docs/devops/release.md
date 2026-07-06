# Release

## Add GitHub Releases

Status: future.

Plan:

- Add a release workflow only after unsigned artifacts pass smoke verification,
  or after an explicit decision to publish untested unsigned artifacts.
- Prefer tag-based release creation.
- Attach confirmed artifacts from all platforms.
- Keep release notes simple and factual.
- Do not introduce signing or notarization as part of the first release workflow.

## Add Installers and Signed Distribution

Status: future.

Plan:

- Revisit macOS DMG packaging after `.app` CI builds are stable.
- Add Apple signing and notarization as a separate project phase.
- Add Windows signing later if distribution needs it.
- Add Linux packaging refinements after confirming the first Linux artifact.
