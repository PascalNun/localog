import { describe, expect, it } from 'vitest';
import type { ModelTerms, ProtocolProviderModel } from '../workflow/types';
import { classDisclosure, memoryClasses, modelLadder } from './modelLadder';
import { GENERATION_MODEL_CATALOG } from './modelCatalog';

const bundled = [
  ['gemma4-12b', 16],
  ['gemma4-31b', 32],
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
  it('has no duplicate catalogue entries', () => {
    const ids = GENERATION_MODEL_CATALOG.map((entry) => entry.id);

    expect(new Set(ids).size).toBe(ids.length);
  });
  it('puts the measured 4B above the unevaluated 3B on an 8 GB machine', () => {
    const ladder = modelLadder(bundled, [], 8);
    const recommended = ladder.find((model) => model.recommended);

    expect(recommended?.entry.id).toBe('qwen3.5-4b');
    expect(ladder.find((model) => model.entry.id === 'ministral-3b')?.recommended).toBe(false);
    expect(ladder.find((model) => model.entry.id === 'qwen3.5-0.8b')?.recommended).toBe(false);
  });

  it('orders the visible classes upward, then the 4 GB fallback', () => {
    const classes = memoryClasses(modelLadder(bundled, [], 8));

    expect(classes.map((group) => group.minimumMemoryGb)).toEqual([8, 16, 24, 32, 4]);
  });

  it('keeps the recommendation on the measured model when a larger one is available', () => {
    const ladder = modelLadder(bundled, [], 32);

    expect(ladder.find((model) => model.entry.id === 'gemma4-12b')?.recommended).toBe(true);
    expect(ladder.find((model) => model.entry.id === 'gemma4-31b')?.recommended).toBe(false);
  });

  it('keeps a family’s rows together, with the recommendation first', () => {
    const group = memoryClasses(modelLadder(bundled, [], 16)).find(
      (each) => each.minimumMemoryGb === 16,
    );
    const ids = group?.models.map((model) => model.entry.id) ?? [];

    expect(ids[0]).toBe('gemma4-12b');
    expect(ids[1]).toBe('gemma4-e4b');
    expect(ids.indexOf('qwen3.5-9b')).toBeGreaterThan(ids.indexOf('gemma4-e4b'));
    expect(ids.indexOf('ministral-3-14b')).toBeGreaterThan(ids.indexOf('qwen3.5-9b'));
    expect(ids.indexOf('ministral-8b')).toBe(ids.indexOf('ministral-3-14b') + 1);
  });

  it('keeps larger classes behind a disclosure and the test model last', () => {
    const classes = memoryClasses(modelLadder(bundled, [], 8));
    const shown = classDisclosure(classes, 8);

    expect(shown.own.map((group) => group.minimumMemoryGb)).toEqual([8]);
    expect(shown.larger.map((group) => group.minimumMemoryGb)).toEqual([16, 24, 32]);
    expect(shown.fallback.map((group) => group.minimumMemoryGb)).toEqual([4]);
  });

  it('shows every class in one list when memory is unknown', () => {
    const shown = classDisclosure(memoryClasses(modelLadder(bundled, [], null)), null);

    expect(shown.own).toHaveLength(5);
    expect(shown.larger).toEqual([]);
    expect(shown.fallback).toEqual([]);
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
