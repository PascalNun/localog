# Contributing to LocaLog

LocaLog is public at an early stage. The product direction, interface quality, local-first promise, and data-safety rules are already intentional, while many implementation details are still being proven. A contribution should strengthen that direction rather than add surface area for its own sake.

## Begin with the project, not the code

Before changing behaviour or architecture, read:

1. [README.md](README.md)
2. [docs/PRODUCT.md](docs/PRODUCT.md)
3. [docs/UX.md](docs/UX.md)
4. [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
5. [docs/MVP.md](docs/MVP.md)
6. [docs/DECISIONS.md](docs/DECISIONS.md)

For interface work, also read [docs/VISUAL_DIRECTION.md](docs/VISUAL_DIRECTION.md). The [documentation guide](docs/README.md) explains the project’s recurring terminology.

Phase 0 validation is complete. The next work is Phase 1A: a small durable project and meeting workflow using fake processing runtimes and the existing interface boundary. Real media and model adapters enter one at a time afterwards; a full pipeline should not bypass the recorded spike decisions.

## Working principles

- Plan a bounded change and state its acceptance criteria.
- Prefer a coherent vertical slice over speculative abstractions.
- Record material product or architecture choices in `docs/DECISIONS.md` or a focused ADR.
- Keep the domain and application rules independent from Tauri, Svelte, operating-system APIs, and individual model runtimes.
- Treat the meeting as the unit of work. A recording never becomes an unassigned top-level object.
- Keep technical settings out of the primary workflow unless the product documentation requires them.
- Treat interface quality as equal to local-first behaviour and reliable data handling. Avoid generic dashboard, component-library, or AI-playground patterns.
- Keep changes and commits small, descriptive, and single-purpose.

Open a discussion before changing the project hierarchy, local-first promise, MVP scope, recording phase, repository licence, public location, or one of the contested alternatives in `docs/DECISIONS.md`. Networked services, telemetry, accounts, remote assets, or cloud dependencies always require an explicit product decision.

## Implementation style

Aim for code that is **lean, legible, proportionate, and responsive**.

- Build the smallest coherent solution that proves the current workflow. Do not turn a local application into a general platform before the product needs it.
- Prefer clear names, short focused functions, explicit data flow, and ordinary control structures over clever compression or hidden behaviour.
- Let code explain what happens. Use comments to explain **why** something is necessary: an invariant, safety boundary, recovery rule, platform difference, or non-obvious trade-off. Do not narrate every line or preserve outdated implementation history in comments.
- Keep modules focused, but split them only when a real boundary has emerged. Avoid universal workflow engines, public plugin systems, broad capability negotiation, or large crate/package graphs without evidence.
- Make dependencies earn their place. A small maintained dependency is reasonable when it removes real risk; an abstraction layer for a hypothetical future is not.
- Keep the interface thread free of media work, inference, migrations, large file operations, and synchronous database access. Ordinary navigation, selection, typing, and editing should generally feel immediate—around 100 ms or less—even while background work runs.
- Bound and throttle progress events, process output, logs, queues, and retries. Avoid keeping large recordings or transcripts in duplicate memory buffers when streaming or incremental work is practical.
- Optimise architecture before micro-optimising code: isolate heavy work, measure the real bottleneck, then improve it without making the code harder to understand than the gain justifies.
- Treat errors and cancellation as normal states. A clear failure path is more valuable than a compact happy path that hides what remains safe.
- Remove dead code and speculative options. If a future idea matters, document it in the roadmap or decision log rather than leaving half-built machinery in production modules.

## Privacy and test data

Never commit or post:

- real meeting recordings, transcripts, protocols, names, or client/project information;
- local databases, exports, logs, diagnostic bundles, secrets, or user paths;
- model weights or runtime binaries;
- third-party assets without documented provenance and compatible publication rights.

Fixtures, examples, screenshots, and issue reproductions must use synthetic material created for redistribution. Ordinary logs must not contain transcript or protocol content.

## Dependencies and local runtimes

Keep dependencies limited and explain why each is necessary. Prefer maintained, narrowly scoped libraries and standard platform or Rust capabilities where they are sufficient.

External processes use argument arrays rather than shell interpolation. Runtimes, binaries, and models must never be downloaded or bundled silently; record their provenance, version, checksum, size, licence, and consent requirements before distribution.

The shared application must remain portable across macOS, Windows, and Linux. Put platform-specific process handling, paths, permissions, audio capture, acceleration, signing, and packaging behind focused adapters.

## Tests and quality

Add tests at actual risk boundaries rather than pursuing an arbitrary coverage percentage. Domain and state changes need unit tests; storage, jobs, and runtime adapters need failure-oriented contract tests. When touching those boundaries, consider cancellation, restart recovery, malformed output, missing runtimes/models, hostile paths, and disk errors.

The interface must remain keyboard usable with visible focus and accessible names. Review important screens in light and dark modes and at compact widths. Empty, progress, failure, interruption, and recovery states require the same care as the happy path.

## Local checks

Install the pinned JavaScript dependencies with `npm install`. Rust is pinned by `rust-toolchain.toml` and the lockfiles.

```sh
npm run check
npm run check:editor-spike
npm run lint
npm test
npm run test:editor-spike
npm run build

cd src-tauri
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings

for spike in storage-recovery process-supervision media-transcription local-provider; do
  (cd "../spikes/$spike" && cargo fmt --check && cargo test && cargo clippy --all-targets -- -D warnings)
done
```

`npm run tauri dev` runs the native shell. Tauri may regenerate `src-tauri/gen/`; that directory is ignored because the capability schemas are build output.

## Licence

LocaLog is licensed under `GPL-3.0-or-later`. By submitting a contribution, you agree to license that contribution under the same terms and confirm that you have the right to do so. Copyright remains with the respective contributor unless a separate written agreement says otherwise.

Third-party code, generated material, models, or copied examples must not be added unless their provenance and compatible licence are documented. See [LICENSE-NOTES.md](LICENSE-NOTES.md).
