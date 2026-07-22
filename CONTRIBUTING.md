# Contributing

Keep changes focused and update the relevant documentation when behaviour or setup changes.

## Changelog

Add each user-visible change to the `Unreleased` section of [CHANGELOG.md](CHANGELOG.md). Write short statements that users can understand; omit internal refactoring, routine maintenance, and documentation-only changes.

The release workflow takes the version entered at launch and automatically:

- moves `Unreleased` entries to the versioned section;
- adds the release date and GitHub Release link;
- creates a new empty `Unreleased` section;
- uses the versioned section for the annotated Git tag and GitHub Release notes.

Do not create or edit release sections manually.

## Testing

Use [Testing](docs/testing.md) as the entry point for applicable test guidance.
