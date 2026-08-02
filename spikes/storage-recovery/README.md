# Storage and recovery spike

This crate is intentionally isolated from `src-tauri`. It tests storage invariants; it is not production architecture and must not be imported by the application.

## Hypothesis

- SQLite can own identity, relationships, lifecycle, jobs, revision metadata, and artifact path/checksum records.
- A durably written immutable artifact can own the content of a committed transcript or protocol revision.
- Database-backed visibility after the durable file write prevents a partially committed revision from appearing after a crash.
- Startup reconciliation can identify incomplete writes, unreferenced files, missing/corrupt committed files, and interrupted jobs without modifying committed content.
- A structured immutable JSON artifact is a viable initial canonical transcript representation; mutable autosave remains a separate working file.

## Acceptance checks

- Inject a failure after file durability but before the database transaction and prove the revision is not visible.
- Detect the resulting unreferenced artifact on recovery.
- Mark queued/running/cancelling jobs interrupted while leaving terminal jobs unchanged.
- Detect incomplete, missing, and checksum-mismatched files.
- Preserve prior committed revisions while replacing mutable autosave state.
- Copy a synthetic original without modifying its source and verify the managed checksum on load.
- Reject hostile identifiers before constructing paths.
- Measure a synthetic long transcript commit, verified read, and recovery scan.

Run with:

```sh
cargo test
cargo run --example measure --release
```

The final keep/change result and measurements are recorded in `docs/DECISIONS.md`. Keep this crate isolated as an executable reference/fault-test oracle; rewrite the production storage module behind the application boundary.
