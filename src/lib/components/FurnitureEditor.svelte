<script lang="ts">
  import { FURNITURE_FIELDS, fieldsFromLine, lineHtml } from '../protocol/furniture';
  import type { FurnitureField, FurnitureRow, PageFurniture } from '../workflow/types';
  import { t } from '../i18n';

  export let furniture: PageFurniture;
  export let projectName: string;
  export let onChange: (furniture: PageFurniture) => Promise<void>;

  /// Every edit is the same shape: take one slot, give back a new one.
  ///
  /// Adding, removing and retyping were three functions that each rebuilt the whole
  /// structure by hand, and each had its own chance to write `header` where it meant
  /// `footer`. There is one now, and the callers differ only in what they do to the
  /// list they are handed.
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

  /// A slot is a line, so it is edited as one.
  ///
  /// It used to be a row of chips with an `Add…` beside them, which is a list of
  /// atoms standing in for a sentence: "Projekt: Neubau Halle 4" could not be
  /// written, and the spaces that hold a value against the words beside it were
  /// invisible inside a chip. Every tool a professional office already has — Word,
  /// Google Docs, and the two Markdown-to-PDF paths — lets somebody write the line
  /// and put the value into the middle of it.
  ///
  /// The content is set on the node rather than rendered by Svelte, because Svelte
  /// re-rendering a contenteditable on every keystroke puts the caret back at the
  /// start. It is refreshed only when the value changes from somewhere else, and
  /// never while the caret is in it.
  function line(node: HTMLElement, fields: FurnitureField[]) {
    node.innerHTML = lineHtml(fields);
    return {
      update(next: FurnitureField[]) {
        if (document.activeElement === node) return;
        const wanted = lineHtml(next);
        if (node.innerHTML !== wanted) node.innerHTML = wanted;
      },
    };
  }

  /// The line read back: a value is a node that says which one it is, and
  /// everything else is the characters somebody typed.
  function readLine(node: HTMLElement): FurnitureField[] {
    return fieldsFromLine(
      Array.from(node.childNodes).map((child) => {
        const kind = child instanceof HTMLElement ? child.dataset.kind : undefined;
        return kind ? { kind } : { text: child.textContent ?? '' };
      }),
    );
  }

  function insert(node: HTMLElement, kind: string) {
    node.focus();
    const choice = FURNITURE_FIELDS.find((each) => each.kind === kind);
    if (!choice) return;
    // At the caret, so a value lands where somebody is writing rather than at the
    // end of the line.
    document.execCommand(
      'insertHTML',
      false,
      `<span class="furniture-value" contenteditable="false" data-kind="${kind}">${choice.label}</span>`,
    );
  }

  // Reactive, not constant: a `const` reads the dictionary once, at the moment
  // this component is first created, and would still say Header after somebody
  // switched to German. The ids are what the model is keyed on and never move.
  $: BANDS = [
    { id: 'header', label: $t.furniture.header },
    { id: 'footer', label: $t.furniture.footer },
  ] as const;

  $: SLOTS = [
    { id: 'left', label: $t.furniture.left },
    { id: 'centre', label: $t.furniture.centre },
    { id: 'right', label: $t.furniture.right },
  ] as const;
</script>

<div class="furniture-editor">
  {#each BANDS as band (band.id)}
    <div class="furniture-band">
      <p class="eyebrow">{band.label}</p>
      {#each SLOTS as slot (slot.id)}
        <div class="furniture-slot">
          <span class="furniture-slot-name">{slot.label}</span>
          <div class="furniture-line-row">
            <div
              class="furniture-line"
              contenteditable="true"
              role="textbox"
              tabindex="0"
              aria-label={`${band.label}, ${slot.label}`}
              use:line={furniture[band.id][slot.id]}
              oninput={(event) => {
                const node = event.currentTarget;
                void editSlot(band.id, slot.id, () => readLine(node));
              }}
              onkeydown={(event) => {
                // One line, so a return would only make a second one nobody can see.
                if (event.key === 'Enter') event.preventDefault();
              }}
            ></div>
            <select
              class="furniture-insert"
              value=""
              aria-label={`Insert a value into ${band.label} ${slot.label}`}
              onchange={(event) => {
                const kind = event.currentTarget.value;
                event.currentTarget.value = '';
                const node = event.currentTarget.previousElementSibling;
                if (!kind || !(node instanceof HTMLElement)) return;
                insert(node, kind);
                void editSlot(band.id, slot.id, () => readLine(node));
              }}
            >
              <option value="">{$t.furniture.insert}</option>
              {#each FURNITURE_FIELDS.filter((choice) => choice.kind !== 'text') as choice (choice.kind)}
                <option value={choice.kind}>{choice.label}</option>
              {/each}
            </select>
          </div>
        </div>
      {/each}
    </div>
  {/each}

  <p class="appearance-note">
    {$t.furniture.lineHint}
  </p>
  <p class="appearance-note">
    {$t.furniture.appliesTo(projectName)}
  </p>
</div>

<style>
  /* The one thing here that is not shared with the rest of the inspector. */
  .furniture-band :global(.eyebrow) {
    margin-bottom: 2px;
  }
</style>
