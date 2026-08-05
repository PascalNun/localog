<script lang="ts">
  import { onDestroy } from 'svelte';
  import type {
    ActiveJob,
    AppRoute,
    MeetingSummary,
    ProjectSummary,
    ProtocolStyle,
    TranscriptDocument,
  } from '../workflow/types';
  import Icon from './Icon.svelte';
  import ProgressPanel from './ProgressPanel.svelte';
  import StageRail from './StageRail.svelte';

  export let project: ProjectSummary;
  export let meeting: MeetingSummary;
  export let transcript: TranscriptDocument | null;
  /** The style actually resolved for this meeting, or null when unknown. */
  export let protocolStyle: ProtocolStyle | null = null;
  export let job: ActiveJob | null;
  export let onNavigate: (route: AppRoute) => void;
  export let onGenerate: () => Promise<void>;
  export let onCancel: () => Promise<void>;
  export let onRetry: () => Promise<void>;
  export let onUpdateSegment: (segmentId: string, text: string) => Promise<void>;
  export let onUpdateSpeaker: (speaker: string, replacement: string) => Promise<void>;
  export let onLoadAudio: (
    meetingId: string,
  ) => Promise<{ source: string; durationMs: number | null } | null>;

  let isPlaying = false;
  let currentSeconds = 0;
  let query = '';
  let inspectorOpen = true;
  let saveState: 'saved' | 'saving' | 'failed' = transcript?.saveState ?? 'saved';
  let audioElement: HTMLAudioElement | null = null;
  let audioSource: string | null = null;
  let audioDuration = 0;
  let followPlayback = true;
  let audioError: string | null = null;
  let isScrubbing = false;
  let loadedAudioFor: string | null = null;

  $: segments = transcript?.segments ?? [];

  // Working audio only exists once the source has been prepared for transcription.
  $: void loadAudio(meeting.id);

  async function loadAudio(meetingId: string) {
    // Snapshot events fire often; only reload when the meeting actually changes.
    if (loadedAudioFor === meetingId) return;
    loadedAudioFor = meetingId;
    const audio = await onLoadAudio(meetingId);
    audioSource = audio?.source ?? null;
    // A different source means the previous transport state no longer applies.
    currentSeconds = 0;
    isPlaying = false;
    audioError = null;
    audioDuration = audio?.durationMs ? audio.durationMs / 1000 : 0;
  }

  // The segment under the playhead, used to highlight and to follow along.
  $: activeSegmentId =
    segments.find(
      (segment) =>
        currentSeconds * 1000 >= segment.startMs && currentSeconds * 1000 < segment.endMs,
    )?.id ?? null;

  $: if (activeSegmentId && isPlaying && followPlayback && !isEditingSegment())
    scrollSegmentIntoView(activeSegmentId);

  function isEditingSegment() {
    const active = window.document.activeElement;
    return Boolean(active && active.hasAttribute('data-segment-id'));
  }

  function scrollSegmentIntoView(segmentId: string) {
    const element = window.document.querySelector(`[data-segment-row="${segmentId}"]`);
    element?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
  }

  $: relevantJob = job?.meetingId === meeting.id && job.kind === 'generation' ? job : null;
  $: generationUnavailable = Boolean(relevantJob && relevantJob.state !== 'completed');
  $: filteredSegments = query.trim()
    ? segments.filter((segment) =>
        `${segment.speaker} ${segment.text}`.toLowerCase().includes(query.toLowerCase()),
      )
    : segments;
  $: speakers = [...new Set(segments.map((segment) => segment.speaker))];
  $: unclearCount = segments.filter((segment) => segment.needsReview).length;

  function togglePlayback() {
    if (!audioElement) return;
    if (audioElement.paused) {
      audioElement.play().catch(() => {
        audioError = 'This meeting’s working audio could not be played.';
      });
    } else {
      audioElement.pause();
    }
  }

  /// Move the playhead; clicking a segment jumps the audio to that moment.
  function seek(seconds: number) {
    currentSeconds = seconds;
    if (audioElement) audioElement.currentTime = seconds;
  }

  function segmentTimeLabel(milliseconds: number) {
    const seconds = Math.floor(milliseconds / 1000);
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const remainder = seconds % 60;
    return [hours, minutes, remainder].map((value) => value.toString().padStart(2, '0')).join(':');
  }

  async function saveSegment(segmentId: string, text: string) {
    saveState = 'saving';
    try {
      await onUpdateSegment(segmentId, text);
      saveState = 'saved';
    } catch {
      saveState = 'failed';
    }
  }

  function moveBetweenSegments(event: KeyboardEvent, segmentId: string) {
    if (!(event.metaKey || event.ctrlKey) || !['ArrowUp', 'ArrowDown'].includes(event.key)) return;
    const index = filteredSegments.findIndex((segment) => segment.id === segmentId);
    const next = event.key === 'ArrowUp' ? index - 1 : index + 1;
    const target = filteredSegments[next];
    if (!target) return;
    event.preventDefault();
    window.document.querySelector<HTMLTextAreaElement>(`[data-segment-id="${target.id}"]`)?.focus();
  }

  function timeLabel(seconds: number) {
    const minutes = Math.floor(seconds / 60)
      .toString()
      .padStart(2, '0');
    const remainder = Math.floor(seconds % 60)
      .toString()
      .padStart(2, '0');
    return `${minutes}:${remainder}`;
  }

  // Stop audio when leaving review so it cannot keep playing on another screen.
  onDestroy(() => audioElement?.pause());
