# CONTRIBUTING

> Run once after clone, and rerun after contributor tooling changes:
> `npm run setup:contributor`

---

## QUICK RULES

- Set up local contributor tooling with `npm run setup:contributor`.
- Use commit prefixes: `feat:`, `fix:`, `docs:`, `chore:`.
- Keep documentation up to date when behavior, setup, or release process changes.

---

## DETAILS

### Contributor Setup

Run the contributor setup after cloning the repository:

```text
npm run setup:contributor
```

Run it again when contributor tooling or contribution rules change.

The setup configures Git hooks for this repository only:

```text
git config --local core.hooksPath scripts/git-hooks
```

This keeps hook scripts versioned in the repository and avoids copying files into
`.git/hooks`. If hook scripts change, `git pull` gets the new version.

The setup does not change global Git settings. If a different local
`core.hooksPath` already exists, the setup stops and explains what to do instead
of overwriting custom hooks silently.

### Commit Messages

Use a commit prefix so release notes can be generated automatically.

Allowed prefixes:

- `feat:` - user-visible feature or meaningful product improvement.
- `fix:` - bug fix or correction of broken behavior.
- `docs:` - documentation, texts, or explanatory materials.
- `chore:` - maintenance, build, dependencies, or internal cleanup.

If one commit fits several categories, use this priority:

```text
feat > fix > docs > chore
```

Examples:

```text
feat: add release notes generation
fix: correct desktop release asset name
docs: update GitHub releases guide
chore: refresh build dependencies
```

Keep the first line short and clear. Describe what changed, not only that
something was updated.

You can check a commit message locally:

```text
scripts/check-commit-message.sh .git/COMMIT_EDITMSG
```

Local commit hook setup:

```text
npm run setup:contributor
```

The local commit hook blocks commits with missing prefixes. GitHub Actions only
prints warnings and does not block builds or pull requests for this check.

Release notes use commit prefixes:

- `feat:` and `fix:` are included in release descriptions.
- `docs:` and `chore:` are kept out of release descriptions by default.
- Existing Git history is not rewritten only to add prefixes.
