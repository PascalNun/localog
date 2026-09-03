import type { ModelTerms } from '../workflow/types';

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
  const fits = generation.find((model) => (model.wantsMemoryGb ?? 0) <= available);
  return (fits ?? smallest).modelId;
}
