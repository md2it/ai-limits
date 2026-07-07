# Versioning

Use SemVer Git tags for GitHub Releases:

```text
vMAJOR.MINOR.PATCH
```

The GitHub `pre-release` flag carries release stability. Do not encode `stable`, `unstable`, or platform names in the tag.

The desktop build workflow enforces:

- release input matches `vMAJOR.MINOR.PATCH`;
- release tag matches `src-tauri/tauri.conf.json` version with the `v` prefix;
- release asset names include the version and platform.

Old `desktop-unstable-*` tags are historical and should not be used for new releases.
