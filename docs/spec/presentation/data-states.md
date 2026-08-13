# User-Facing Limit Data States

This document defines how already obtained provider results map to user-facing states. It does not define data retrieval, source selection, or recovery controls.

- Available data: show the available limit records.
- No fresh data: show that no fresh usable limit records are available; do not present this as a technical source error.
- Unavailable or failed source: show the source failure or unavailability separately from no-fresh-data.

Each surface owns its wording, layout, and available follow-up actions for these states.
