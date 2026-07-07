# User-Facing Time Display

All user-facing times are displayed in the local time zone of the user's device.

The application must convert timestamps from the core data into device-local time before rendering them.

This rule applies to Tauri UI, terminal UI, and system notification text.

For today, show only time:

```text
20:48
```

For another date, show date and time:

```text
Jul 6, 01:49
```

Do not show `UTC+3` or another timezone suffix in user-facing surfaces.

The timestamp format is only the date-time value. Surrounding text is owned by the surface that renders it:

- Tauri source line: `as of {time}`.
- terminal source line: `Source {source}: {time}`.
- terminal limit row: `reset {time}`.
- notification body: `reset {time}`.

Timestamp fields:

- provider source timestamp: `dataTimestamp`.
- limit reset timestamp: `limits[].resetTime`.

Timestamp handling:

- parse supported timestamp-like strings, numbers, and `Date` values where possible.
- render today's timestamps as `HH:MM`.
- render other days as `MMM D, HH:MM`.
- strip timezone suffixes if a value cannot be parsed as an instant.
- display `unknown` for missing provider `dataTimestamp`.
