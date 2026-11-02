import { describe, expect, it } from 'vitest';
import { GENERATION_MODEL_CATALOG, hardwareTierForMemory } from './modelCatalog';
import { en } from '../i18n/en';

describe('generation model catalogue', () => {
  it('uses the conservative baseline when memory is unknown', () => {
    expect(hardwareTierForMemory(null)).toBe('baseline');
    expect(hardwareTierForMemory(8)).toBe('baseline');
    expect(hardwareTierForMemory(16)).toBe('standard');
    expect(hardwareTierForMemory(32)).toBe('larger');
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
