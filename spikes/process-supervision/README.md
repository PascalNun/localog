# Process supervision study

This isolated crate tested how LocaLog should run local tools without freezing the interface or losing control of a child process. It is not imported into the application.

The study covered direct argument-array invocation, controlled working directories, bounded stdout/stderr, progress throttling, descendant process groups, cancellation with escalation, missing executables, hostile arguments, and a single heavy-work lane.

Run:

```sh
cargo test
cargo build --release --bin synthetic-worker
cargo run --example measure --release
```

The application keeps the narrow supervised-process boundary. It does not forward raw runtime output to the UI or ordinary logs.
