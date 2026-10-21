import { describe, expect, it } from 'vitest';
import type { ModelTerms } from '../workflow/types';
import { managedModelOptions } from './managedModels';
const terms: ModelTerms[] = [
  ['gemma4-12b', 16],
  ['qwen3.5-4b', 8],
  ['ministral-3b', 8],
  ['qwen3.5-0.8b', 4],
].map(([id, ram]) => ({
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
describe('managed model guidance', () => {
  it('keeps oversized downloads visible but unavailable on the Surface', () => {
    const options = managedModelOptions(terms, 8);
    expect(options).toHaveLength(4);
    expect(options[0]?.model.modelId).toBe('qwen3.5-4b');
    expect(options[0]?.recommended).toBe(true);
    expect(options.find(({ model }) => model.modelId === 'gemma4-12b')?.fits).toBe(false);
    expect(options.find(({ model }) => model.modelId === 'qwen3.5-0.8b')?.recommended).toBe(false);
  });
  it('prefers a suitable installed model over another download', () => {
    const installed = terms.map((model) => ({
      ...model,
      installed: model.modelId === 'ministral-3b',
    }));
    const options = managedModelOptions(installed, 8);
    expect(options[0]?.model.modelId).toBe('ministral-3b');
    expect(options[0]?.recommended).toBe(true);
  });
  it('does not invent hardware recommendations when memory is unknown', () => {
    expect(managedModelOptions(terms, null).every((model) => !model.recommended)).toBe(true);
  });
  it('uses the backend RAM requirement and includes only verified managed entries', () => {
    const options = managedModelOptions(
      [
        { ...terms[1]!, minimumMemoryGb: 16 },
        { ...terms[0]!, modelId: 'unknown' },
      ],
      8,
    );
    expect(options).toHaveLength(1);
    expect(options[0]?.fits).toBe(false);
  });
});
