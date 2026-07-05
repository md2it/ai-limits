# Tauri UI Time Display

All user-facing times in the Tauri UI are displayed in the local time zone of the user's device.

The application must convert timestamps from the core data into device-local time before rendering them.

For today, show only time:

```text
20:48
```

For another date, show date and time:

```text
Jul 6, 01:49
```

Do not show `UTC+3` or another timezone suffix in the UI.

Backend timestamp fields:

- provider source timestamp: `dataTimestamp`.
- limit reset timestamp: `limits[].resetTime`.

Frontend timestamp handling:

- parse timestamp-like strings, numbers, and `Date` values where possible.
- render today's timestamps as `HH:MM`.
- render other days as `MMM D, HH:MM`.
- strip timezone suffixes if a value cannot be parsed as an instant.
- display `unknown` for missing provider `dataTimestamp`.
