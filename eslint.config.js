import js from '@eslint/js';
import svelte from 'eslint-plugin-svelte';
import globals from 'globals';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  { ignores: ['dist/', 'node_modules/', 'src-tauri/target/'] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  ...svelte.configs.recommended,
  {
    languageOptions: { globals: { ...globals.browser, ...globals.node } },
    rules: {
      // A leading underscore means "bound on purpose, never read".
      //
      // The project already writes it that way in the two places it comes up:
      // `const { [id]: _gone, ...rest } = map` is how a key is dropped without
      // mutating, and `{#each list as _, at (at)}` wants the index and not the
      // item. Both bindings must exist for the syntax to work and neither can be
      // used. Stating the convention once is better than three suppressions that
      // each have to explain it again.
      '@typescript-eslint/no-unused-vars': [
        'error',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_', caughtErrorsIgnorePattern: '^_' },
      ],
    },
  },
  {
    files: ['**/*.svelte', '**/*.svelte.ts', '**/*.svelte.js'],
    languageOptions: { parserOptions: { parser: tseslint.parser } },
    rules: {
      // Off because it crashes the whole run, and because it is wrong here.
      //
      // The rule calls `source.isSpaceBetweenTokens` while building its
      // suggestion, which ESLint 10 removed, so reporting once takes down every
      // file rather than that one: `npm run lint` has not completed since the
      // upgrade. Upgrading the plugin does not help — 3.23.0 makes the same call.
      //
      // Silencing a rule to stop a crash would be the wrong trade if the rule
      // were right. It is not. What it flags is `$: bandAt = (at, band) => …` in
      // ProtocolView, where the reactive assignment is the point: it is what
      // makes the template recompute when `facts` or `pageCount` change. The
      // plain function the rule wants would be defined once and never re-run.
      //
      // Worth revisiting when the plugin supports ESLint 10, in case it grows a
      // way to tell a deliberate reactive function from an accidental one.
      'svelte/no-reactive-functions': 'off',
    },
  },
);
