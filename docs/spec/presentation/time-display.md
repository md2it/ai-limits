# User-Facing Time Display

## Code

Shared timestamp parsing and local display live in `src/presentation/time/`:

```text
presentation/time/
  mod.rs     — thin facade (`TimeContext`, `format_user_timestamp`, `format_user_date`) and package tests
  parse.rs   — ISO/unix and source-specific parsing, timezone suffix split, roll policies with assemblers
  format.rs  — local display labels and unparsed-value timezone-suffix stripping
```

The package root keeps the public facade; `parse` and `format` stay private to the package. Display rules stay here, not in providers.

## Rules

All user-facing times are displayed in the local time zone of the user's device.

The application must convert timestamps from the core data into device-local time before rendering them.

This rule applies to Tauri UI, terminal UI, and system notification text.

Do not show `UTC+3` or another timezone suffix in user-facing surfaces.

## Two display forms

There are two user-facing forms. Which one applies is decided by the field class, not by the surface.

### Timestamp form

Used for **moments** — values whose time of day matters and which are always near the present.

For today, show an explicit `today` label followed by the time:

```text
today 20:48
```

For another date, show date and time:

```text
Jul 6, 01:49
```

Rendered by `format_user_timestamp`.

### Date form

Used for **calendar dates** — values whose time of day is meaningless and which may be far from the present in either direction.

Always show the year, never show a time component:

```text
Jan 12, 2026
```

Rendered by `format_user_date`.

The year is mandatory in this form. A subscription can have started years ago, and an annual plan can renew up to a year out, so a `MMM D` label without a year would read as the current year and misinform the user.

## Field classes

| Field | Form |
| --- | --- |
| provider source timestamp: `dataTimestamp` | timestamp |
| limit reset timestamp: `limits[].resetTime` | timestamp |
| `account.subscription_started_at` | date |
| `account.renewal_at` | date |

The subscription fields back the Plan output kind; see [output-kinds.md](output-kinds.md).

The formatted value is only the date-time value. Surrounding text is owned by the surface that renders it:

- Tauri source line: `as of {time}`.
- terminal source line: `Source {source}: {time}`.
- terminal limit row: `reset {time}`.
- notification body: `reset {time}`.
- Tauri Plan section: `Started {date} · renews {date}`.

Timestamp handling:

- parse supported timestamp-like strings, numbers, and `Date` values where possible.
- render today's timestamps as `today HH:MM`.
- render other days as `MMM D, HH:MM`.
- render subscription dates as `MMM D, YYYY`.
- strip timezone suffixes if a value cannot be parsed as an instant; both forms degrade the same way.
- display `unknown` for missing provider `dataTimestamp`.

Both forms share one parse path, so a value that parses as an instant for one form parses for the other; they differ only in how the parsed local value is labelled.
