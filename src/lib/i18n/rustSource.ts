/**
 * The Rust source, as text, for the guards that check it against the dictionary.
 *
 * Only the tests import this. It lives beside them rather than inside one of them
 * because two guards need the same two things — which Rust files to read, and how
 * to ignore the parts of them that never ship — and the second guard was written
 * on 30 August 2026, when the first one's copy would have become the second one's
 * copy.
 */
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';

const RUST = join(process.cwd(), 'src-tauri', 'src');

/**
 * The source with its `#[cfg(test)]` items removed, so unshipped code does not count.
 *
 * A test fixture may name a stage or raise a failure that no real run can reach —
 * `imports.rs` raises one deliberately — and holding the dictionary to those would
 * be asking it to have words for something nobody can ever see.
 */
export function shipped(source: string): string {
  let kept = '';
  let rest = source;
  for (;;) {
    const at = rest.indexOf('#[cfg(test)]');
    if (at === -1) break;
    kept += rest.slice(0, at);
    const after = rest.slice(at);
    const open = after.indexOf('{');
    if (open === -1) break;
    let depth = 0;
    let end = -1;
    for (let offset = open; offset < after.length; offset += 1) {
      if (after[offset] === '{') depth += 1;
      else if (after[offset] === '}') {
        depth -= 1;
        if (depth === 0) {
          end = offset + 1;
          break;
        }
      }
    }
    if (end === -1) break;
    rest = after.slice(end);
  }
  return kept + rest;
}

/** Every `.rs` file under `src-tauri/src`, shallow and one level down. */
export function everyRustSource(): { path: string; text: string }[] {
  const found: string[] = [];
  for (const entry of readdirSync(RUST, { withFileTypes: true })) {
    if (entry.isFile() && entry.name.endsWith('.rs')) found.push(join(RUST, entry.name));
    if (entry.isDirectory()) {
      for (const inner of readdirSync(join(RUST, entry.name))) {
        if (inner.endsWith('.rs')) found.push(join(RUST, entry.name, inner));
      }
    }
  }
  return found.map((path) => ({ path, text: readFileSync(path, 'utf8') }));
}

/**
 * The files a stage can be reported from: the pipeline and the provider it drives.
 *
 * Not every Rust file. `lib.rs` names its Tauri event channels the same way a stage
 * with a detail is written — `job:{meeting_id}` — and reading it made that guard fail
 * on two event names. The command layer does not report stages; the pipeline does.
 */
export function pipelineSources(): string[] {
  const pipeline = join(RUST, 'processing');
  return everyRustSource()
    .filter(
      ({ path }) =>
        path === join(RUST, 'provider.rs') ||
        path === join(RUST, 'processing.rs') ||
        path.startsWith(pipeline),
    )
    .map(({ text }) => text);
}
