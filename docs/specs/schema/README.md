# JSON Schemas for the `.sltwx` family

This folder holds the published JSON Schemas (draft 2020-12) for the files inside a `.sltwx`, `.sltwxa` or `.sltwxd` container (`docs/specs/FILE_FORMAT.md`, Versioning).

- One file per document part and `format_version`, named `<part>-<MAJOR.MINOR>.json`, for example `manifest-0.1.json`.
- The schemas are written in the canonical JSON style (2-space indent, keys sorted).
- The Rust readers in `crates/swx-io` are the reference: for `manifest.json` that is `read_manifest` (`crates/swx-io/src/manifest.rs`). `crates/swx-io/tests/manifest_schema.rs` checks the schema against it.
- `model-part-0.1.json`: a part's `model.json`, format 0.1. Uniqueness and sort order of IDs can't be expressed in a schema; `read_unpacked` enforces them, and `crates/swx-io/tests/model_schema.rs` checks the schema against it.
