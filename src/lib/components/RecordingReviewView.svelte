<script lang="ts">
  import type {
    AppRoute,
    MeetingSummary,
    RecordingEdits,
    RecordingReview,
  } from '../workflow/types';
  import Icon from './Icon.svelte';
  import { clockFromMillis } from '../time';
  import { t } from '../i18n';

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
    // The linter reads this as writing a value nobody goes on to use. It is read by
    // this block's own condition on the next run, which is the whole point of it.
    // eslint-disable-next-line no-useless-assignment
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
    caretMs = at;
    selecting = { fromMs: at, toMs: at };
  }

  function extendSelection(event: PointerEvent) {
    if (!dragging || !selecting) return;
    caretMs = timeAt(event);
    selecting = { ...selecting, toMs: caretMs };
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

  /// Selecting by keyboard, because dragging is the only way to do this otherwise
  /// and a keyboard cannot drag. A caret moves along the recording with the arrow
  /// keys; holding shift takes the selection with it, which is how selection works
  /// in every text field a person has ever used.
  ///
  /// The step is a thirtieth of the recording so that crossing an hour-long meeting
  /// takes a few seconds, and a finer one is available on the same keys.
  let caretMs = 0;
  function stepFor(event: KeyboardEvent) {
    if (event.altKey) return Math.max(100, Math.round(durationMs / 2000));
    if (event.shiftKey || event.metaKey) return Math.max(1000, Math.round(durationMs / 300));
    return Math.max(1000, Math.round(durationMs / 30));
  }

  function onTrackKey(event: KeyboardEvent) {
    if (!durationMs) return;
    const extend = event.shiftKey;
    // Assigned by every branch that falls through; the rest return.
    let next: number;
    switch (event.key) {
      case 'ArrowRight':
        next = Math.min(durationMs, caretMs + stepFor(event));
        break;
      case 'ArrowLeft':
        next = Math.max(0, caretMs - stepFor(event));
        break;
      case 'Home':
        next = 0;
        break;
      case 'End':
        next = durationMs;
        break;
      case 'Escape':
        selecting = null;
        return;
      default:
        return;
    }
    event.preventDefault();
    if (extend) {
      // Anchor where the selection began, so shift-left after shift-right shrinks
      // it rather than starting a new one going the other way.
      const anchor = selecting ? selecting.fromMs : caretMs;
      selecting = { fromMs: anchor, toMs: next };
    } else {
      selecting = null;
    }
    caretMs = next;
  }

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

  const percent = (ms: number) => (durationMs ? (ms / durationMs) * 100 : 0);
</script>

<main class="workspace" id="main-content">
  <header class="workspace-header">
    <div>
      <p class="breadcrumb">{meeting.title} <span>›</span> Recording</p>
      <h1 tabindex="-1">{$t.recordingReview.heading}</h1>
      <p class="page-lead">
        Cut what the meeting does not need before it is transcribed. Your recording is never changed
        — everything here can be undone.
      </p>
    </div>
  </header>

  {#if !review}
    <section class="stage-message">
      <p class="eyebrow">{$t.recordingReview.eyebrow}</p>
      <h2>{$t.recordingReview.noAudio}</h2>
      <p>
        This meeting has no prepared audio to review. It becomes available once the import has been
        committed.
      </p>
    </section>
  {:else}
    <section class="review-stage">
      <div class="review-timeline">
        <div class="review-times">
          <span>{clockFromMillis(0)}</span>
          <span class="review-kept"
            >{clockFromMillis(keptMs)} of {clockFromMillis(durationMs)} kept</span
          >
          <span>{clockFromMillis(durationMs)}</span>
        </div>

        <div
          class="review-track"
          bind:this={track}
          role="slider"
          tabindex="0"
          aria-label={$t.recordingReview.waveformLabel}
          aria-valuemin={0}
          aria-valuemax={Math.round(durationMs / 1000)}
          aria-valuenow={Math.round(caretMs / 1000)}
          aria-valuetext={selection
            ? `Selected ${clockFromMillis(selection.fromMs)} to ${clockFromMillis(selection.toMs)}`
            : clockFromMillis(caretMs)}
          onpointerdown={beginSelection}
          onpointermove={extendSelection}
          onpointerup={endSelection}
          onpointercancel={endSelection}
          onkeydown={onTrackKey}
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
          <div class="review-caret" style="left: {percent(caretMs)}%"></div>
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
            Selected {clockFromMillis(selection.fromMs)} to {clockFromMillis(selection.toMs)}.
          {:else}
            Drag across the recording to select a stretch, or use the arrow keys and hold shift.
          {/if}
        </p>

        <div class="review-actions">
          <button class="text-action" disabled={!selection} onclick={trimStartHere}
            >{$t.recordingReview.startHere}</button
          >
          <button class="text-action" disabled={!selection} onclick={removeSelection}
            >{$t.recordingReview.removeSelection}</button
          >
          <button class="text-action" disabled={!selection} onclick={trimEndHere}
            >{$t.recordingReview.endHere}</button
          >
        </div>
      </div>

      <aside class="review-inspector">
        <div class="inspector-section">
          <p class="eyebrow">{$t.recordingReview.edits}</p>
          {#if !anyEdits}
            <p class="review-empty">{$t.recordingReview.nothingRemoved}</p>
          {:else}
            <ul class="review-edits">
              {#if startMs > 0}
                <li>
                  <span>Starts at {clockFromMillis(startMs)}</span>
                  <button
                    class="text-action"
                    onclick={() => commit({ ...edits, startMs: 0 })}
                    aria-label={$t.recordingReview.undoStartTrim}>{$t.recordingReview.undo}</button
                  >
                </li>
              {/if}
              {#if (edits.endMs ?? null) !== null}
                <li>
                  <span>Ends at {clockFromMillis(endMs)}</span>
                  <button
                    class="text-action"
                    onclick={() => commit({ ...edits, endMs: null })}
                    aria-label={$t.recordingReview.undoEndTrim}>{$t.recordingReview.undo}</button
                  >
                </li>
              {/if}
              {#each removed as span, index (index)}
                <li>
                  <span
                    >Removed {clockFromMillis(Math.min(span.fromMs, span.toMs))} to {clockFromMillis(
                      Math.max(span.fromMs, span.toMs),
                    )}</span
                  >
                  <button
                    class="text-action"
                    onclick={() => undo(index)}
                    aria-label={$t.recordingReview.putStretchBack}>{$t.recordingReview.undo}</button
                  >
                </li>
              {/each}
            </ul>
            <button class="text-action" onclick={clearAll}
              >{$t.recordingReview.putEverythingBack}</button
            >
          {/if}
          <p class="review-note">
            <Icon name="info" size={14} />
            <span>{$t.recordingReview.untouchedNote}</span>
          </p>
        </div>

        <div class="inspector-section">
          <p class="eyebrow">{$t.recordingReview.next}</p>
          <button class="primary-action" onclick={onContinue}
            >{$t.recordingReview.continueToTranscription}</button
          >
          <button
            class="text-action"
            onclick={() => onNavigate({ name: 'meeting', meetingId: meeting.id })}
            >{$t.recordingReview.backToMeeting}</button
          >
        </div>
      </aside>
    </section>
  {/if}
</main>
