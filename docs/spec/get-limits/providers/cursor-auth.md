# Cursor API2 Session Recovery

## Goal and boundary

An active Cursor CLI sign-in is required. Install Cursor CLI and run `cursor agent login` in a terminal, then complete the browser sign-in. The CLI does not need to stay open. Cursor plan and usage collection uses authenticated `DashboardService` methods on `api2.cursor.sh`. Routine access-token expiry is recovered inside the backend collection without a sign-in prompt or a failed provider event. API keys, browser cookies, model inference, and automatic browser login are excluded.

This is the target contract. Implementation readiness and remaining verification are recorded in [cursor-options.md](cursor-options.md#verification-and-implementation-gap). Usage field mapping remains in [cursor-api2-usage.md](cursor-api2-usage.md).

---

## Credential discovery and account continuity

Read only the access and refresh credentials from the standard Cursor installation for the current OS user:

- macOS CLI: Keychain services `cursor-access-token` and `cursor-refresh-token`, account `cursor-user`
- Windows native CLI: `%APPDATA%/Cursor/auth.json`, with `%USERPROFILE%/AppData/Roaming` as the base when `APPDATA` is unset
- Linux CLI: `$XDG_CONFIG_HOME/cursor/auth.json`, falling back to `~/.config/cursor/auth.json`
- Standard Cursor IDE: `User/globalStorage/state.vscdb` under `~/Library/Application Support/Cursor` on macOS, `%APPDATA%/Cursor` on Windows, or `$XDG_CONFIG_HOME/Cursor` with `~/.config/Cursor` fallback on Linux; read only `cursorAuth/accessToken` and `cursorAuth/refreshToken`

Windows and Linux paths are verification targets, not proof of working platform support. WSL uses Linux paths and the WSL user context; it does not silently read a Windows user's session. Custom profiles and alternative Cursor distributions are outside automatic discovery.

Use the system `sqlite3` executable directly, without a shell or an added library dependency, with `-readonly`, JSON output, a fixed query restricted to the two credential keys, and a short busy timeout. Open SQLite read-only without migrations or writes. If the utility is unavailable, explain that IDE credentials cannot be read and direct the user to Cursor CLI sign-in; the CLI JSON/Keychain source remains independent. Windows does not bundle this utility, so IDE fallback there requires it to be installed and available in PATH. Missing, locked, unreadable, and malformed stores are distinct fixed diagnostics. Never request broad filesystem access or run an arbitrary shell command to locate credentials.

Protected IDE credential containers are unavailable for collection: never decrypt them, send them to API2, or treat them as account identity. Use the CLI session when the IDE store contains a protected container and no account has been pinned. The remaining source-selection rules apply only when the collector actually has two readable credentials. Inspect only the local identity needed to detect differing accounts. JWT `sub` and `exp` are routing and scheduling hints, not proof of authentication; server authorization remains authoritative. If both stores identify different accounts, require an explicit source choice rather than silently selecting one. If identity cannot be established safely, do not merge or fall back between stores.

The manual source choice is IDE or CLI, uses the existing provider settings surface, and persists only for the application session; it never exposes credential contents. If the chosen source is unavailable, explain that condition rather than silently switching accounts.

Pin the selected account for the application session. Re-reading the selected store may adopt a newer session for that same account. Automatic fallback to another store is allowed only when the account matches. An account change clears the old account's limits and in-memory credentials before any new result is shown. Credentials and internal account identity never enter provider output, frontend events, or logs.

---

## Collection and refresh sequence

1. Re-read the selected credential store and compare it with the in-memory session; do not replace a successfully refreshed session with an older token still stored by Cursor
2. If the access token expires within five minutes, attempt session refresh before the dashboard request; a missing or unreadable expiry instead uses the server response to decide
3. Call `GetPlanInfo` as the access probe, followed by the documented dashboard sequence
4. On HTTP 401 or a parsed Connect-RPC `unauthenticated` code, re-read the selected store once and retry the failed method if a newer same-account token exists
5. Otherwise attempt one refresh exchange and retry the failed dashboard method once with the recovered session
6. Share one recovery budget across the entire collection, including later methods and event pages; recovery cannot recurse or restart pagination indefinitely

A missing refresh credential does not invalidate a still-accepted access token. If proactive refresh fails transiently while the access token is still valid, continue dashboard reads with that token and defer the next refresh until cooldown ends; only a final unsuccessful collection produces a failure state.

Successful proactive refresh consumes the same single refresh-exchange budget. A store-reload retry and a refresh retry are each allowed at most once. Parse error codes as JSON fields instead of substring matches. HTTP 403 and `permission_denied` are authorization restrictions, not evidence that the token expired; do not enter a refresh loop or require login solely because of them.

Use `POST https://api2.cursor.sh/oauth/token`, JSON content type, and the fields `grant_type: refresh_token`, `client_id`, and `refresh_token`. The public client identifier is the one verified in the official Cursor client for the selected credential flow; it is not an API key or a user secret. Validate IDE and CLI compatibility separately before enabling each flow.

A successful refresh returns a nonempty `access_token`; `shouldLogout: true` signals a terminal session rejection. Use an explicit replacement `refresh_token` when provided. The collector otherwise uses the returned access token as the next refresh credential, a provider-specific continuation rule rather than a generic OAuth rule. Verify consecutive refreshes before declaring a platform supported.

Reject malformed, empty, or unsuccessful refresh responses. HTTP status, parsed error code, `shouldLogout`, and response shape determine the outcome together. A timeout, 429, 5xx, or unknown payload is transient or incompatible, never an instruction to sign out. Respect a valid `Retry-After` for 429; otherwise defer another attempt for at least 60 seconds. Manual refresh does not bypass that cooldown. Each dashboard or refresh request has a 20-second timeout. Cancellation and a disabled Cursor provider stop queued work.

Keep recovered credentials only in backend memory and discard them on exit, account change, or sign-out detection. Do not rewrite Cursor's Keychain entries, JSON files, or SQLite database. After restart, re-read the vendor store and recover again if necessary. If live verification shows that this cannot survive refresh rotation or concurrent Cursor refreshes, the flow is not releasable until credential ownership and persistence are specified and verified.

Share a single in-flight refresh for the same account across CLI/backend callers within one process and all desktop surfaces. Before adopting a response, check that the selected account has not changed. Concurrent independent processes and the IDE are part of the required live validation, not a reason to overwrite vendor stores.

---

## User-visible outcomes

- A recovered expiry produces one successful collection and no intermediate `provider-refresh-failed`, sign-in prompt, browser opening, or error flash
- A transient failure retains the last successful desktop snapshot and its original `data_as_of`; show a short nonblocking update-failure message and retry through the existing provider scheduler after cooldown
- With no successful snapshot, show unavailable data with Retry; distinguish network failure, unreadable credentials, account choice, and sign-in required using fixed messages
- Missing credentials or a confirmed terminal session rejection directs the user to run `cursor agent login`; opening a sign-in flow requires a user action
- A 403 explains insufficient access without claiming that the user is signed out
- Account change or confirmed sign-out hides the previous account's snapshot; retained values must never appear to belong to a different account
- CLI collection remains bounded, returns a fixed diagnostic when recovery fails, and does not pretend that a desktop cached snapshot is a fresh API response

Keep the common structured schema unchanged. Recovery states remain internal to the provider; the final result uses existing status and diagnostic fields. Desktop recovery messages accompany the retained snapshot without changing its source timestamps. Limit notifications run only from a successful new collection, never from retained data after a failed attempt.

---

## Acceptance checks

Automated checks use fake clocks, credential stores, and transport responses; they do not need real credentials:

- Valid, near-expiry, expired, and malformed tokens; missing refresh credential; newer store token; older store token after successful in-memory refresh
- Successful refresh, terminal rejection, empty token, invalid JSON, 401, 403, Connect-RPC error, timeout, 429 cooldown, and 5xx
- One recovery budget across initial and later methods, shared concurrent refresh, cancellation, and account change while a refresh is in flight
- Different IDE/CLI accounts require selection; same-account fallback never mixes data; sign-out removes the prior snapshot
- Recovered expiry emits success only; transient failure preserves timestamps; retained data triggers no limit notification
- No token, refresh payload, email, or account identifier reaches stdout, stderr, raw data, process arguments, environment variables, diagnostics, or frontend events

Before enabling a credential flow, perform live checks with an explicitly authorized test session on macOS, Windows native, and Linux:

- Confirm store paths, file/Keychain access, account continuity, and successful dashboard access
- Confirm refresh without browser interaction, then a second refresh using the resulting session and recovery after process restart
- Exercise concurrent IDE/CLI activity without signing Cursor out or corrupting its credentials
- Verify all five dashboard methods and optional-field degradation against sanitized response-shape fixtures
- Confirm network loss and terminal revocation lead to the specified distinct UI states

Reading official client code is evidence of a protocol shape; it does not replace successful live refresh checks. Do not label an unverified OS or credential flow as supported.
