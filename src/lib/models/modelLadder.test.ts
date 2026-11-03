import { describe, expect, it } from 'vitest';
import type { ModelTerms, ProtocolProviderModel } from '../workflow/types';
import { classDisclosure, memoryClasses, modelLadder } from './modelLadder';
import { GENERATION_MODEL_CATALOG } from './modelCatalog';

const bundled = [
  ['gemma4-12b', 16],
  ['gemma4-31b', 32],
  ['qwen3.5-4b', 8],
  ['ministral-3b', 8],
].map(([id, ram]): ModelTerms => ({
  modelId: String(id),
  minimumMemoryGb: Number(ram),
  kind: 'generation',
  byteCount: 1,
  licence: 'apache2',
  licenceUrl: 'https://www.apache.org/licenses/LICENSE-2.0',
  installed: false,
  acceptanceRequired: false,
  acceptedAtMs: null,
}));

const installed = (names: string[]): ProtocolProviderModel[] =>
  names.map((name) => ({ name, size: 1, digest: `sha256:${name}` }));

const recommendedClass = (memoryGb: number | null) =>
  modelLadder(bundled, [], memoryGb).find((model) => model.recommended)?.minimumMemoryGb ?? null;

describe('the model ladder', () => {
  it('has no duplicate catalogue entries', () => {
    const ids = GENERATION_MODEL_CATALOG.map((entry) => entry.id);

    expect(new Set(ids).size).toBe(ids.length);
  });

  it('recommends the measured 4B on an 8 GB machine, not the smaller 3B', () => {
    const ladder = modelLadder(bundled, [], 8);
    const recommended = ladder.find((model) => model.recommended);

    expect(recommended?.entry.id).toBe('qwen3.5-4b');
    expect(ladder.find((model) => model.entry.id === 'ministral-3b')?.recommended).toBe(false);
  });

  it('lists the classes in ascending order', () => {
    const classes = memoryClasses(modelLadder(bundled, [], 8));

    expect(classes.map((group) => group.minimumMemoryGb)).toEqual([8, 16, 24, 32]);
  });

  it('keeps the measured default when a larger unmeasured model is available', () => {
    const ladder = modelLadder(bundled, [], 32);

    expect(ladder.find((model) => model.entry.id === 'gemma4-12b')?.recommended).toBe(true);
    expect(ladder.find((model) => model.entry.id === 'gemma4-31b')?.recommended).toBe(false);
  });

  it('keeps a family’s rows together', () => {
    const group = memoryClasses(modelLadder(bundled, [], 16)).find(
      (each) => each.minimumMemoryGb === 16,
    );
    const ids = group?.models.map((model) => model.entry.id) ?? [];

    expect(ids).toEqual(['gemma4-12b', 'qwen3.5-9b', 'ministral-3-14b']);
  });

  it('shows only the machine’s own class and the measured default, with both directions disclosed', () => {
    const classes = (memoryGb: number) => memoryClasses(modelLadder(bundled, [], memoryGb));

    const eight = classDisclosure(classes(8), 8, recommendedClass(8));
    expect(eight.own.map((group) => group.minimumMemoryGb)).toEqual([8]);
    expect(eight.smaller).toEqual([]);
    expect(eight.larger.map((group) => group.minimumMemoryGb)).toEqual([16, 24, 32]);

    const sixteen = classDisclosure(classes(16), 16, recommendedClass(16));
    expect(sixteen.own.map((group) => group.minimumMemoryGb)).toEqual([16]);
    expect(sixteen.smaller.map((group) => group.minimumMemoryGb)).toEqual([8]);
    expect(sixteen.larger.map((group) => group.minimumMemoryGb)).toEqual([24, 32]);

    const thirtyTwo = classDisclosure(classes(32), 32, recommendedClass(32));
    expect(thirtyTwo.own.map((group) => group.minimumMemoryGb)).toEqual([16, 24, 32]);
    expect(thirtyTwo.smaller.map((group) => group.minimumMemoryGb)).toEqual([8]);
    expect(thirtyTwo.larger).toEqual([]);
  });

  it('shows every class in one list when memory is unknown', () => {
    const shown = classDisclosure(memoryClasses(modelLadder(bundled, [], null)), null, null);

    expect(shown.own).toHaveLength(4);
    expect(shown.smaller).toEqual([]);
    expect(shown.larger).toEqual([]);
  });

  it('offers only larger classes below the supported minimum', () => {
    const shown = classDisclosure(
      memoryClasses(modelLadder(bundled, [], 4)),
      4,
      recommendedClass(4),
    );

    expect(shown.own).toEqual([]);
    expect(shown.smaller).toEqual([]);
    expect(shown.larger.map((group) => group.minimumMemoryGb)).toEqual([8, 16, 24, 32]);
  });

  it('includes an Ollama-only model as a provider row', () => {
    const ladder = modelLadder(bundled, installed(['qwen3.5:9b']), 16);
    const row = ladder.find((model) => model.entry.id === 'qwen3.5-9b');

    expect(row).toMatchObject({ source: 'provider', installed: true, fits: true });
    expect(row?.provider?.name).toBe('qwen3.5:9b');
  });

  it('invents no hardware recommendation when memory is unknown', () => {
    expect(modelLadder(bundled, [], null).every((model) => !model.recommended)).toBe(true);
  });

  it('uses the bundled RAM requirement when it differs from the catalogue', () => {
    const narrowed = bundled.map((term) =>
      term.modelId === 'qwen3.5-4b' ? { ...term, minimumMemoryGb: 16 } : term,
    );
    const ladder = modelLadder(narrowed, [], 8);

    expect(ladder.find((model) => model.entry.id === 'qwen3.5-4b')?.fits).toBe(false);
  });
});
