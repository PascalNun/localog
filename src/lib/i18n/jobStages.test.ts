import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import { chooseLanguage, jobErrorDetail, jobErrorTitle, stageText } from './index';
import { en } from './en';

/**
 * Nothing connects the string a step reports to the words that render it, and the two
 * are now in different languages as well as different files. `writing_subject` was
 * once renamed to `writing_section` and its label was not, so the longest phase of
 * writing a protocol showed "Working" — the fallback — while three shorter notices
 * about the result showed nothing at all.
 *
 * This guard used to live in `storage.rs` and matched the text of a function. It
 * moved here on 29 August 2026 when the words did, and it is stronger for the move:
 * it checks the codes against the real dictionary object rather than against source
 * text that happened to contain them.
 */
const RUST = join(process.cwd(), 'src-tauri', 'src');

/** The source with its test-gated items removed, so unshipped stages do not count. */
function shipped(source: string): string {
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

/**
 * Every stage code this can see the pipeline reporting.
 *
 * **Deliberately not a complete list, and the direction of that matters.** A stage can
 * also be written into a SQL literal, chosen by a branch into a variable, or built by
 * `format!` in a way no pattern here will match. Missing one means this guard checks
 * less — never that it fails on correct code — which is the only direction an
 * incomplete extractor is safe in.
 *
 * The reverse check, "words that no stage emits", was written and removed for exactly
 * this reason: it needs a complete list to be true, and it flagged seven live stages
 * on its first run.
 */
function reported(source: string): string[] {
  const found: string[] = [];
  // Reported at the call site.
  for (const call of source.matchAll(/\b(?:progress|report)\(\s*[^;]{0,120}?,\s*"([a-z_]+)"/g)) {
    if (call[1]) found.push(call[1]);
  }
  // Built with its live detail: `format!("finding_subjects:{} of {}", …)`. The
  // placeholder has to close, or this matches the JSON keys in the prompts too.
  for (const built of source.matchAll(/"([a-z_]+):\{[a-z_]*\}/g)) {
    if (built[1]) found.push(built[1]);
  }
  return found;
}

/**
 * The files a stage can be reported from: the pipeline and the provider it drives.
 *
 * Not every Rust file. `lib.rs` names its Tauri event channels the same way a stage
 * with a detail is written — `job:{meeting_id}` — and reading it made this guard fail
 * on two event names. The command layer does not report stages; the pipeline does.
 */
function rustSources(): string[] {
  const files = [
    join(RUST, 'provider.rs'),
    join(RUST, 'processing.rs'),
    ...readdirSync(join(RUST, 'processing'))
      .filter((name) => name.endsWith('.rs'))
      .map((name) => join(RUST, 'processing', name)),
  ];
  return files.map((path) => readFileSync(path, 'utf8'));
}

describe('every stage the pipeline reports', () => {
  it('has words for it, so none of them reads as “Working”', () => {
    const known = new Set(Object.keys(en.jobStages));
    const missing = new Set<string>();
    for (const source of rustSources()) {
      for (const stage of reported(shipped(source))) {
        if (!known.has(stage)) missing.add(stage);
      }
    }
    expect([...missing].sort()).toEqual([]);
  });
});

describe('rendering a stage', () => {
  it('is said in the language the interface is in', () => {
    chooseLanguage('en');
    expect(stageText('transcribing_audio')).toBe('Transcribing');
    chooseLanguage('de');
    expect(stageText('transcribing_audio')).toBe('Wird transkribiert');
    chooseLanguage('en');
  });

  it('puts a live detail into the words, because a long step must move', () => {
    expect(stageText('finding_subjects:3 of 13')).toBe(
      'Finding what was discussed — passage 3 of 13',
    );
    expect(stageText('finding_subjects')).toBe('Finding what was discussed');
  });

  it('splits on the first colon only', () => {
    expect(stageText('joining_failed:the reply was not valid: line 3')).toContain(
      'the reply was not valid: line 3',
    );
  });

  it('falls back to “Working” for a stage nobody wrote words for', () => {
    expect(stageText('a_stage_nobody_named')).toBe('Working');
  });
});

describe('a job that failed', () => {
  it('names the failure and says what is safe, in the reader’s language', () => {
    chooseLanguage('en');
    expect(jobErrorTitle('provider_unavailable')).toBe(
      'Local protocol generation could not connect',
    );
    chooseLanguage('de');
    expect(jobErrorTitle('provider_unavailable')).toBe(
      'Die lokale Protokollerzeugung konnte keine Verbindung herstellen',
    );
    chooseLanguage('en');
  });

  /**
   * The bug this conversion fixed. Those stored values became codes on 27 August and
   * the progress panel prints the detail directly, so somebody whose generation had
   * failed was shown the literal word `responseUnusable`.
   */
  it('renders a code the failing step stored, rather than printing it', () => {
    expect(jobErrorDetail('provider_invalid_output', 'responseUnusable')).toContain(
      'could not use as a protocol',
    );
    expect(jobErrorDetail('provider_invalid_output', 'responseUnusable')).not.toContain(
      'responseUnusable',
    );
  });

  it('prefers what the step knew to what its class knows', () => {
    const general = jobErrorDetail('provider_model_missing', '');
    const particular = jobErrorDetail('provider_model_missing', 'ollamaModelGone');
    expect(particular).not.toBe(general);
    expect(general).toContain('Ollama');
  });

  it('falls back to a sentence rather than to nothing', () => {
    // Somebody whose work has just stopped needs words more than anybody.
    expect(jobErrorTitle('a_failure_nobody_named')).toBe('Import could not finish');
    expect(jobErrorDetail('a_failure_nobody_named', '')).toContain('remains in Draft');
  });

  it('lets a plain sentence from an older build through unchanged', () => {
    const older = 'Something the previous build wrote out in full.';
    expect(jobErrorDetail('processing_failed', older)).toBe(older);
  });
});
