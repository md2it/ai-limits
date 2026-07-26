# Get Limits Scenario

`get_limits.rs` follows the document [methods/overview.md](methods/overview.md).

Purpose:

- select enabled provider methods
- call provider methods in the right order
- apply the shared usable-limit decision from [data-validation.md](data-validation.md)
- apply provider fallback-chain logic for default and best-source runs
- apply desktop source priority logic for Fast, Full, and Best modes
- assemble a shared result for the desktop and CLI

Boundaries:

- does not contain terminal output
- does not contain low-level process execution
- does not contain low-level HTTP primitives
- does not parse provider-specific output when that is a provider method's responsibility
