import type { ProtocolProviderModel } from '../workflow/types';

/**
 * A deliberately small catalogue for the normal model picker.
 *
 * The catalogue is a product boundary, not a marketplace. Entries can be
 * shown before they are downloadable so the user can see the planned path,
 * while selection still requires a verified model already exposed by the
 * active provider.
 *
 * ## Facts here, words in the dictionary
 *
 * Every entry once carried its own prose — a description, a size, an origin, a
 * licence, a list of languages — and on 30 August 2026 a French interface showed
 * all of it in English, because prose stored as data is prose no dictionary can
 * reach. What stays here is what is true in any language: which model this is,
 * what it needs, what was measured. The sentences live under `settings.model*`
 * and are keyed by the `id` below.
 *
 * `origin` is the clearest case of why. It used to be the string
 * `'European model'`, and the badge beside a model name was drawn by comparing
 * against that string — an identifier and a label in one value, which is the
 * fault this project has now found five times. Translating the label would have
 * silently stopped the badge appearing.
 */

/**
 * Every model the catalogue holds, as a closed set.
 *
 * A union rather than `string` so that the words for a model are checked the way
 * every other string is: `settings.modelDescription[entry.id]` does not compile
 * until each id here has a sentence, which is what stops a new entry shipping as
 * a blank card.
 */
export type GenerationModelId =
  'gemma4-12b' | 'ministral-8b' | 'qwen3.5-4b' | 'ministral-3b' | 'granite4.1-8b' | 'llama-8b';

/** A language a model claims, or `more` for "and many others". */
export type ModelLanguage = 'de' | 'en' | 'ja' | 'more';

/** Where a model comes from, as a fact rather than as the words for it. */
export type ModelOrigin = 'international' | 'european';

/** Which licence a model is offered under. */
export type ModelLicence = 'apache2' | 'gemma' | 'modelSpecific';

/** What the picker can say about a model right now. */
export type ModelStatus = 'installed' | 'notInstalled' | 'plannedCandidate';

export interface GenerationModelEntry {
  id: GenerationModelId;
  /** The model's own name, which is a product name and the same everywhere. */
  name: string;
  family: string;
  providerNames: string[];
  tier: 'baseline' | 'standard' | 'larger';
  minimumMemoryGb: number;
  /**
   * What it occupies once installed, where that has been measured. Null where it
   * has not, and the picker then says which class of model it is instead of
   * inventing a number.
   */
  installedGb: number | null;
  languages: ModelLanguage[];
  /** The languages it has been evaluated in *here*, which is a shorter list. */
  testedLanguages: ModelLanguage[];
  origin: ModelOrigin;
  licence: ModelLicence;
  status: 'baseline' | 'candidate' | 'planned';
}

export type HardwareTier = 'baseline' | 'standard' | 'larger';

/**
 * The first catalogue stays intentionally short. New models should enter it
 * only after their licence, runtime, memory use and German/English quality
 * have been checked.
 */
