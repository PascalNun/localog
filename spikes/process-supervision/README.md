# Process supervision spike

This crate is isolated from `src-tauri`. It validates process mechanics with a synthetic executable and must not be imported as production architecture.

## Hypothesis

- A Rust adapter can launch an executable directly with argument arrays, a controlled working directory, and an allowlisted environment.
- One process group can represent one supervised job, including descendants.
- Noisy line output can be consumed without blocking the child while progress is parsed, bounded, and throttled before crossing the UI boundary.
- Cancellation can request process-group termination, wait for a grace period, escalate to a forced kill, and retain bounded diagnostics.
- A single heavy-job lane can reject concurrent starts without becoming a workflow engine.

## Acceptance checks

- Receive typed progress while limiting high-frequency updates to roughly 10 per second.
- Keep stdout/stderr diagnostic tails bounded under output flooding.
- Cancel a worker and its descendant process group within a bounded time.
- Ignore malformed progress safely.
- Reject a second process in the single heavy-job lane.
- Return actionable missing-executable errors.
- Pass hostile-looking arguments literally without invoking a shell.

Run with:

```sh
cargo test
cargo build --release --bin synthetic-worker
cargo run --example measure --release
```

The keep/change decision and measurements are recorded in `docs/DECISIONS.md` after validation.
