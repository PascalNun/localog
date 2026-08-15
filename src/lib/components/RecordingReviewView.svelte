<script lang="ts">
  import type {
    AppRoute,
    MeetingSummary,
    RecordingEdits,
    RecordingReview,
  } from '../workflow/types';
  import Icon from './Icon.svelte';

  export let meeting: MeetingSummary;
  export let review: RecordingReview | null = null;
  export let onNavigate: (route: AppRoute) => void;
  export let onSave: (edits: RecordingEdits) => Promise<void>;
  export let onContinue: () => void;

  /** Held here while somebody works, and written when they stop moving. */
  let edits: RecordingEdits = { startMs: 0, endMs: null, removed: [] };
  let loaded = '';
  // Reload the working copy when a different recording arrives, never on every
  // redraw, or a person's dragging would be undone by their own saving.
  $: if (review && loaded !== meeting.id) {
    loaded = meeting.id;
    edits = {
      startMs: review.edits.startMs ?? 0,
      endMs: review.edits.endMs ?? null,
      removed: [...(review.edits.removed ?? [])],
    };
  }

  $: durationMs = review?.durationMs ?? 0;
  $: waveform = review?.waveform ?? [];
  $: startMs = Math.min(edits.startMs ?? 0, durationMs);
  $: endMs = Math.min(edits.endMs ?? durationMs, durationMs);
  $: removed = edits.removed ?? [];
  $: keptMs = Math.max(
    0,
    endMs - startMs - removed.reduce((total, span) => total + spanInside(span), 0),
  );

  /** How much of a removal actually falls inside the trim, which is all that counts. */
  function spanInside(span: { fromMs: number; toMs: number }) {
    const from = Math.max(Math.min(span.fromMs, span.toMs), startMs);
    const to = Math.min(Math.max(span.fromMs, span.toMs), endMs);
    return Math.max(0, to - from);
  }

  let selecting: { fromMs: number; toMs: number } | null = null;
  let dragging = false;
  let track: HTMLDivElement;

  function timeAt(event: PointerEvent) {
    const box = track.getBoundingClientRect();
    const through = Math.min(1, Math.max(0, (event.clientX - box.left) / box.width));
    return Math.round(through * durationMs);
  }

  function beginSelection(event: PointerEvent) {
    if (!durationMs) return;
    dragging = true;
    track.setPointerCapture(event.pointerId);
    const at = timeAt(event);
    selecting = { fromMs: at, toMs: at };
  }

  function extendSelection(event: PointerEvent) {
    if (!dragging || !selecting) return;
    selecting = { ...selecting, toMs: timeAt(event) };
  }

  function endSelection(event: PointerEvent) {
    if (!dragging) return;
    dragging = false;
    track.releasePointerCapture(event.pointerId);
    // A click rather than a drag selects nothing; it is how somebody dismisses a
    // selection they no longer want.
    if (selecting && Math.abs(selecting.toMs - selecting.fromMs) < 250) selecting = null;
  }

  $: selection = selecting
    ? {
        fromMs: Math.min(selecting.fromMs, selecting.toMs),
        toMs: Math.max(selecting.fromMs, selecting.toMs),
      }
    : null;

  async function commit(next: RecordingEdits) {
    edits = next;
    await onSave(next);
  }

  function removeSelection() {
    if (!selection) return;
    void commit({ ...edits, removed: [...removed, selection] });
    selecting = null;
  }

  function trimStartHere() {
    if (!selection) return;
    void commit({ ...edits, startMs: selection.fromMs });
    selecting = null;
  }

  function trimEndHere() {
    if (!selection) return;
    void commit({ ...edits, endMs: selection.toMs });
    selecting = null;
  }

  function undo(index: number) {
    void commit({ ...edits, removed: removed.filter((_, at) => at !== index) });
  }

  function clearAll() {
    void commit({ startMs: 0, endMs: null, removed: [] });
    selecting = null;
  }

  $: anyEdits = startMs > 0 || (edits.endMs ?? null) !== null || removed.length > 0;

  function clock(ms: number) {
    const total = Math.max(0, Math.round(ms / 1000));
    const hours = Math.floor(total / 3600);
    const minutes = Math.floor((total % 3600) / 60);
    const seconds = total % 60;
    const pad = (value: number) => String(value).padStart(2, '0');
    return hours > 0 ? `${hours}:${pad(minutes)}:${pad(seconds)}` : `${minutes}:${pad(seconds)}`;
  }

  const percent = (ms: number) => (durationMs ? (ms / durationMs) * 100 : 0);
</script>

