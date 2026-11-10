import { describe, expect, it } from 'vitest';
import { catalogueEntryFor, recommendGeneration } from './generationChoice';
import { GENERATION_MODEL_CATALOG } from './modelCatalog';
import type { ModelTerms } from '../workflow/types';

/**
 * Real catalogue identifiers, deliberately. The memory a model wants is no longer
 * carried on the fixture — it is read from the catalogue — so a test that invented
 * its own identifiers would be testing a lookup that always misses.
 */
function downloadable(modelId: string, byteCount: number): ModelTerms {
  return {
    modelId,
    kind: 'generation',
    byteCount,
    licence: 'gemma',
    licenceUrl: 'https://ai.google.dev/gemma/terms',
    installed: false,
    acceptanceRequired: true,
    acceptedAtMs: null,
  };
}

const LARGE = downloadable('gemma4-12b', 6_975_879_296); // catalogue: 16 GB
const SMALL = downloadable('qwen3.5-4b', 3_400_000_000); // catalogue: 8 GB
const BOTH = [LARGE, SMALL];

describe('recommendGeneration', () => {
  it('takes the largest model the machine has room for', () => {
    expect(recommendGeneration(BOTH, 32)).toBe('gemma4-12b');
    expect(recommendGeneration(BOTH, 16)).toBe('gemma4-12b');
    expect(recommendGeneration(BOTH, 8)).toBe('qwen3.5-4b');
  });

  // Somebody on a machine smaller than anything we ship still needs an answer.
  it('falls back to the smallest when none of them fit', () => {
    expect(recommendGeneration(BOTH, 4)).toBe('qwen3.5-4b');
  });

  // WebKit does not implement navigator.deviceMemory, so this is the macOS case
  // rather than an edge one, and the backend says null where the browser says
  // undefined.
  it('reads an unreported machine as the conservative baseline', () => {
    expect(recommendGeneration(BOTH, undefined)).toBe('qwen3.5-4b');
    expect(recommendGeneration(BOTH, null)).toBe('qwen3.5-4b');
  });

  it('ignores models that are not for generation', () => {
    const transcription = { ...downloadable('base', 147_951_465), kind: 'transcription' as const };
    expect(recommendGeneration([transcription], 32)).toBeUndefined();
    expect(recommendGeneration([transcription, ...BOTH], 32)).toBe('gemma4-12b');
  });

  it('does not depend on the order it is given', () => {
    expect(recommendGeneration([SMALL, LARGE], 8)).toBe('qwen3.5-4b');
    expect(recommendGeneration([SMALL, LARGE], 32)).toBe('gemma4-12b');
  });

  // What the lookup does if a model reaches it that the catalogue has never
  // described. Not recommended rather than guessed at — and bundledCatalogue
  // fails before anybody can ship one.
  it('does not recommend a model the catalogue does not describe', () => {
    const stranger = downloadable('not-in-the-catalogue', 1_000_000_000);
    expect(recommendGeneration([LARGE, stranger], 32)).toBe('gemma4-12b');
  });
});

describe('catalogueEntryFor', () => {
  it('finds the entry the interface reads a model from', () => {
    expect(catalogueEntryFor('gemma4-12b')?.name).toBe('Gemma 4 12B');
    expect(catalogueEntryFor('ministral-8b')?.origin).toBe('european');
    expect(catalogueEntryFor('not-in-the-catalogue')).toBeUndefined();
  });

  it('agrees with the catalogue it reads from', () => {
    for (const entry of GENERATION_MODEL_CATALOG) {
      expect(catalogueEntryFor(entry.id)).toBe(entry);
    }
  });
});
