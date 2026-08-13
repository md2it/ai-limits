# Get Limits Scenario

`get_limits/` follows the document [methods/overview.md](methods/overview.md).

Purpose:

- select enabled provider methods
- call provider methods in the right order
- apply provider fallback-chain logic for default and best-source runs
- apply the desktop CLI-first source chain
- assemble a shared result for the desktop and CLI

Boundaries:

- does not contain terminal output
- does not contain low-level process execution
- does not contain low-level HTTP primitives
- does not parse provider-specific output when that is a provider method's responsibility

## Code

```text
get_limits/
  mod.rs
  plan.rs
  chain.rs
  freshness.rs
```

- `mod.rs` — thin public facade (`get_source_plan_limits` / `get_source_limits`) and re-exports
- `plan.rs` — `SourcePlan`, UI plan options, and chain constants
- `chain.rs` — fallback-chain runner and usable/unusable report selection
- `freshness.rs` — local snapshot expiry policy for Codex/Claude local sources
