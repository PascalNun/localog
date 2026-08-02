import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    environment: 'node',
    include: ['spikes/markdown-editor/tests/**/*.test.ts'],
    restoreMocks: true,
  },
});
