import { GENERATION_MODEL_CATALOG, type GenerationModelEntry } from './modelCatalog';
import type { ModelTerms } from '../workflow/types';

/**
 * What the catalogue knows about a model this application downloads itself.
 *
 * The two halves are deliberately split. The backend owns the file — where it
 * comes from, what it weighs, its checksum, whether it is on the disk — and the
 * catalogue owns what a person reads about it: the name, who made it, the memory
 * it wants, what the evaluation runs found. Stating either half twice is how the
 * two drift, and `bundledCatalogue.test.ts` fails if a bundled model is missing
 * from the catalogue or disagrees with it about its licence.
 */
export function catalogueEntryFor(modelId: string): GenerationModelEntry | undefined {
  return GENERATION_MODEL_CATALOG.find((entry) => entry.id === modelId);
}

/**
 * Which of the bundled models to put in front of somebody.
 *
 * The largest that this machine has the memory for, because size is the thing
 * that tracks quality here and the only reason to take a smaller one is that the
 * larger will not run. Where nothing fits, the smallest — a recommendation that
 * cannot be met is still better as the closest thing to it than as silence.
 *
 * A recommendation and not a restriction: every model stays selectable, and
 * somebody who wants to find out what a larger one does on a smaller machine is
 * allowed to. The memory figures are estimates, so the honest response to a near
 * miss is to let a person decide rather than to enforce a guess.
 *
 * An unreported machine is read as 8 GB, matching the conservative baseline the
 * settings screen already shows. It arrives as `null` from the backend and as
 * `undefined` from the browser, so both mean the same thing here.
 */
export function recommendGeneration(
  models: ModelTerms[],
  memoryGb: number | null | undefined,
): string | undefined {
  const generation = models
    .filter((model) => model.kind === 'generation')
    .sort((first, second) => second.byteCount - first.byteCount);
  const smallest = generation.at(-1);
  if (!smallest) return undefined;
  const available = memoryGb ?? 8;
  const fits = generation.find((model) => {
    const wanted = catalogueEntryFor(model.modelId)?.minimumMemoryGb;
    // A model the catalogue does not describe is not recommended. The guard makes
    // that unreachable; this is what it does if the guard is ever removed.
    return wanted !== undefined && wanted <= available;
  });
  return (fits ?? smallest).modelId;
}
