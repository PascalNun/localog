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
  const recommended =
    options.find(
      (option) => option.fits && option.model.installed && option.entry.id !== 'qwen3.5-0.8b',
    ) ?? options.find((option) => option.fits && option.entry.id !== 'qwen3.5-0.8b');
  return options
    .map((option) => ({ ...option, recommended: memoryGb !== null && option === recommended }))
    .sort(
      (a, b) => Number(b.recommended) - Number(a.recommended) || Number(b.fits) - Number(a.fits),
    );
}
