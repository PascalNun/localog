# Markdown editing and autosave spike

This isolated TypeScript spike validates the state machine needed behind LocaLog's protocol writing surface. It is not imported by the application and does not decide the detailed editor implementation.

## Scope

- Canonical Markdown remains unchanged during editing and Markdown export.
- Working-state autosaves are debounced, single-flight, sequenced, and separate from immutable revision commits.
- At most one in-flight text snapshot and one newer current value are retained by the session.
- Failed saves remain visibly dirty and can be retried without losing the current text.
- Stale or malformed acknowledgements cannot mark newer content as saved.
- Plain-text export follows a deliberately conservative, deterministic transformation.
- Long synthetic documents exercise edit latency, save coalescing, and export cost.

The spike deliberately does not decide whether editing a reviewed revision moves the meeting to `protocol_draft` or to a distinct changed-since-review presentation. That remains an approval question.

## Run

From the repository root:

```sh
npm run test:editor-spike
npm run check:editor-spike
```

All fixtures are generated synthetic text. No meeting content is stored in this directory.
