import { describe, expect, it } from 'vitest';
import type { ModelTerms, ProtocolProviderModel } from '../workflow/types';
import { memoryClasses, modelLadder } from './modelLadder';

const bundled = [
  ['gemma4-12b', 16],
  ['qwen3.5-4b', 8],
  ['ministral-3b', 8],
  ['qwen3.5-0.8b', 4],
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

describe('the model ladder', () => {
  it('puts the measured 4B above the unevaluated 3B on an 8 GB machine', () => {
    const ladder = modelLadder(bundled, [], 8);
    const recommended = ladder.find((model) => model.recommended);

    expect(recommended?.entry.id).toBe('qwen3.5-4b');
    expect(ladder.find((model) => model.entry.id === 'ministral-3b')?.recommended).toBe(false);
    expect(ladder.find((model) => model.entry.id === 'qwen3.5-0.8b')?.recommended).toBe(false);
  });

  it('orders the visible classes 8, 16, then the 4 GB fallback', () => {
    const classes = memoryClasses(modelLadder(bundled, [], 8));

    expect(classes.map((group) => group.minimumMemoryGb)).toEqual([8, 16, 4]);
  });

  it('does not recommend the smoke-test 0.8B as the only 4 GB fit', () => {
    const ladder = modelLadder(bundled, [], 4);
    const small = ladder.find((model) => model.entry.id === 'qwen3.5-0.8b');

    expect(small?.fits).toBe(true);
    expect(ladder.every((model) => !model.recommended)).toBe(true);
  });

  it('includes an Ollama-only model as a provider row', () => {
    const ladder = modelLadder(bundled, installed(['ministral-3:8b']), 16);
    const row = ladder.find((model) => model.entry.id === 'ministral-8b');

    expect(row).toMatchObject({ source: 'provider', installed: true, fits: true });
    expect(row?.provider?.name).toBe('ministral-3:8b');
  });

  it('keeps a planned model from looking selectable', () => {
    const ladder = modelLadder(bundled, [], 16);
    const row = ladder.find((model) => model.entry.id === 'llama-8b');

    expect(row).toMatchObject({ source: 'planned', installed: false, provider: null });
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
