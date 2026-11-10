import { describe, expect, it } from 'vitest';
import {
  GENERATION_MODEL_CATALOG,
  hardwareTierForMemory,
  modelStatus,
  recommendationFor,
} from './modelCatalog';
import { en } from '../i18n/en';

describe('generation model catalogue', () => {
  it('uses the conservative baseline when memory is unknown', () => {
    expect(hardwareTierForMemory(null)).toBe('baseline');
    expect(hardwareTierForMemory(8)).toBe('baseline');
    expect(hardwareTierForMemory(16)).toBe('standard');
    expect(hardwareTierForMemory(32)).toBe('larger');
  });

  it('prefers the measured baseline when it is installed', () => {
    const result = recommendationFor(
      [{ name: 'qwen3.5:4b', size: 3_400_000_000, digest: 'sha256:qwen' }],
      8,
    );

    expect(result.entry.id).toBe('qwen3.5-4b');
    expect(result.installed?.name).toBe('qwen3.5:4b');
  });

  it('does not recommend an 8B model to an unknown or baseline machine', () => {
    const result = recommendationFor(
      [{ name: 'granite4.1:8b', size: 5_300_000_000, digest: 'sha256:granite' }],
      null,
    );

    expect(result.entry.minimumMemoryGb).toBe(8);
    expect(result.installed).toBeNull();
  });

  it('can use an installed 8B comparison model on a standard machine', () => {
    const result = recommendationFor(
      [{ name: 'granite4.1:8b', size: 5_300_000_000, digest: 'sha256:granite' }],
      16,
    );

    expect(result.entry.id).toBe('granite4.1-8b');
    expect(result.installed?.name).toBe('granite4.1:8b');
  });

  it('recognises an installed model and distinguishes planned candidates', () => {
    const granite = GENERATION_MODEL_CATALOG.find((entry) => entry.id === 'granite4.1-8b');
    const ministral = GENERATION_MODEL_CATALOG.find((entry) => entry.id === 'ministral-8b');

    expect(granite).toBeDefined();
    expect(ministral).toBeDefined();
    expect(
      modelStatus(granite!, {
        name: 'granite4.1:8b',
        size: 5_300_000_000,
        digest: 'sha256:granite',
      }),
    ).toBe('installed');
    // Ministral 8B is installable and has been measured, so it is a candidate
    // rather than a plan. Llama is still the entry that has never been run.
    expect(modelStatus(ministral!, null)).toBe('notInstalled');
    const llama = GENERATION_MODEL_CATALOG.find((entry) => entry.id === 'llama-8b');
    expect(modelStatus(llama!, null)).toBe('plannedCandidate');
  });

  /**
   * The catalogue holds facts and the dictionary holds the words for them, which
   * only works while every fact has words. The id union makes a missing
   * description a compile error; these are the fields typing cannot reach,
   * because they are keys of an object the entry merely names.
   */
  it('has words for every fact it states', () => {
    for (const entry of GENERATION_MODEL_CATALOG) {
      expect(en.settings.modelDescription[entry.id]).toBeTruthy();
      expect(en.settings.modelOrigin[entry.origin]).toBeTruthy();
      expect(en.settings.modelLicence[entry.licence]).toBeTruthy();
      for (const code of [...entry.languages, ...entry.testedLanguages]) {
        expect(en.settings.modelLanguage[code]).toBeTruthy();
      }
    }
  });

  /**
   * A description is where a measurement is quoted, so it is where a measurement
   * goes stale. Naming the model in its own description is how the wrong one gets
   * pasted under the right heading.
   */
  it('does not name a different model in a model’s own description', () => {
    for (const entry of GENERATION_MODEL_CATALOG) {
      const said = en.settings.modelDescription[entry.id];
      const others = GENERATION_MODEL_CATALOG.filter((other) => other.family !== entry.family);
      for (const other of others) {
        expect(said).not.toContain(other.name);
      }
    }
  });
});
