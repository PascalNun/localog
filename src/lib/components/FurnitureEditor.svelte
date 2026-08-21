<script lang="ts">
  import { FURNITURE_FIELDS, fieldLabel, needsPageNumbers } from '../protocol/furniture';
  import type { FurnitureField, FurnitureRow, PageFurniture } from '../workflow/types';

  export let furniture: PageFurniture;
  export let projectName: string;
  export let onChange: (furniture: PageFurniture) => Promise<void>;

  /// Every edit is the same shape: take one slot, give back a new one.
  ///
  /// Adding, removing and retyping were three functions that each rebuilt the whole
  /// structure by hand, and each had its own chance to write `header` where it meant
  /// `footer`. There is one now, and the three callers differ only in what they do
  /// to the list they are handed.
  async function editSlot(
    band: 'header' | 'footer',
    slot: keyof FurnitureRow,
    change: (fields: FurnitureField[]) => FurnitureField[],
  ) {
    await onChange({
      ...furniture,
      [band]: { ...furniture[band], [slot]: change(furniture[band][slot]) },
    });
  }

  const BANDS = [
    { id: 'header', label: 'Header' },
    { id: 'footer', label: 'Footer' },
  ] as const;

  const SLOTS = [
    { id: 'left', label: 'Left' },
    { id: 'centre', label: 'Centre' },
    { id: 'right', label: 'Right' },
  ] as const;
</script>

<div class="furniture-editor">
  {#each BANDS as band (band.id)}
    <div class="furniture-band">
      <p class="eyebrow">{band.label}</p>
      {#each SLOTS as slot (slot.id)}
        <div class="furniture-slot">
          <span class="furniture-slot-name">{slot.label}</span>
          <div class="furniture-chips">
            {#each furniture[band.id][slot.id] as field, at (at)}
              <span class="furniture-chip">
                {#if field.kind === 'text'}
                  <input
                    value={field.value}
                    placeholder="Your text"
                    onchange={(event) => {
                      const typed = event.currentTarget.value;
                      void editSlot(band.id, slot.id, (fields) =>
                        fields.map((each, index) =>
                          index === at && each.kind === 'text'
                            ? { kind: 'text', value: typed }
                            : each,
                        ),
                      );
                    }}
                  />
                {:else}
                  {fieldLabel(field)}
                {/if}
                <button
                  class="furniture-remove"
                  aria-label={`Remove ${fieldLabel(field)}`}
                  onclick={() =>
                    void editSlot(band.id, slot.id, (fields) =>
                      fields.filter((_, index) => index !== at),
                    )}>×</button
                >
              </span>
            {/each}
            <select
              value=""
              aria-label={`Add to ${band.label} ${slot.label}`}
              onchange={(event) => {
                const kind = event.currentTarget.value;
                event.currentTarget.value = '';
                if (!kind) return;
                const field = (
                  kind === 'text' ? { kind: 'text', value: '' } : { kind }
                ) as FurnitureField;
                void editSlot(band.id, slot.id, (fields) => [...fields, field]);
              }}
            >
              <option value="">Add…</option>
              {#each FURNITURE_FIELDS as choice (choice.kind)}
                <option value={choice.kind}>{choice.label}</option>
              {/each}
            </select>
          </div>
        </div>
      {/each}
    </div>
  {/each}

  {#if needsPageNumbers(furniture)}
    <p class="appearance-note">
      Word counts the pages itself, so the number is right there. The PDF is printed by the browser,
      which will not say what page it is on — the page number is left out of that one rather than
      printed wrongly on every sheet.
    </p>
  {/if}
  <p class="appearance-note">
    Applies to every protocol in {projectName}. It repeats on the printed page and is not part of
    the document you are editing.
  </p>
</div>

<style>
  /* The one thing here that is not shared with the rest of the inspector. */
  .furniture-band :global(.eyebrow) {
    margin-bottom: 2px;
  }
</style>
