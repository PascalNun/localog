# Markdown editing and autosave study

This isolated TypeScript study tested the state behind the protocol writing surface. It is not imported by the application and does not decide the final editor technology.

It covers canonical Markdown, separate working autosaves, debounced and sequenced saves, failed-save recovery, stale acknowledgements, conservative plain-text export, and long synthetic documents.

Run:

```sh
npm run test:editor-spike
npm run check:editor-spike
```

The study deliberately leaves the final post-review lifecycle decision to the application decision log. No meeting content is stored here.
