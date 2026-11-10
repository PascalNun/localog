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
  | 'qwen3.5-4b'
  | 'qwen3.5-9b'
  | 'qwen3.5-27b'
  | 'qwen3.5-35b-a3b'
  | 'gemma4-12b'
  | 'gemma4-26b'
  | 'gemma4-31b'
  | 'ministral-3-14b';

/** A language a model claims, or `more` for "and many others". */
export type ModelLanguage = 'de' | 'en' | 'ja' | 'more';

/** Where a model comes from, as a fact rather than as the words for it. */
export type ModelOrigin = 'international' | 'european';

/** Which licence a model is offered under. */
export type ModelLicence = 'apache2' | 'gemma' | 'modelSpecific';

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
 * The catalogue stays curated: every downloadable file has a verified licence,
 * runtime and checksum. A new size may enter before it has been measured, but it
 * carries an empty `testedLanguages` and is never the default until it has been
 * run on a reference meeting here.
 *
 * `status: 'baseline'` is the product judgement, not a measurement: it says this
 * is the model LocaLog falls back to when the machine fits it. Measurement alone
 * is not enough — a model can be measured and still be unusable, as Ministral 8B
 * was at one seed in three. The recommendable models are marked explicitly, so
 * the recommendation no longer depends on catalogue order.
 *
 * Each family appears once per memory class. A smaller sibling that needs the same
 * class as a larger one is not an alternative, it is a worse version of it; the
 * ones that were dominated or measured unusable are not listed.
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
    licence: 'apache2',
    status: 'baseline',
  },
  {
    id: 'gemma4-26b',
    name: 'Gemma 4 26B A4B',
    family: 'Gemma 4',
    providerNames: ['gemma4:26b'],
    tier: 'larger',
    minimumMemoryGb: 24,
    installedGb: null,
    languages: ['de', 'en', 'more'],
    testedLanguages: [],
    origin: 'international',
    licence: 'apache2',
    status: 'candidate',
  },
  {
    id: 'gemma4-31b',
    name: 'Gemma 4 31B',
    family: 'Gemma 4',
    providerNames: ['gemma4:31b'],
    tier: 'larger',
    minimumMemoryGb: 32,
    installedGb: null,
    languages: ['de', 'en', 'more'],
    testedLanguages: [],
    origin: 'international',
    licence: 'apache2',
    status: 'candidate',
  },
  {
    id: 'qwen3.5-35b-a3b',
    name: 'Qwen3.5 35B A3B',
    family: 'Qwen3.5',
    providerNames: ['qwen3.5:35b-a3b'],
    tier: 'larger',
    minimumMemoryGb: 32,
    installedGb: null,
    languages: ['de', 'en', 'more'],
    testedLanguages: [],
    origin: 'international',
    licence: 'apache2',
    status: 'candidate',
  },
  {
    id: 'qwen3.5-27b',
    name: 'Qwen3.5 27B',
    family: 'Qwen3.5',
    providerNames: ['qwen3.5:27b'],
    tier: 'larger',
    minimumMemoryGb: 32,
    installedGb: null,
    languages: ['de', 'en', 'more'],
    testedLanguages: [],
    origin: 'international',
    licence: 'apache2',
    status: 'candidate',
  },
  {
    id: 'qwen3.5-9b',
    name: 'Qwen3.5 9B',
    family: 'Qwen3.5',
    providerNames: ['qwen3.5:9b'],
    tier: 'standard',
    minimumMemoryGb: 16,
    installedGb: null,
    languages: ['de', 'en', 'more'],
    testedLanguages: [],
    origin: 'international',
    licence: 'apache2',
    status: 'candidate',
  },
  {
    id: 'ministral-3-14b',
    name: 'Ministral 3 14B',
    family: 'Ministral 3',
    providerNames: ['ministral-3:14b', 'ministral-3:14b-instruct-2512-q4_K_M'],
    tier: 'standard',
    minimumMemoryGb: 16,
    installedGb: null,
    languages: ['de', 'en', 'ja', 'more'],
    testedLanguages: [],
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
    status: 'baseline',
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
