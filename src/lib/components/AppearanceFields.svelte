<script lang="ts">
  import { APPEARANCE_CHOICES } from '../protocol/appearance';
  import type { DocumentAppearance } from '../workflow/types';
  import { t } from '../i18n';

  export let appearance: DocumentAppearance;
  export let projectName: string;
  export let onChange: (appearance: DocumentAppearance) => Promise<void>;

  /// One handler for five settings.
  ///
  /// They were five copies of the same three lines differing only in the key and the
  /// cast, which is the shape a machine writes and a person then has to read five
  /// times to check they are the same.
  async function change<K extends keyof DocumentAppearance>(key: K, value: DocumentAppearance[K]) {
    await onChange({ ...appearance, [key]: value });
  }

  /// Each row, as the field it sets and the choices it offers. `bodySize` is a number
  /// and the rest are strings, which is the only reason the value is read back
  /// through the option list rather than used directly.
  const ROWS = [
    { key: 'font', label: 'Font', choices: APPEARANCE_CHOICES.font },
    { key: 'bodySize', label: $t.appearance.bodySize, choices: APPEARANCE_CHOICES.bodySize },
    {
      key: 'headingScale',
      label: $t.appearance.headingScale,
      choices: APPEARANCE_CHOICES.headingScale,
    },
    {
      key: 'lineSpacing',
      label: $t.appearance.lineSpacing,
      choices: APPEARANCE_CHOICES.lineSpacing,
    },
    { key: 'pageWidth', label: $t.appearance.pageWidth, choices: APPEARANCE_CHOICES.pageWidth },
  ] as const;
</script>

<div class="appearance-fields">
  {#each ROWS as row (row.key)}
    <label>
      <span>{row.label}</span>
      <select
        value={String(appearance[row.key])}
        onchange={(event) => {
          const chosen = row.choices.find(
            (choice) => String(choice.value) === event.currentTarget.value,
          );
          if (chosen) void change(row.key, chosen.value as DocumentAppearance[typeof row.key]);
        }}
      >
        {#each row.choices as choice (choice.value)}
          <option value={String(choice.value)}>{choice.label}</option>
        {/each}
      </select>
    </label>
  {/each}
  <p class="appearance-note">
    Applies to every protocol in {projectName}, so a firm's documents look alike. It changes how the
    protocol is set, never what it says — that is the style above.
  </p>
</div>
