# Principles and Risks

## GitHub Actions Principles

- Start with manual workflows.
- Use native runners per platform.
- Keep the current workflow focused on build artifacts, not release publishing.
- Do not require DMG for the first macOS CI success.
- Do not add signing or notarization secrets yet.
- Keep platform-specific commands explicit if a matrix makes the workflow hard to
  read.

## Artifact Principles

- CI already proves that downloadable unsigned artifacts can be produced.
- Artifact names should be stable and human-readable.
- Artifact paths must remain based on actual CI output, not assumptions.
- macOS `.app` is archived before upload so the bundle structure is preserved.
- GitHub Actions artifact retention is currently 14 days.
- Long-term release artifacts should be handled through GitHub Releases or a
  documented release staging directory, not `/private/tmp`.

## Known Warnings and Risks

- GitHub Actions reported that `actions/checkout@v4`, `actions/setup-node@v4`,
  and `actions/upload-artifact@v4` target a Node.js 20 runtime that is deprecated
  and currently forced to run on Node.js 24. This is not blocking now, but action
  versions should be revisited when newer versions are available.
- GitHub Actions reported that `macos-latest` will migrate to macOS 26. If
  release stability becomes sensitive to macOS runner changes, pin the runner to
  a specific macOS version.
- Linux artifact size is materially larger than macOS and Windows artifacts:
  `ai-limits-linux-unsigned` was 80,837,949 bytes in the verified run.
- Downloaded artifacts used for verification were stored in `/private/tmp`, which
  is not durable storage.
- Artifact install/open verification has not been completed yet.
