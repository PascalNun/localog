import { describe, expect, it } from 'vitest';
import type { ModelTerms, ProtocolProviderModel } from '../workflow/types';
import { memoryClasses, modelLadder, withoutModel } from './modelLadder';
import { GENERATION_MODEL_CATALOG } from './modelCatalog';

const bundled = [
  ['gemma4-12b', 16],
  ['gemma4-31b', 32],
  ['qwen3.5-4b', 8],
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

const recommendedId = (memoryGb: number | null) =>
  modelLadder(bundled, [], memoryGb).find((model) => model.recommended)?.entry.id ?? null;

describe('the model ladder', () => {
  it('has no duplicate catalogue entries', () => {
    const ids = GENERATION_MODEL_CATALOG.map((entry) => entry.id);

    expect(new Set(ids).size).toBe(ids.length);
  });

  it('recommends the measured 4B on an 8 GB machine', () => {
    const ladder = modelLadder(bundled, [], 8);

    expect(ladder.find((model) => model.recommended)?.entry.id).toBe('qwen3.5-4b');
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

  it('takes the recommended model out of the list and drops a class left empty', () => {
    const ladder = modelLadder(bundled, [], 8);
    const classes = memoryClasses(ladder);
    const recommended = ladder.find((model) => model.recommended)?.entry.id;

    if (!recommended) throw new Error('expected a recommendation');

    expect(withoutModel(classes, recommended).map((group) => group.minimumMemoryGb)).toEqual([
      16, 24, 32,
    ]);
    expect(withoutModel(classes, 'qwen3.5-9b')).toHaveLength(4);
  });

  it('removes the recommended model but keeps the rest of its class', () => {
    const ladder = modelLadder(bundled, [], 16);
    const classes = memoryClasses(ladder);
    const recommended = ladder.find((model) => model.recommended)?.entry.id;

    if (!recommended) throw new Error('expected a recommendation');

    const more = withoutModel(classes, recommended);
    const sixteen = more.find((group) => group.minimumMemoryGb === 16);

    expect(more.map((group) => group.minimumMemoryGb)).toEqual([8, 16, 24, 32]);
    expect(sixteen?.models.map((model) => model.entry.id)).toEqual([
      'qwen3.5-9b',
      'ministral-3-14b',
    ]);
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

  it('has no recommendation below the smallest supported class', () => {
    expect(recommendedId(4)).toBeNull();
  });

  it('uses the bundled RAM requirement when it differs from the catalogue', () => {
    const narrowed = bundled.map((term) =>
      term.modelId === 'qwen3.5-4b' ? { ...term, minimumMemoryGb: 16 } : term,
    );
    const ladder = modelLadder(narrowed, [], 8);

    expect(ladder.find((model) => model.entry.id === 'qwen3.5-4b')?.fits).toBe(false);
  });
});
