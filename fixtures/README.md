# Synthetic fixtures only

This directory contains small, redistributable material made specifically for LocaLog tests and demonstrations.

Keep it safe to publish. Never add real meeting audio, client or project names, transcripts, protocols, user paths, model files, exports, databases, logs, secrets, or diagnostic bundles.

## The rule this directory states was broken once

Between 16 and 30 August 2026 the names of people, a firm and a project from the
reference recording were in the source — as test fixtures, as documentation
examples, and as the demo data the browser preview shows. They were on a public
repository for a fortnight.

The recordings themselves never left: `.gitignore` has always covered audio,
models, databases, transcripts and exports, and no tracked file has ever held a
local path. **What leaked was quotation.** The vocabulary and correction stages
were built against a real meeting, and its output was kept as the realistic
example — which is how a name gets from an ignored directory into a committed
test without anybody moving a file.

That is the failure mode worth remembering: the material stays local and its
_contents_ walk out in a code comment. A real name makes a better example than an
invented one, which is exactly why it is tempting and exactly why it is banned.

`src/lib/redactedNames.test.ts` now fails if any of them reappears. It reads its
list from `eval/`, which never leaves the machine — a denylist of real names in a
public repository would republish what it exists to keep out.
