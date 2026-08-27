<script lang="ts">
  import Icon from './Icon.svelte';
  import { t } from '../i18n';
  import type { Section } from '../protocol/sections';
  import type { SetAsideSection } from '../workflow/types';

  export let sections: Section[];
  export let setAside: SetAsideSection[];
  export let onMove: (from: number, to: number) => Promise<void>;
  export let onSetAside: (index: number) => Promise<void>;
  export let onBringBack: (index: number) => Promise<void>;
  export let onAdd: () => Promise<void>;
  export let onGoTo: (index: number) => void;

  let dragging: number | null = null;

  async function drop(onto: number) {
    const from = dragging;
    dragging = null;
    if (from === null || from === onto) return;
    await onMove(from, onto);
  }

  /// Reordering without a mouse.
  ///
  /// Dragging was the only way to move a section, which leaves anybody working from
  /// the keyboard unable to do it at all. Focus follows the section rather than the
  /// row it left, so a run of presses moves it several places.
  async function moveByKey(event: KeyboardEvent, index: number) {
    const to = event.key === 'ArrowUp' ? index - 1 : event.key === 'ArrowDown' ? index + 1 : null;
    if (to === null || to < 0 || to >= sections.length) return;
    event.preventDefault();
    await onMove(index, to);
    queueMicrotask(() => {
      document.querySelectorAll<HTMLElement>('.section-grip')[to]?.focus();
    });
  }
</script>

{#if sections.length === 0}
  <p class="section-none">{$t.sections.noHeadings}</p>
{:else}
  <ul class="section-list">
    {#each sections as section, index (section.from)}
      <li
        class:dragging={dragging === index}
        draggable="true"
        ondragstart={() => (dragging = index)}
        ondragover={(event) => event.preventDefault()}
        ondrop={() => void drop(index)}
        ondragend={() => (dragging = null)}
      >
        <button
          class="section-grip"
          aria-label={$t.sections.moveSection(section.title)}
          title={$t.sections.dragHint}
          onkeydown={(event) => void moveByKey(event, index)}>⠿</button
        >
        <button class="section-name" onclick={() => onGoTo(index)}>{section.title}</button>
        <button
          class="icon-button compact"
          title={$t.sections.setThisAside}
          aria-label={$t.sections.setAsideNamed(section.title)}
          onclick={() => void onSetAside(index)}><Icon name="close" size={14} /></button
        >
      </li>
    {/each}
  </ul>
{/if}

{#if setAside.length > 0}
  <p class="section-stash-label">{$t.sections.setAside}</p>
  <ul class="section-list stashed">
    {#each setAside as held, index (held.title + index)}
      <li>
        <span class="section-grip-spacer" aria-hidden="true"></span>
        <span class="section-name">{held.title}</span>
        <button
          class="icon-button compact"
          title={$t.sections.putThisBack}
          aria-label={$t.sections.putBackNamed(held.title)}
          onclick={() => void onBringBack(index)}><Icon name="plus" size={14} /></button
        >
      </li>
    {/each}
  </ul>
{/if}

<button class="inspector-control" onclick={() => void onAdd()}>
  <Icon name="plus" size={16} />
  <span>{$t.sections.addSection}</span>
  <span></span>
</button>
<p class="section-note">
  {$t.sections.setAsideNote}
</p>
