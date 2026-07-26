# Contributing

Keep changes focused and update the relevant documentation when behaviour or setup changes.

## Entry points

- Start documentation navigation with [Documentation](docs/documentation.md); use [Architecture](docs/architecture.md) for code boundaries.
- Use [Runtime Architecture](docs/runtime-architecture.md) to understand runtime components, shared state, and information flows.
- Start code navigation with `src/lib.rs` for the shared core, `src/main.rs` for the CLI, and `src-tauri/src/main.rs` for the desktop app.
- [Testing](docs/testing/testing.md) as the entry point for applicable test guidance.

## Local development

Run the desktop app in development mode:

```sh
npm run tauri:dev
```

The dev command builds and opens the app once. It does not watch files or rebuild after code changes; stop and run the command again when you intentionally want a fresh build.

## Changelog

Add each user-visible change to the `Unreleased` section of [CHANGELOG.md](CHANGELOG.md). Write short statements that users can understand; omit internal refactoring, routine maintenance, and documentation-only changes.

The release workflow takes the version entered at launch and automatically:

- moves `Unreleased` entries to the versioned section;
- adds the release date and GitHub Release link;
- creates a new empty `Unreleased` section;
- uses the versioned section for the annotated Git tag and GitHub Release notes.

Do not create or edit release sections manually.
