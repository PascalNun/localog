import { describe, expect, it } from 'vitest';
import {
  GENERATION_MODEL_CATALOG,
  hardwareTierForMemory,
  modelStatusLabel,
  recommendationFor,
} from './modelCatalog';

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
      modelStatusLabel(granite!, {
        name: 'granite4.1:8b',
        size: 5_300_000_000,
        digest: 'sha256:granite',
      }),
    ).toBe('Installed');
    expect(modelStatusLabel(ministral!, null)).toBe('Planned candidate');
  });
});
