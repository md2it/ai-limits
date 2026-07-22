# Versioning

Use SemVer Git tags for GitHub Releases:

```text
vMAJOR.MINOR.PATCH
```

The GitHub `pre-release` flag carries release stability. Do not encode `stable`, `unstable`, or platform names in the tag.

The desktop build workflow enforces:

- release input matches `vMAJOR.MINOR.PATCH`;
- release asset names include the version and platform.
- `CHANGELOG.md` contains a non-empty `Unreleased` section.

The release version is entered when the workflow starts. The workflow moves the current `Unreleased` entries into that version, adds the release date and GitHub Release link, then creates an annotated tag and GitHub pre-release from the resulting changelog section.

Old `desktop-unstable-*` tags are historical and should not be used for new releases.
