import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';

/**
 * Names from the reference meeting must not come back.
 *
 * They were in the source from 16 August to 30 August 2026 — people, a firm and a
 * project, used as test fixtures, documentation examples and demo data. The
 * recordings never left the machine; `.gitignore` has always covered audio,
 * databases, transcripts and exports. What leaked was quotation: building the
 * vocabulary and correction stages against a real meeting, and keeping its output
 * as the realistic example.
 *
 * That is a mistake a search cannot prevent, because the names look like ordinary
 * test data once they are in. So it is a guard instead.
 *
 * ## Why the list is not in this repository
 *
 * A denylist of real names, committed to a public repository, republishes exactly
 * what it exists to keep out — and hashing them would not help, since a dozen
 * surnames fall to a dictionary in seconds. The list lives in `eval/`, which
 * `eval/.gitignore` keeps entirely local, next to the material it came from.
 *
 * Without that file this test skips rather than fails. A fresh clone has no
 * reference meeting and nothing to protect; the guard matters on the machine that
 * does.
 */
const LIST = join(process.cwd(), 'eval', 'redacted-names.txt');

function redacted(): string[] {
  if (!existsSync(LIST)) return [];
  return readFileSync(LIST, 'utf8')
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => line !== '' && !line.startsWith('#'));
}

/** Only tracked files: what is published is what git would push. */
function trackedFiles(): string[] {
  return execFileSync('git', ['ls-files'], { encoding: 'utf8' })
    .split('\n')
    .filter((path) => path !== '' && !path.endsWith('package-lock.json'));
}

describe('the reference meeting', () => {
  it('is not quoted anywhere a push would carry', () => {
    const names = redacted();
    if (names.length === 0) {
      // Said rather than silent: a guard that skips without saying so reads as a
      // guard that passed.
      console.log(`no ${LIST} on this machine — nothing to check against`);
      return;
    }
    const wanted = names.map((name) => name.toLowerCase());
    const found: string[] = [];
    for (const path of trackedFiles()) {
      let text: string;
      try {
        text = readFileSync(path, 'utf8').toLowerCase();
      } catch {
        continue; // A binary or unreadable file quotes nothing.
      }
      for (const [index, name] of wanted.entries()) {
        if (text.includes(name)) found.push(`${path} — entry ${index + 1}`);
      }
    }
    // The name itself is deliberately not in the failure message, so a CI log
    // does not become the leak this prevents. The line number is enough to find.
    expect(found).toEqual([]);
  });
});
