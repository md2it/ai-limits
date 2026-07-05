# Tauri UI Refresh

Provider blocks should render immediately when the UI opens. Empty data is acceptable while a provider has not returned data yet.

Each provider block refreshes independently:

- initial load starts refreshes for enabled providers in parallel
- `UPDATE ALL NOW` starts refreshes for enabled providers in parallel
- `UPD MANUALLY` in one provider block refreshes only that provider
- scheduled refresh runs only for the provider whose interval fired
- a slow or failed provider must not block other provider blocks from updating
- each block owns its own loading, updated, and failed status
- global loading should not hide or block provider blocks

The preferred integration model is one Tauri request per provider. The frontend should not call a combined all-provider request and then wait for the slowest provider before updating the screen.

## Boundaries

- UI must not duplicate provider-fetching logic.
- UI must not decide real limit semantics.
- Future integration should use structured data from the Rust core through Tauri commands.
