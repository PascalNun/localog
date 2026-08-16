import type { ProtocolProviderModel } from '../workflow/types';

/**
 * A deliberately small catalogue for the normal model picker.
 *
 * The catalogue is a product boundary, not a marketplace. Entries can be
 * shown before they are downloadable so the user can see the planned path,
 * while selection still requires a verified model already exposed by the
 * active provider.
 */
export interface GenerationModelEntry {
  id: string;
  name: string;
  family: string;
  providerNames: string[];
  tier: 'baseline' | 'standard' | 'larger';
  minimumMemoryGb: number;
  sizeLabel: string;
  languages: string[];
  testedLanguages: string[];
  originLabel: string;
  licenseLabel: string;
  description: string;
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
    sizeLabel: 'about 8 GB installed',
    languages: ['German', 'English', 'many more'],
    testedLanguages: ['German'],
    originLabel: 'International open model',
    licenseLabel: 'Gemma terms of use',
    description:
      'The most accurate and the steadiest of the measured models: it kept 27 to 31 of a meeting’s 35 figures across three runs, where the next best kept as few as 6. Slower — about fourteen minutes for an eighty-minute meeting.',
    status: 'baseline',
  },
  {
    id: 'qwen3.5-4b',
    name: 'Qwen3.5 4B',
    family: 'Qwen3.5',
    providerNames: ['qwen3.5:4b'],
    tier: 'baseline',
    minimumMemoryGb: 8,
    sizeLabel: 'about 3.4 GB installed',
    languages: ['German', 'English', 'many more'],
    testedLanguages: ['German'],
    originLabel: 'International open model',
    licenseLabel: 'Apache 2.0',
    description:
      'The fastest measured model, at about five minutes for an eighty-minute meeting, and the choice where memory is short. It never produced the table of next steps the formal style asks for.',
    status: 'candidate',
  },
  {
    id: 'ministral-3b',
    name: 'Ministral 3 3B',
    family: 'Ministral 3',
    providerNames: ['ministral-3:3b', 'ministral-3:3b-instruct-2512-q4_K_M'],
    tier: 'baseline',
    minimumMemoryGb: 8,
    sizeLabel: 'small edge model',
    languages: ['German', 'English', 'Japanese', 'many more'],
    testedLanguages: [],
    originLabel: 'European model',
    licenseLabel: 'Apache 2.0',
    description: 'The first European candidate for the weakest supported Mac.',
    status: 'candidate',
  },
  {
    id: 'ministral-8b',
    name: 'Ministral 3 8B',
    family: 'Ministral 3',
    providerNames: ['ministral-3:8b', 'ministral-3:8b-instruct-2512-q4_K_M'],
    tier: 'standard',
    minimumMemoryGb: 16,
    sizeLabel: 'larger local model',
    languages: ['German', 'English', 'Japanese', 'many more'],
    testedLanguages: ['German'],
    originLabel: 'European model',
    licenseLabel: 'Apache 2.0',
    description:
      'Measured on a German meeting at three settings and wrote a usable protocol at one of them: the others produced a two-line stub and a JSON document where markdown was asked for. Kept as the European candidate, not yet an alternative to the baseline.',
    status: 'candidate',
  },
  {
    id: 'granite4.1-8b',
    name: 'Granite 4.1 8B',
    family: 'Granite 4.1',
    providerNames: ['granite4.1:8b'],
    tier: 'standard',
    minimumMemoryGb: 16,
    sizeLabel: 'about 5.3 GB installed',
    languages: ['German', 'English'],
    testedLanguages: [],
    originLabel: 'International open model',
    licenseLabel: 'Apache 2.0',
    description: 'An installed 8B comparison model for the 16 GB test machine.',
    status: 'candidate',
  },
  {
    id: 'llama-8b',
    name: 'Llama 8B',
    family: 'Llama',
    providerNames: [],
    tier: 'standard',
    minimumMemoryGb: 16,
    sizeLabel: 'larger local model',
    languages: ['German', 'English'],
    testedLanguages: [],
    originLabel: 'International open model',
    licenseLabel: 'Model-specific',
    description: 'A later comparison slot for a verified Llama release.',
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

  // Prefer a verified installed model. The catalogue order is intentional and is
  // the order the models were measured in: Gemma 4 12B kept 27 to 31 of a
  // meeting's 35 stated figures across three seeds where Granite kept as few as
  // 6 on identical input, and Qwen never produced the table of next steps the
  // formal style asks for. Qwen stays ahead of nothing else on the 8 GB tier,
  // where Gemma does not fit.
  const installed = allowed
    .map((entry) => ({ entry, installed: installedProviderModel(entry, models) }))
    .find(({ installed }) => installed !== null);
  if (installed) return installed;

  const fallback = allowed[0] ?? GENERATION_MODEL_CATALOG[0]!;
  return { entry: fallback, installed: installedProviderModel(fallback, models) };
}

export function modelStatusLabel(
  entry: GenerationModelEntry,
  installed: ProtocolProviderModel | null,
): string {
  if (installed) return 'Installed';
  if (entry.status === 'planned') return 'Planned candidate';
  return 'Not installed';
}
