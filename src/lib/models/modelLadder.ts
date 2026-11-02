import { GENERATION_MODEL_CATALOG } from './modelCatalog';
import type { GenerationModelEntry } from './modelCatalog';
import type { ModelTerms, ProtocolProviderModel } from '../workflow/types';

/**
 * How a catalogue entry can be reached.
 *
 * `bundled` is a model LocaLog can download and run itself. `provider` is a
 * model somebody installed in Ollama. `planned` is a future entry with neither
 * path yet, and it must never look selectable.
 */
export type ModelSource = 'bundled' | 'provider' | 'planned';

/** One row of the model ladder. */
export interface LadderModel {
  entry: GenerationModelEntry;
  minimumMemoryGb: number;
  fits: boolean;
  /** The bundled catalogue entry, when this model is downloadable here. */
  bundled: ModelTerms | null;
  /** The installed Ollama model, when one of its provider names is present. */
  provider: ProtocolProviderModel | null;
  source: ModelSource;
  installed: boolean;
  recommended: boolean;
}

/** A group of models that need the same memory class. */
export interface MemoryClass {
  minimumMemoryGb: number;
  models: LadderModel[];
}

/**
 * One catalogue, two runtimes.
 *
 * The bundled list and the Ollama list used to be drawn separately, which is how
 * an 8 GB machine could be shown an installed 3B in one place and a measured 4B
 * in another. The catalogue is the single source of truth; `terms` says whether
 * LocaLog can download an entry, and `providers` says whether Ollama already has
 * one. An entry with neither path is marked planned rather than offered.
 */
export function modelLadder(
  terms: ModelTerms[],
  providers: ProtocolProviderModel[],
  memoryGb: number | null,
): LadderModel[] {
  const models = GENERATION_MODEL_CATALOG.map((entry) => {
    const bundled =
      terms.find((term) => term.kind === 'generation' && term.modelId === entry.id) ?? null;
    const provider = providers.find((model) => entry.providerNames.includes(model.name)) ?? null;
    const minimumMemoryGb = bundled?.minimumMemoryGb ?? entry.minimumMemoryGb;
    const source: ModelSource = bundled
      ? 'bundled'
      : provider !== null || entry.providerNames.length > 0
        ? 'provider'
        : 'planned';
    return {
      entry,
      minimumMemoryGb,
      fits: memoryGb === null || memoryGb >= minimumMemoryGb,
      bundled,
      provider,
      source,
      installed: bundled?.installed === true || provider !== null,
      recommended: false,
    };
  });

  // The measured baseline comes first in the catalogue; later entries are the
  // rest of the family. Only an evaluated model can be recommended, and a
  // provider model that is merely named but not installed is not a
  // recommendation. The 0.8B exists so a 4 GB machine can run something at all.
  const recommended = models.find(
    (model) =>
      model.entry.id !== 'qwen3.5-0.8b' &&
      model.entry.testedLanguages.length > 0 &&
      model.fits &&
      (model.bundled !== null || model.provider !== null),
  );

  return models.map((model) => ({
    ...model,
    recommended: memoryGb !== null && model === recommended,
  }));
}

/**
 * The ladder, smallest useful class first and the development fallback last.
 *
 * The 4 GB class is deliberately after 8 and 16, because the 0.8B is a smoke-test
 * model rather than the first rung of a recommendation.
 */
export function memoryClasses(models: LadderModel[]): MemoryClass[] {
  const classes = new Map<number, LadderModel[]>();
  for (const model of models) {
    classes.set(model.minimumMemoryGb, [...(classes.get(model.minimumMemoryGb) ?? []), model]);
  }
  const order = (gb: number) => (gb < 8 ? Number.MAX_SAFE_INTEGER : gb);
  return [...classes.entries()]
    .sort(([left], [right]) => order(left) - order(right))
    .map(([minimumMemoryGb, models]) => ({ minimumMemoryGb, models: displayOrder(models) }));
}

/**
 * A class's rows, with each family kept together and the recommendation first.
 *
 * The catalogue's order is the recommendation order, which is not a reading
 * order: it places Ministral 8B between two Gemma rows and puts the 9B before the
 * 12B it is an alternative to. Sorting for display is safe because `recommended`
 * is decided before this runs.
 *
 * Families are ordered by where they first appear in the catalogue, which puts the
 * family holding the recommendation first on that machine without hard-coding a
 * preference. Ordering them by name instead scattered Qwen 4B, Ministral 3B and
 * Qwen 2B across the 8 GB class.
 */
export function displayOrder(models: LadderModel[]): LadderModel[] {
  const familyRank = new Map<string, number>();
  for (const [index, entry] of GENERATION_MODEL_CATALOG.entries()) {
    if (!familyRank.has(entry.family)) familyRank.set(entry.family, index);
  }
  const rank = (family: string) => familyRank.get(family) ?? Number.MAX_SAFE_INTEGER;

  return [...models].sort((left, right) => {
    const family = rank(left.entry.family) - rank(right.entry.family);
    if (family !== 0) return family;
    if (left.recommended !== right.recommended) return left.recommended ? -1 : 1;
    return left.entry.name.localeCompare(right.entry.name);
  });
}

/**
 * The classes a machine should see, and the larger ones behind a disclosure.
 *
 * The 4 GB class is separated from `own` so the smoke-test model stays last on
 * every machine, including a 4 GB one where it is technically the only fit.
 */
export interface ClassDisclosure {
  own: MemoryClass[];
  larger: MemoryClass[];
  fallback: MemoryClass[];
}

export function classDisclosure(classes: MemoryClass[], memoryGb: number | null): ClassDisclosure {
  if (memoryGb === null) return { own: classes, larger: [], fallback: [] };
  return {
    own: classes.filter((group) => group.minimumMemoryGb >= 8 && group.minimumMemoryGb <= memoryGb),
    larger: classes.filter((group) => group.minimumMemoryGb > memoryGb),
    fallback: classes.filter((group) => group.minimumMemoryGb < 8),
  };
}
