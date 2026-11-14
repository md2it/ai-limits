# Linux Permissions

Allowed target:

- Native desktop notifications where supported.
- Provider-specific local data paths.
- Network only for providers that require a network source.
- External browser open only for documented setup links.

Linux permissions must be reviewed before stable Linux distribution.

## Cursor session recovery

Allow read-only access to the current user's standard Cursor CLI credential file and the two Cursor IDE credential keys, plus API2 dashboard reads and `/oauth/token` session recovery, only as defined in [Cursor session recovery](../../get-limits/providers/cursor-auth.md). This access allows routine token expiry to recover without another sign-in. Do not write vendor credential stores, extract browser credentials, or persist copies of Cursor secrets. Verify platform paths and access on the target OS before distribution.

Cursor IDE credential reads execute the system `sqlite3` utility directly with a constant read-only query. No shell is used. If sqlite3 is absent, IDE fallback is unavailable; CLI credential reads remain independent.
