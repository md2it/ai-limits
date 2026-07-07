# CONTRIBUTING

> Run once after clone, and rerun after contributor tooling changes:
> `npm run setup:contributor`

---

## QUICK RULES

Use commit prefixes: `feat:`, `fix:`, `docs:`, `chore:`.

Keep documentation up to date when behavior, setup, or release process changes.

---

## DETAILS

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

The setup configures local Git hooks for this repository only. The commit hook
warns about missing prefixes but does not block the commit.

Release notes use commit prefixes:

- `feat:` and `fix:` are included in release descriptions.
- `docs:` and `chore:` are kept out of release descriptions by default.
- Existing Git history is not rewritten only to add prefixes.
