# Test boundaries

Cross-boundary and end-to-end tests belong here as the validated Rust adapters arrive. Phase 0 fake-workflow tests currently live beside the TypeScript boundary in `src/lib/workflow/` so behaviour and implementation remain reviewable together.

Fixtures must be synthetic and redistributable. Real recordings, transcripts, names, protocols, databases, model files, and confidential paths do not belong in tests or snapshots.
