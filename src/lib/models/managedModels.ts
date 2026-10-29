import type { ModelTerms } from '../workflow/types';
import { GENERATION_MODEL_CATALOG } from './modelCatalog';

/** Keep every managed option visible; an oversized model must explain why it is unavailable. */
export function managedModelOptions(terms: ModelTerms[], memoryGb: number | null) {
  const options = terms
    .filter((model) => model.kind === 'generation')
    .flatMap((model) => {
      const entry = GENERATION_MODEL_CATALOG.find((entry) => entry.id === model.modelId);
      if (!entry) return [];
      const minimumMemoryGb = model.minimumMemoryGb ?? entry.minimumMemoryGb;
      return [
        { model, entry, minimumMemoryGb, fits: memoryGb === null || memoryGb >= minimumMemoryGb },
      ];
    });
  // Recommend what was measured for this machine, not merely what is already
  // installed. The catalogue order is the quality order, and the 3B is an
  // unevaluated European candidate: preferring it because it happens to be on
  // disk steered an 8 GB machine to the slower, untested model instead of the
  // measured 4B. Selection is still the user's; the badge is guidance.
  //
  // The 0.8B is never recommended. On a 4 GB machine it is the only model that
  // fits, but it is a smoke-test model and the badge would say otherwise; the
  // card remains downloadable and its description says what it is.
  const recommended = options.find((option) => option.fits && option.entry.id !== 'qwen3.5-0.8b');
  return options
    .map((option) => ({ ...option, recommended: memoryGb !== null && option === recommended }))
    .sort(
      (a, b) => Number(b.recommended) - Number(a.recommended) || Number(b.fits) - Number(a.fits),
    );
}
