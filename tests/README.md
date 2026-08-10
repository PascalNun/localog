# Test boundaries

This directory is for tests that cross a real application boundary: storage and files, supervised runtimes, native commands, or a complete synthetic workflow.

The browser fake-workflow tests live beside the TypeScript boundary because they describe that boundary directly. The Rust unit tests live beside the modules they protect. Neither location changes the rule that tests should cover real risks rather than chase a percentage.

Fixtures must be synthetic and redistributable. Real recordings, transcripts, names, protocols, databases, model files, and confidential paths do not belong in tests, snapshots, or issue reports.
