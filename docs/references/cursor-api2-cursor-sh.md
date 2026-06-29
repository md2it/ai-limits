# Cursor api2.cursor.sh

## Status

Research document. Describes an unofficial way to retrieve Cursor usage/limits via the `api2.cursor.sh` backend.

---

## Problem

For personal Pro/Ultra/Team plans, a public official usage API is not confirmed. However, the access token obtained after `cursor agent login` allows retrieving current limits via an endpoint used by the Cursor client backend.

---

## Primary Verified Endpoint

Endpoint:

```text
POST https://api2.cursor.sh/aiserver.v1.DashboardService/GetCurrentPeriodUsage
```

Headers:

```text
Authorization: Bearer <access_token>
Content-Type: application/json
Connect-Protocol-Version: 1
```

Body:

```json
{}
```

Example response:

```json
{
  "planUsage": {
    "remaining": 2000,
    "limit": 2000,
    "autoPercentUsed": 0,
    "apiPercentUsed": 0,
    "totalPercentUsed": 0
  },
  "displayMessage": "You've used 0% of your included usage",
  "billingCycleStart": "1782614703000",
  "billingCycleEnd": "1785206703000"
}
```

Fields:

- `planUsage.remaining` — remaining included usage
- `planUsage.limit` — included usage limit
- `planUsage.totalPercentUsed` — total usage percentage
- `planUsage.autoPercentUsed` — Auto usage percentage
- `planUsage.apiPercentUsed` — API models usage percentage
- `billingCycleStart` — billing cycle start in Unix ms
- `billingCycleEnd` — billing cycle end in Unix ms
- `displayMessage` — human-readable Cursor message

---

## Tokens

On macOS after `cursor agent login`, tokens can be found in Keychain:

```sh
security find-generic-password -s cursor-access-token -w
security find-generic-password -s cursor-refresh-token -w
```

Access token is short-lived. On `401`, a new login or refresh via the Cursor OAuth endpoint is required.

Refresh token is a sensitive secret. The application must not read, log, or refresh it without a separate security review and an explicit user scenario.

---

## Alternatives

| Option | Plan/availability | Status | Comment |
|---|---|---|---|
| IDE backend `api2.cursor.sh` | Pro/Ultra/Team | Implemented in PoC | Uses access token after `cursor agent login`; unofficial contract |
| Dashboard API `cursor.com/api/...` | Any | Research-only | Requires web session cookie; high security risk |
| Admin API `api.cursor.com` | Enterprise | Official | Suitable for Enterprise monitoring; on Pro/Teams without Enterprise expect 403 |

---

## Limitations

- `api2.cursor.sh` is not a publicly documented contract
- endpoint may change without notice
- response schema should be validated carefully
- dashboard cookies must not be used as the default product mechanism
- refresh token cannot be treated as ordinary application configuration

---

## Recommendation

For personal Pro/Ultra/Team scenarios, the primary candidate is `GetCurrentPeriodUsage` via Cursor Agent access token.

Before production use, a separate security review is required:

- which tokens are read
- where tokens are stored
- which data must not be logged or saved
- how to display an error when the token expires
- whether the application needs to perform refresh itself
