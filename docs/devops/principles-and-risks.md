# Principles and Risks

## GitHub Actions Principles

- Start with manual workflows.
- Use native runners per platform.
- Keep platform builds explicit and publish only after all required platform jobs pass.
- Do not require DMG for the first macOS GitHub Actions success.
- Keep signing and notarization secrets limited to the explicit macOS GitHub Actions path.
- Keep platform-specific commands explicit if a matrix makes the workflow hard to read.

## Artifact Principles

- GitHub Actions already proves that downloadable desktop artifacts can be produced.
- macOS GitHub Actions now supports Apple Developer ID signing.
- Artifact names should be stable and human-readable.
- Artifact paths must remain based on actual GitHub Actions output, not assumptions.
- macOS `.app` is archived before upload so the bundle structure is preserved.
- GitHub Actions artifact retention is currently 14 days.
- Unstable desktop builds are published through GitHub pre-releases for easier collaborator access.
- Long-term stable release artifacts should be handled through stable GitHub Releases, not `/private/tmp`.

## Release Principles

- Use the term `unstable` for current desktop pre-releases.
- Keep release titles short and avoid repeating the repository name or full tag.
- A pre-release may be useful and downloadable while still being incomplete, unsigned, and bug-prone.
- Publish separate release assets per operating system so users download only what they need.
- Do not present unstable pre-releases as stable or store-ready.
- Do not present a macOS pre-release as notarized unless the workflow ran in `full` mode.

## Known Warnings and Risks

- GitHub Actions reported that `actions/checkout@v4`, `actions/setup-node@v4`, and `actions/upload-artifact@v4` target a Node.js 20 runtime that is deprecated and currently forced to run on Node.js 24. This is not blocking now, but action versions should be revisited when newer versions are available.
- GitHub Actions reported that `macos-latest` will migrate to macOS 26. If release stability becomes sensitive to macOS runner changes, pin the runner to a specific macOS version.
- Linux artifact size is materially larger than macOS and Windows artifacts:
  `ai-limits-linux-unsigned` was 80,837,949 bytes in the verified run.
- Downloaded artifacts used for verification were stored in `/private/tmp`, which is not durable storage.
- Artifact install/open verification has not been completed yet.
