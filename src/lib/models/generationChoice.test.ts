import { describe, expect, it } from 'vitest';
import { recommendGeneration } from './generationChoice';
import type { ModelTerms } from '../workflow/types';

function model(modelId: string, byteCount: number, wantsMemoryGb: number | null): ModelTerms {
  return {
    modelId,
    kind: 'generation',
    byteCount,
    licence: 'gemma',
    licenceUrl: 'https://ai.google.dev/gemma/terms',
    installed: false,
    acceptanceRequired: true,
    acceptedAtMs: null,
    wantsMemoryGb,
  };
}

const CATALOGUE = [
  model('gemma4-12b', 6_975_879_296, 16),
  model('gemma4-e4b', 5_154_941_280, 12),
  model('gemma4-e2b', 3_349_516_256, 8),
];

describe('recommendGeneration', () => {
  it('takes the largest model the machine has room for', () => {
    expect(recommendGeneration(CATALOGUE, 32)).toBe('gemma4-12b');
    expect(recommendGeneration(CATALOGUE, 16)).toBe('gemma4-12b');
    expect(recommendGeneration(CATALOGUE, 12)).toBe('gemma4-e4b');
    expect(recommendGeneration(CATALOGUE, 8)).toBe('gemma4-e2b');
  });

  // Somebody on a machine smaller than anything we ship still needs an answer.
  it('falls back to the smallest when none of them fit', () => {
    expect(recommendGeneration(CATALOGUE, 4)).toBe('gemma4-e2b');
  });

  // WebKit does not implement navigator.deviceMemory, so this is the macOS case
  // rather than an edge one.
  it('reads an unreported machine as the conservative baseline', () => {
    expect(recommendGeneration(CATALOGUE, undefined)).toBe('gemma4-e2b');
    // The backend says null where the browser says undefined.
    expect(recommendGeneration(CATALOGUE, null)).toBe('gemma4-e2b');
  });

  it('ignores models that are not for generation', () => {
    const transcription = { ...model('base', 147_951_465, null), kind: 'transcription' as const };
    expect(recommendGeneration([transcription], 32)).toBeUndefined();
    expect(recommendGeneration([transcription, ...CATALOGUE], 32)).toBe('gemma4-12b');
  });

  it('does not depend on the order it is given', () => {
    const shuffled = [
      model('gemma4-e2b', 3_349_516_256, 8),
      model('gemma4-12b', 6_975_879_296, 16),
      model('gemma4-e4b', 5_154_941_280, 12),
    ];
    expect(recommendGeneration(shuffled, 12)).toBe('gemma4-e4b');
  });
});