<main class="workspace" id="main-content">
  <header class="workspace-header">
    <div>
      <p class="breadcrumb">{meeting.title} <span>›</span> Recording</p>
      <h1 tabindex="-1">Review recording</h1>
      <p class="page-lead">
        Cut what the meeting does not need before it is transcribed. Your recording is never changed
        — everything here can be undone.
      </p>
    </div>
  </header>

  {#if !review}
    <section class="stage-message">
      <p class="eyebrow">Recording</p>
      <h2>No working audio yet</h2>
      <p>
        This meeting has no prepared audio to review. It becomes available once the import has been
        committed.
      </p>
    </section>
  {:else}
    <section class="review-stage">
      <div class="review-timeline">
        <div class="review-times">
          <span>{clock(0)}</span>
          <span class="review-kept">{clock(keptMs)} of {clock(durationMs)} kept</span>
          <span>{clock(durationMs)}</span>
        </div>

        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="review-track"
          bind:this={track}
          onpointerdown={beginSelection}
          onpointermove={extendSelection}
          onpointerup={endSelection}
          onpointercancel={endSelection}
        >
          <div class="review-waveform" aria-hidden="true">
            {#each waveform as peak, index (index)}
              <span style="height: {Math.max(2, peak * 100)}%"></span>
            {/each}
          </div>

          <!-- What the edits leave out, drawn over the shape rather than removed
               from it, so a person can see what they cut and put it back. -->
          {#if startMs > 0}
            <div class="review-dropped" style="left: 0; width: {percent(startMs)}%"></div>
          {/if}
          {#if endMs < durationMs}
            <div class="review-dropped" style="left: {percent(endMs)}%; right: 0"></div>
          {/if}
          {#each removed as span, index (index)}
            <div
              class="review-dropped"
              style="left: {percent(Math.min(span.fromMs, span.toMs))}%; width: {percent(
                Math.abs(span.toMs - span.fromMs),
              )}%"
            ></div>
          {/each}
          {#if selection}
            <div
              class="review-selection"
              style="left: {percent(selection.fromMs)}%; width: {percent(
                selection.toMs - selection.fromMs,
              )}%"
            ></div>
          {/if}
        </div>

        <p class="review-hint">
          {#if selection}
            Selected {clock(selection.fromMs)} to {clock(selection.toMs)}.
          {:else}
            Drag across the recording to select a stretch.
          {/if}
        </p>

        <div class="review-actions">
          <button class="text-action" disabled={!selection} onclick={trimStartHere}
            >Start here</button
          >
          <button class="text-action" disabled={!selection} onclick={removeSelection}
            >Remove selection</button
          >
          <button class="text-action" disabled={!selection} onclick={trimEndHere}>End here</button>
        </div>
      </div>

      <aside class="review-inspector">
        <div class="inspector-section">
          <p class="eyebrow">Edits</p>
          {#if !anyEdits}
            <p class="review-empty">Nothing removed. The whole recording will be transcribed.</p>
          {:else}
            <ul class="review-edits">
              {#if startMs > 0}
                <li>
                  <span>Starts at {clock(startMs)}</span>
                  <button
                    class="text-action"
                    onclick={() => commit({ ...edits, startMs: 0 })}
                    aria-label="Undo the start trim">Undo</button
                  >
                </li>
              {/if}
              {#if (edits.endMs ?? null) !== null}
                <li>
                  <span>Ends at {clock(endMs)}</span>
                  <button
                    class="text-action"
                    onclick={() => commit({ ...edits, endMs: null })}
                    aria-label="Undo the end trim">Undo</button
                  >
                </li>
              {/if}
              {#each removed as span, index (index)}
                <li>
                  <span
                    >Removed {clock(Math.min(span.fromMs, span.toMs))} to {clock(
                      Math.max(span.fromMs, span.toMs),
                    )}</span
                  >
                  <button
                    class="text-action"
                    onclick={() => undo(index)}
                    aria-label="Put this stretch back">Undo</button
                  >
                </li>
              {/each}
            </ul>
            <button class="text-action" onclick={clearAll}>Put everything back</button>
          {/if}
          <p class="review-note">
            <Icon name="info" size={14} />
            <span>The recording itself is untouched. These are instructions for what to use.</span>
          </p>
        </div>

        <div class="inspector-section">
          <p class="eyebrow">Next</p>
          <button class="primary-action" onclick={onContinue}>Continue to transcription</button>
          <button
            class="text-action"
            onclick={() => onNavigate({ name: 'meeting', meetingId: meeting.id })}
            >Back to the meeting</button
          >
        </div>
      </aside>
    </section>
  {/if}
</main>
