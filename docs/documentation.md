# Documentation

## Purpose

This document is a recommended guide for maintaining project documentation. It supports periodic documentation reviews and improvements to clarity, structure, and consistency. It is not a mandatory checklist for every change.

## Entry Points

- `README.md` gives a brief, clear explanation of the product value and directs readers to relevant documentation.
- `CONTRIBUTING.md` explains how contributors work with the project and directs them to development and testing guidance.
- Before changing code, contributors find and read the documentation for the affected area.

## Navigation and Structure

- The documentation tree is the primary navigation mechanism. A reader should be able to select the needed document from paths and names without opening unrelated files.
- Directories represent encapsulated and isolated system components or clear development processes. Nested directories represent nested logic, processes, or code components.
- Keep documents within the boundary of their directory. Do not expose internal component details outside that boundary unless another area needs the fact.
- Prefer a flat structure. Add a subdirectory only when it makes a meaningful boundary clearer.
- Keep cross-cutting application specifications at the `docs/spec/` root. Keep only documentation-wide guidance and non-specification materials at the `docs/` root.
- Avoid index documents. Use links only when they prevent duplication or lead to an entry point.
- Split a document when it covers more than one purpose or becomes difficult to use. Keep the split flat unless the parts form a clear group.

## Content Rules

- Each document has one clear purpose, a narrow scope, and enough context to serve that purpose on its own.
- Keep documents concise, specific, and replaceable. Removing or replacing one document must not break the overall documentation structure.
- Use one source of truth for each fact. Do not duplicate rules, contracts, or process details.
- Describe the current system. Do not preserve implementation history, legacy behaviour, or compatibility details unless they are required for current work.
- Describe behaviour, contracts, and requirements without binding them to a specific code implementation. Prefer meaning over code snippets and implementation details.
- An approved specification may temporarily describe the next implementation before code exists. It must be synchronized with the code when the work is completed, revised when the decision changes, and removed if the work is cancelled.
- Do not trust documentation blindly. When documentation and code disagree, clarify the expected behaviour with a person before updating either artifact.

## Keeping Documentation Current

- Apply KISS and DRY when changing documentation.
- Remove stale documentation when it no longer serves a current purpose. Prefer rewriting an existing relevant document to adding a new one. Add a document when new logic needs its own clear purpose.
- When code changes, review the documentation for the affected area and update it whenever it would otherwise become inaccurate.
- Update documentation for changes to behaviour, contracts, architecture, setup, operations, testing, or approved specifications.
- Periodically review the documentation tree for stale, excessive, duplicated, unclear, or misplaced material.
- Verify that changed documents match the agreed behaviour, relevant code, and their location in the tree.

## Style

- Write in concise English.
- Use no more than three heading levels: `#`, `##`, and `###`.
- Do not hard-wrap logical lines.
- Prefer lists to tables. Use tables only when they make a comparison, mapping, or repeated structure substantially clearer.
- Use links sparingly and only for a clear reader need or to avoid duplication.