export const GENERATION_MODEL_CATALOG: GenerationModelEntry[] = [
  {
    id: 'gemma4-12b',
    name: 'Gemma 4 12B',
    family: 'Gemma 4',
    providerNames: ['gemma4:12b'],
    tier: 'standard',
    minimumMemoryGb: 16,
    installedGb: 8,
    languages: ['de', 'en', 'more'],
    testedLanguages: ['de'],
    origin: 'international',
    licence: 'gemma',
    status: 'baseline',
  },
  {
    id: 'ministral-8b',
    name: 'Ministral 3 8B',
    family: 'Ministral 3',
    providerNames: ['ministral-3:8b', 'ministral-3:8b-instruct-2512-q4_K_M'],
    tier: 'standard',
    minimumMemoryGb: 16,
    installedGb: null,
    languages: ['de', 'en', 'ja', 'more'],
    testedLanguages: ['de'],
    origin: 'european',
    licence: 'apache2',
    status: 'candidate',
  },
  {
    id: 'qwen3.5-4b',
    name: 'Qwen3.5 4B',
    family: 'Qwen3.5',
    providerNames: ['qwen3.5:4b'],
    tier: 'baseline',
    minimumMemoryGb: 8,
    installedGb: 3.4,
    languages: ['de', 'en', 'more'],
    testedLanguages: ['de'],
    origin: 'international',
    licence: 'apache2',
    status: 'candidate',
  },
  {
    id: 'ministral-3b',
    name: 'Ministral 3 3B',
    family: 'Ministral 3',
    providerNames: ['ministral-3:3b', 'ministral-3:3b-instruct-2512-q4_K_M'],
    tier: 'baseline',
    minimumMemoryGb: 8,
    installedGb: null,
    languages: ['de', 'en', 'ja', 'more'],
    testedLanguages: [],
    origin: 'european',
    licence: 'apache2',
    status: 'candidate',
  },
  {
    id: 'granite4.1-8b',
    name: 'Granite 4.1 8B',
    family: 'Granite 4.1',
    providerNames: ['granite4.1:8b'],
    tier: 'standard',
    minimumMemoryGb: 16,
    installedGb: 5.3,
    languages: ['de', 'en'],
    testedLanguages: ['de'],
    origin: 'international',
    licence: 'apache2',
    status: 'candidate',
  },
  {
    id: 'llama-8b',
    name: 'Llama 8B',
    family: 'Llama',
    providerNames: [],
    tier: 'standard',
    minimumMemoryGb: 16,
    installedGb: null,
    languages: ['de', 'en'],
    testedLanguages: [],
    origin: 'international',
    licence: 'modelSpecific',
    status: 'planned',
  },
];

export function hardwareTierForMemory(memoryGb: number | null): HardwareTier {
  if (memoryGb === null || memoryGb <= 8) return 'baseline';
  if (memoryGb <= 16) return 'standard';
  return 'larger';
}

/**
 * Safari's webview does not expose deviceMemory consistently. Unknown memory
 * is treated conservatively so the picker never recommends a model that is
 * predictably too large for the weakest supported machine.
 */
export function browserMemoryGb(): number | null {
  if (typeof navigator === 'undefined') return null;
  const value = (navigator as Navigator & { deviceMemory?: number }).deviceMemory;
  return typeof value === 'number' && Number.isFinite(value) && value > 0 ? value : null;
}

export function installedProviderModel(
  entry: GenerationModelEntry,
  models: ProtocolProviderModel[],
): ProtocolProviderModel | null {
  return models.find((model) => entry.providerNames.includes(model.name)) ?? null;
}

export function recommendationFor(
  models: ProtocolProviderModel[],
  memoryGb: number | null,
): { entry: GenerationModelEntry; installed: ProtocolProviderModel | null } {
  const tier = hardwareTierForMemory(memoryGb);
  const allowed = GENERATION_MODEL_CATALOG.filter(
    (entry) => entry.minimumMemoryGb <= (tier === 'baseline' ? 8 : tier === 'standard' ? 16 : 32),
  );

  // Prefer a verified installed model. The catalogue order is what was measured on
  // a German meeting at three settings each, best first:
  //
  //   Gemma 4 12B   27-31 of 35 figures, a protocol at every setting
  //   Ministral 8B  28 of 35 at its best, a protocol at one setting of three
  //   Qwen 4B       20-24 of 35, and never the table of next steps the style asks
  //   Granite 8B    22, 19 and 6 on identical input — not recommended
  //
  // Qwen still comes before the untested 3B, so an eight-gigabyte machine — where
  // neither Gemma nor Ministral 8B fits — is offered the one that has been run.
  const installed = allowed
    .map((entry) => ({ entry, installed: installedProviderModel(entry, models) }))
    .find(({ installed }) => installed !== null);
  if (installed) return installed;

  const fallback = allowed[0] ?? GENERATION_MODEL_CATALOG[0]!;
  return { entry: fallback, installed: installedProviderModel(fallback, models) };
}

/** Which of the three things the picker can say about this model is true. */
export function modelStatus(
  entry: GenerationModelEntry,
  installed: ProtocolProviderModel | null,
): ModelStatus {
  if (installed) return 'installed';
  if (entry.status === 'planned') return 'plannedCandidate';
  return 'notInstalled';
}
