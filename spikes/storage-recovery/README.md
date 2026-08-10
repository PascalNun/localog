# Storage and recovery study

This small isolated crate tested the storage idea before it was used in the application. It is a reference and fault-injection study, not a production module.

## What it asked

Can SQLite describe the relationships, lifecycle, jobs, revisions, and checksums while immutable files hold committed transcript and protocol content? Can the application recover safely if it stops between writing a file and recording it in the database?

## What it covered

- file durability before database visibility;
- unreferenced, missing, incomplete, and checksum-mismatched files;
- interrupted jobs;
- separate working autosaves and committed revisions;
- immutable imported originals;
- hostile identifiers and path containment;
- long synthetic documents and recovery scans.

Run:

```sh
cargo test
cargo run --example measure --release
```

The application keeps the tested invariants, but the study itself remains isolated so its fault-injection code cannot quietly become production architecture.
