# Local Files Data Quality

## Source Discovery and Scan

The diagram below describes local-file discovery and scan behavior.

```mermaid
stateDiagram-v2
    [*] --> Limit_request

    Limit_request --> Provider_selected
    Provider_selected --> Method_selected
    Method_selected --> Source_discovery

    Source_discovery --> No_roots: No roots found
    Source_discovery --> Roots_found: One or more roots found

    Roots_found --> Scan_files
    Scan_files --> Parse_records
    Parse_records --> Normalize_data

    Normalize_data --> Limits_shown_to_user
    No_roots --> Limits_unavailable

    Limits_shown_to_user --> [*]
    Limits_unavailable --> [*]
```

---

## Data Quality and Freshness

These source-specific rules feed the shared usable-limit decision documented in [../data-validation.md](../data-validation.md).

- data quality must include source type, timestamp of latest relevant record, and confidence level
- if the latest relevant record is older than the configured staleness threshold, mark data as stale
- if a reliably parsed automatic limit reset is more than two minutes in the past, reject the whole local current-limit snapshot
- if files exist but no relevant records are found, return a clear `no data found` result
- if roots are missing, return a clear `source not found` result with searched roots
