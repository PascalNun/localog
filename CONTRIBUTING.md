# Contributing to LocaLog

LocaLog is being developed in the open while its foundations are still moving. A good contribution makes the central workflow clearer, safer, faster, or more useful. It does not add surface area simply because the technology makes it possible.

## Start with the product

Before changing behaviour or architecture, read:

1. [README.md](README.md)
2. [docs/PRODUCT.md](docs/PRODUCT.md)
3. [docs/UX.md](docs/UX.md)
4. [docs/MVP.md](docs/MVP.md)
5. [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
6. [docs/DECISIONS.md](docs/DECISIONS.md)
7. [docs/PLAN.md](docs/PLAN.md)

For interface work, also read [docs/VISUAL_DIRECTION.md](docs/VISUAL_DIRECTION.md). The documentation guide explains the role of each document.

## How we work

Keep work small enough to understand and large enough to prove something real.

- Start with the user's task: moving from a recording to a useful, trustworthy protocol.
- State the intended outcome, the acceptance criteria, and anything that is not yet decided.
- Prefer a coherent vertical slice over a general framework for future features.
- Keep the meeting as the unit of work. A recording belongs to a meeting, and a meeting belongs to a project.
- Keep product rules independent from Tauri, Svelte, operating-system APIs, and individual model runtimes.
- Keep technical settings out of the normal workflow. Advanced details should be available when needed, not placed in front of everyone.
- Treat interface quality as equal to privacy and data reliability. Calm spacing, clear language, keyboard use, empty states, failures, and recovery are part of the feature.
- Keep commits small, descriptive, and focused on one change.
- Record a product or architecture change in [docs/DECISIONS.md](docs/DECISIONS.md), and update [docs/PLAN.md](docs/PLAN.md) when implementation status changes.

An assigned task is not permission to change an accepted product decision silently.

## Implementation style

Aim for code that is lean, legible, proportionate, and responsive.

- Prefer clear names, focused functions, explicit data flow, and ordinary control structures.
- Comment why an invariant, recovery rule, safety boundary, or platform difference exists. Do not narrate code that is already self-explanatory.
- Use section headings in long files so a reader can find the main responsibilities quickly.
- Split modules when a real boundary has emerged. Do not introduce a workflow engine, provider SDK, export crate, or large dependency graph for hypothetical future needs.
- Remove dead experiments from production modules. Keep useful research in a spike or evaluation harness.
- Keep media work, inference, migrations, large file operations, and synchronous database work off the interface thread.
- Bound progress events, logs, process output, queues, and retries. Ordinary logs must never contain transcript, protocol, or audio content.
- Optimise the architecture first. Measure a real bottleneck before reaching for low-level optimisation.
- Treat failure, cancellation, interruption, and recovery as normal states that deserve a clear path.

## Privacy and test data

Never commit or post:

- real recordings, transcripts, protocols, names, client information, or project information;
- local databases, exports, logs, diagnostic bundles, credentials, or user paths;
- model weights or runtime binaries;
- third-party material without recorded provenance and compatible publication rights.

Use synthetic, redistributable fixtures. Real evaluation material belongs in the ignored `eval/` directory and should stay on the machine where it was collected.

## Runtimes and dependencies

Keep dependencies limited and justify additions. Prefer standard Rust and platform capabilities when they are sufficient.

External processes use argument arrays rather than shell interpolation. Runtime paths, working directories, and environment variables are controlled. Model and runtime downloads are explicit, consent-gated, checksummed, and never hidden in ordinary workflow actions.

The application is intended for macOS, Windows, and Linux. Keep platform-specific paths, process handling, permissions, acceleration, signing, recording, and packaging behind focused boundaries.

## Quality checks

Add tests at real risk boundaries rather than chasing a coverage percentage. Storage, jobs, runtimes, cancellation, recovery, malformed output, hostile paths, and disk failures deserve tests. Interface changes need keyboard and focus checks in both themes and at compact widths.

From the repository root:

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
```

The isolated spikes have their own READMEs and checks. Real-runtime tests are normally explicit or ignored because they require local tools and models.

## Licence

LocaLog is licensed under `GPL-3.0-or-later`. Contributions are accepted under the same terms unless a separate written agreement exists. Third-party code, assets, models, and runtimes retain their own licences; see [LICENSE-NOTES.md](LICENSE-NOTES.md).
