# Cursor Usage Retrieval Options

## Known usage retrieval options

| Option | Plan/availability | Status | Notes |
|---|---|---|---|
| IDE backend `api2.cursor.sh` | Pro/Ultra/Team | Implemented | Stable internal endpoint used by Cursor; uses an access token after `cursor agent login`; its contract is not publicly documented |
| Cursor CLI `status` | Pro/Ultra/Team | Optional pre-check | `cursor-agent status --format json` is local and reports identity and auth state only; usable to check "is the user logged in" before network calls, but carries no plan, price, limit, or usage data |
| Cursor CLI `about` | Pro/Ultra/Team | Rejected | `cursor-agent about --format json` reports `subscriptionTier`, but it obtains it over the network from `GetMe` and `GetPlanInfo`. It is not a local source and only duplicates `GetPlanInfo.planName` |
| Cursor CLI `acp` | Any | Rejected | Undocumented Agent Client Protocol over stdio; by specification it carries only `session/*`, `fs/*`, and `terminal/*`. No account, subscription, or limit methods exist in it |
| `cursor-agent-svc` daemon | Any | Rejected | Exposes only `health`, `register`, `heartbeat`, `drain`, `flush`, `snapshot`, `updateAuth`; no account or usage data |
| Local state (`state.vscdb`, sockets) | Any | Rejected | No plan cache in `state.vscdb`, and no local listening socket carries account data |
| Dashboard API `cursor.com/api/...` | Any | Research-only | Requires web session cookie; high security risk |
| Admin API `api.cursor.com` | Enterprise | Official | Suitable for Enterprise monitoring; 403 expected on Pro/Teams without Enterprise |

---

## Recommendation

For personal Pro/Ultra/Team, the only option is a locally authorized Cursor Agent and `api2.cursor.sh`. This is not a preference among alternatives: Cursor keeps no local copy of account, plan, or limit data, and the IDE itself calls the same backend. The endpoint is a stable internal Cursor endpoint, not a publicly documented provider contract, and requires a separate security review before production use.

For production/enterprise monitoring, the official Admin API is preferred when available for the plan and provides the required level of detail.

---

## Limitations

- `api2.cursor.sh` is a stable internal Cursor endpoint but has no publicly documented contract; `cursor.com/api/*` is also not publicly documented and either may change without notice
- the access token is short-lived
- the refresh token is a sensitive secret
- automated work with dashboard cookies should be disabled by default
- no local fallback exists, so an unreachable network means no Cursor data at all rather than degraded data