</script>

<main class="workspace dense-workspace" id="main-content">
  <header class="workspace-header meeting-header">
    <div>
      <p class="breadcrumb">{project.name} <span>›</span> {meeting.title}</p>
      <h1 tabindex="-1">Transcript review</h1>
      <p>{meeting.occurredAt} · {meeting.durationLabel ?? 'Duration pending'}</p>
    </div>
    <button
      class="secondary-action inspector-toggle"
      onclick={() => (inspectorOpen = !inspectorOpen)}>Review details</button
    >
  </header>

  <StageRail meetingId={meeting.id} lifecycle={meeting.lifecycle} {onNavigate} />
  {#if relevantJob && relevantJob.state !== 'completed'}<ProgressPanel
      job={relevantJob}
      {onCancel}
      {onRetry}
    />{/if}

  <div class:without-inspector={!inspectorOpen} class="context-layout">
    <div class="transcript-main">
      <section class="audio-transport" aria-label="Meeting source context">
        {#if audioSource}
          <audio
            bind:this={audioElement}
            src={audioSource}
            preload="metadata"
            onloadedmetadata={() => {
              if (audioElement && Number.isFinite(audioElement.duration))
                audioDuration = audioElement.duration;
            }}
            ontimeupdate={() => {
              if (!isScrubbing) currentSeconds = audioElement?.currentTime ?? 0;
            }}
            onerror={() => (audioError = 'This meeting’s working audio could not be loaded.')}
            onplay={() => (isPlaying = true)}
            onpause={() => (isPlaying = false)}
            onended={() => (isPlaying = false)}
          ></audio>
          <button
            class="play-button"
            onclick={togglePlayback}
            aria-label={isPlaying ? 'Pause audio' : 'Play audio'}
            ><Icon name={isPlaying ? 'pause' : 'play'} size={20} /></button
          >
          <span class="time-readout">{timeLabel(currentSeconds)}</span>
          <input
            aria-label="Seek audio"
            class="seek-range"
            type="range"
            min="0"
            max={Math.max(audioDuration, 1)}
            step="0.1"
            value={currentSeconds}
            onpointerdown={() => (isScrubbing = true)}
            onpointerup={() => (isScrubbing = false)}
            onkeydown={() => (isScrubbing = true)}
            onkeyup={() => (isScrubbing = false)}
            oninput={(event) => seek(Number(event.currentTarget.value))}
          />
          <span class="time-readout">{timeLabel(audioDuration)}</span>
          <button
            class="quiet-action follow-toggle"
            aria-pressed={followPlayback}
            title="Scroll the transcript to the segment being played"
            onclick={() => (followPlayback = !followPlayback)}>Follow</button
          >
        {:else}
          <p class="transport-empty">
            Working audio becomes available once this meeting has been transcribed.
          </p>
        {/if}
      </section>
      {#if audioError}<p class="setting-error" role="alert">{audioError}</p>{/if}

      <div class="transcript-toolbar">
        <label class="search-field"
          ><Icon name="search" size={16} /><span class="sr-only">Search transcript</span><input
            bind:value={query}
            placeholder="Search transcript"
          /></label
        >
        <span class="review-summary"
          >{unclearCount ? `${unclearCount} segment flagged` : 'No segments flagged'}</span
        >
      </div>

      <section class="transcript-list" aria-label="Editable transcript">
        {#each filteredSegments as segment (segment.id)}
          <article
            class:needs-review={segment.needsReview}
            class:playing={segment.id === activeSegmentId}
            class="transcript-segment"
            data-segment-row={segment.id}
          >
            <button
              class="timestamp"
              onclick={() => seek(segment.startMs / 1000)}
              title="Jump to {segmentTimeLabel(segment.startMs)}"
              >{segmentTimeLabel(segment.startMs)}</button
            >
            <span class="speaker-label">{segment.speaker}</span>
            <label
              ><span class="sr-only">Transcript text at {segmentTimeLabel(segment.startMs)}</span
              ><textarea
                rows="2"
                value={segment.text}
                data-segment-id={segment.id}
                onkeydown={(event) => moveBetweenSegments(event, segment.id)}
                onblur={(event) => saveSegment(segment.id, event.currentTarget.value)}
              ></textarea></label
            >
            {#if segment.needsReview}<span class="review-flag"
                ><Icon name="warning" size={14} /> Check term</span
              >{/if}
          </article>
        {/each}
      </section>

      <footer class="workspace-action-bar">
        <div>
          <strong
            >{saveState === 'saving'
              ? 'Saving locally…'
              : saveState === 'failed'
                ? 'Autosave failed — your last saved work is intact'
                : transcript?.isDirty
                  ? 'Working edits saved locally'
                  : 'Transcript revision saved'}</strong
          ><small>Speaker identity is never inferred in this milestone.</small>
        </div>
        <button class="primary-action" onclick={onGenerate} disabled={generationUnavailable}
          >Generate protocol <Icon name="arrow" /></button
        >
      </footer>
    </div>

    {#if inspectorOpen}
      <aside class="context-inspector" aria-label="Transcript review details">
        <div class="inspector-heading">
          <div>
            <p class="eyebrow">Review</p>
            <h2>Speakers</h2>
          </div>
          <button
            class="icon-button compact"
            aria-label="Close inspector"
            onclick={() => (inspectorOpen = false)}><Icon name="close" size={16} /></button
          >
        </div>
        <p class="inspector-copy">
          Rename generic labels only when you know the participant. Automatic diarisation is not
          implied.
        </p>
        <div class="speaker-list">
          {#each speakers as speaker, index (speaker)}
            <label
              ><span class="speaker-token">S{index + 1}</span><input
                value={speaker}
                aria-label={`Rename ${speaker}`}
                onblur={(event) => onUpdateSpeaker(speaker, event.currentTarget.value)}
              /></label
            >
          {/each}
        </div>
        <div class="inspector-section">
          <p class="eyebrow">Unclear terms</p>
          <h3>{unclearCount ? `${unclearCount} to check` : 'None remaining'}</h3>
          <p>
            Editing a flagged segment marks it reviewed. Vocabulary suggestions will be validated in
            the transcription spike.
          </p>
        </div>
        {#if protocolStyle}
          <div class="inspector-section">
            <p class="eyebrow">Protocol style</p>
            <h3>{protocolStyle.name}</h3>
            <p>{protocolStyle.description}</p>
          </div>
        {/if}
      </aside>
    {/if}
  </div>
</main>
