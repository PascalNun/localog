import { describe, expect, it } from 'vitest';
import { everyRustSource, shipped } from '../i18n/rustSource';
import { GENERATION_MODEL_CATALOG } from './modelCatalog';

/**
 * The backend downloads models; this side describes them. Nothing enforced the
 * join, and on 2 September 2026 nothing needed to, because there was one model.
 *
 * A second one arrived with its name, its origin and the memory it wants written
 * out twice — once in `models.rs` and once here — which is the shape of every
 * fault this session found: one fact in two places, agreeing on the day it was
 * written. Those duplicates are gone, and this is what keeps them gone. The two
 * halves now split cleanly: the backend owns the file, this side owns the words,
 * and the only thing they must agree on is which models exist and under what
 * licence, because the licence decides whether a download stops for consent.
 *
 * The parse is deliberately narrow. It reads the `MODELS` table as text, which is
 * enough while every entry is a literal — a property of the table rather than a
 * promise, and if that ever stops holding this finds fewer entries rather than
 * failing on correct code.
 */
function bundledGenerationModels(): { id: string; licence: string; acceptance: boolean }[] {
  const source = everyRustSource().find((file) => file.path.endsWith('models.rs'));
  if (!source) throw new Error('models.rs is no longer where this guard looks for it');
  const entries: { id: string; licence: string; acceptance: boolean }[] = [];
  for (const block of shipped(source.text).matchAll(/ModelSpec \{([\s\S]*?)\n {4}\}/g)) {
    const body = block[1] ?? '';
    if (!body.includes('ModelKind::Generation')) continue;
    const id = /id: "([^"]+)"/.exec(body)?.[1];
    const licence = /licence: "([^"]+)"/.exec(body)?.[1];
    if (!id || !licence) continue;
    entries.push({ id, licence, acceptance: /acceptance_required: true/.test(body) });
  }
  return entries;
}

describe('the models this application downloads itself', () => {
  // If this finds nothing it is passing for the wrong reason, so it says so.
  it('are found by this guard at all', () => {
    expect(bundledGenerationModels().length).toBeGreaterThan(0);
  });

  it('are all described by the catalogue the interface reads', () => {
    for (const model of bundledGenerationModels()) {
      const entry = GENERATION_MODEL_CATALOG.find((candidate) => candidate.id === model.id);
      expect(
        entry,
        `${model.id} is downloadable but the catalogue does not describe it`,
      ).toBeTruthy();
    }
  });

  // The licence decides whether the download stops for consent, and the interface
  // names it on the card. Disagreeing means a card offering Apache 2.0 above a
  // backend that refuses until somebody accepts Google's terms.
  it('carry the licence the catalogue says they do', () => {
    for (const model of bundledGenerationModels()) {
      const entry = GENERATION_MODEL_CATALOG.find((candidate) => candidate.id === model.id);
      expect(entry?.licence, `${model.id}`).toBe(model.licence);
    }
  });

  // Apache-2.0 grants its permissions without anybody agreeing to anything, so a
  // consent step in front of it is a step that means nothing. Gemma's terms are an
  // agreement, so skipping it would be a licence step somebody can walk past.
  it('stop for consent exactly when the licence is an agreement', () => {
    for (const model of bundledGenerationModels()) {
      expect(model.acceptance, `${model.id} (${model.licence})`).toBe(model.licence !== 'apache2');
    }
  });
});
