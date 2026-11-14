# Cursor Usage Retrieval Options

## Source selection

Use authenticated `DashboardService` calls on `api2.cursor.sh` for Cursor plan, limits, and usage. This is an internal account-data interface with no published stability guarantee. Model calls, API keys, and browser cookies are excluded from this integration.

- An active Cursor CLI sign-in is required. Run `cursor agent login` and complete browser sign-in. AI Limits uses the saved session; the CLI need not stay running. Discovery and recovery follow [cursor-auth.md](cursor-auth.md)
- CLI `status` and `about` are not usage sources or refresh commands; the inspected CLI can call `GetMe` from `status`, and `GetMe` plus `GetPlanInfo` from `about`
- ACP and the agent service do not supply the required account usage contract
- Local state can supply credentials, but no verified local plan/limit cache replaces the dashboard calls
- Web dashboard endpoints requiring cookies are excluded
- The official Enterprise Admin API is a separate integration, not a fallback for this session-based source

The [official CLI authentication documentation](https://cursor.com/docs/cli/reference/authentication) documents browser login, status, and logout, but no command that guarantees silent refresh or exports a fresh access token. Do not make scheduled CLI execution a prerequisite for API2 session recovery.

---

## Verification and implementation gap

Live verification on macOS: after Cursor CLI sign-in, the local collector returned real plan and limit data. A separate refresh exchange issued a new token, and `GetPlanInfo` accepted it. Without valid authorization, the API rejects dashboard requests.

Implementation and remaining verification:

- The collector discovers IDE and CLI credentials, uses bounded session recovery with HTTP/Connect-RPC classification, and keeps recovered secrets only in memory
- IDE SQLite reads use the system sqlite3 utility; there is no added dependency, and IDE credentials are unavailable when this utility is absent, notably on a standard Windows installation
- CLI JSON credential discovery and platform paths have automated coverage; actual Windows/Linux operation and their CLI refresh compatibility remain unverified
- Desktop transient failures retain the previous same-account snapshot and timestamp; explicit account changes or terminal session rejection clear it
- IDE/CLI source choice is available in Settings for the current application session, without exposing account identifiers
- The optional missing token component remains unknown; neither a zero nor a complete total is invented
- Live concurrency with a simultaneously refreshing IDE and revoked-session UI checks remain release acceptance work; fake transport tests cover bounded recovery and terminal rejection

Passing dashboard reads establishes current availability, not future contract stability. Keep sanitized response-shape fixtures and run the provider-specific acceptance checks before declaring a platform or session flow supported. Team/enterprise, on-demand spend scale, and subscription-field limitations remain as recorded in [cursor-api2-usage.md](cursor-api2-usage.md).

The IDE stores its session in protected containers. AI Limits does not decrypt them or send them to API2. If IDE was explicitly selected, choose Use CLI in Settings after signing in.
