# Snapshot Store

This document defines the persisted provider snapshot contract, storage layout, replacement rules, and component boundaries.

Related:

- runtime relationships: [runtime-architecture.md](runtime-architecture.md)
- shared data validation: [get-limits/data-validation.md](get-limits/data-validation.md)
- shared structured schema: [get-limits/structured-info-schema.md](get-limits/structured-info-schema.md)
- macOS widget consumer: [widgets/architecture.md](widgets/architecture.md)

---

## Purpose

Snapshot Store keeps the latest acceptable structured limit data for each provider so a system widget can display data and Tauri can avoid redundant scheduled collection.

Snapshot Store is a cache. It is not the source of truth for current provider data and does not run providers, schedule updates, or define data validity.

---

## Producers and Consumers

Tauri and the planned Background Agent are snapshot producers. Both use the shared Rust core and may replace a snapshot after an acceptable live result.

The macOS Widget Extension, Tauri, and Background Agent are snapshot readers. Widget Extension reads snapshots for presentation. Tauri and Background Agent read snapshots before scheduled refresh and perform live collection only when the stored result cannot satisfy the request.

---

## Snapshot Contract

Each provider file contains the existing serialized `StructuredSourceInfo` directly.

The snapshot includes the complete structured result even though the initial widgets display only limits. It does not contain the surrounding `SourceData`, so raw provider responses and stderr are not persisted.

Conceptual shape:

```json
{
  "provider": "codex",
  "source": "codex_cli",
  "status": {},
  "account": {},
  "limits": [],
  "usage": {}
}
```

No outer envelope, failed-attempt history, error history, or separate snapshot schema version is stored. Snapshot age comes from the existing structured timestamps.

Snapshot readers ignore unknown fields and tolerate absent optional fields. If a future incompatible structured format cannot be decoded, the snapshot is treated as unavailable and replaced by a later acceptable collection.

---

## Replacement Rule

The writer replaces a provider snapshot only when the shared Rust core classifies the result as usable limit data according to [get-limits/data-validation.md](get-limits/data-validation.md).

Snapshot Store does not implement a separate validation policy. A failed or unusable result leaves the previous file untouched.

---

## Tauri Scheduled Reuse

Before a scheduled provider refresh, Tauri checks the stored snapshot.

The snapshot satisfies the refresh only when:

- it can be decoded
- its source is compatible with the selected source chain according to [get-limits/source-chains.md](get-limits/source-chains.md)
- its `collected_at` plus the configured Tauri interval is later than the current time

When the snapshot satisfies the refresh, Tauri maps it to its existing presentation model and schedules the next refresh for `collected_at` plus the configured interval.

When the snapshot does not satisfy the refresh, Tauri performs the normal live source chain and writes an acceptable result.

Manual `UPDATE NOW` and `UPDATE ALL NOW` actions bypass snapshot reuse and always perform live collection.

Background Agent reuse timing is owned by [background-agent.md](background-agent.md).

---

## macOS App Group

The shared container identifier is:

```text
group.md2it.ai-limits.shared
```

The directory layout is:

```text
App Group/
  snapshots/
    codex.json
    claude.json
    cursor.json
```

The App Group contains shared application state only. It must not contain raw provider responses, logs, credentials, authentication tokens, or unrelated application files.

Each provider has an independently replaceable file. This prevents one provider's read-modify-write cycle from overwriting another provider's result and isolates malformed data to one provider.

---

## Atomic Replacement

Each write uses a unique temporary file beside the destination and atomically replaces the destination only after serialization and validation succeed.

Readers must tolerate a missing file and a malformed document. They must never observe a partially written snapshot.

Tauri and Background Agent do not take a cross-process write lock. If acceptable results complete close together, the last atomic replacement wins. Small freshness differences between concurrent acceptable results are intentionally tolerated.

---

## Accepted Decisions

- use JSON while the store contains only current snapshots and does not require history, queries, or multi-record transactions
- use `group.md2it.ai-limits.shared` as the macOS App Group identifier
- keep one provider snapshot per file under `snapshots/`
- store serialized `StructuredSourceInfo` directly without an outer envelope
- write the complete structured result after shared validation
- keep failed attempts and error history out of snapshots
- make Widget Extension, Tauri, and Background Agent snapshot readers
- let scheduled Tauri refresh reuse a fresh source-compatible snapshot
- calculate the next Tauri refresh from the reused snapshot's `collected_at`
- make manual Tauri refresh bypass snapshot reuse
- leave the previous snapshot unchanged after a failed or unusable result
- use unique temporary files and atomic replacement without cross-process locking
- accept last-writer-wins behavior for concurrent acceptable results
