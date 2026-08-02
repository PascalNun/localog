<script lang="ts">
  import { onDestroy } from 'svelte';
  import type {
    ActiveJob,
    AppRoute,
    MeetingSummary,
    ProjectSummary,
    TranscriptSegment,
  } from '../workflow/types';
  import Icon from './Icon.svelte';
  import ProgressPanel from './ProgressPanel.svelte';
  import StageRail from './StageRail.svelte';

  export let project: ProjectSummary;
  export let meeting: MeetingSummary;
  export let segments: TranscriptSegment[];
  export let job: ActiveJob | null;
  export let onNavigate: (route: AppRoute) => void;
  export let onGenerate: () => Promise<void>;
  export let onCancel: () => Promise<void>;
  export let onRetry: () => Promise<void>;
  export let onUpdateSegment: (segmentId: string, text: string) => Promise<void>;
  export let onUpdateSpeaker: (speaker: string, replacement: string) => Promise<void>;

  let isPlaying = false;
  let currentSeconds = 0;
  let query = '';
  let inspectorOpen = true;
  let playbackTimer: ReturnType<typeof setInterval> | null = null;

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
    isPlaying = !isPlaying;
    if (isPlaying && !playbackTimer) {
      playbackTimer = setInterval(() => {
        currentSeconds = Math.min(102, currentSeconds + 1);
        if (currentSeconds >= 102) stopPlayback();
      }, 1000);
    } else if (!isPlaying) stopPlayback();
  }

  function stopPlayback() {
    isPlaying = false;
    if (playbackTimer) clearInterval(playbackTimer);
    playbackTimer = null;
  }

  function seek(seconds: number) {
    currentSeconds = seconds;
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

  onDestroy(stopPlayback);
</script>

<main class="workspace dense-workspace" id="main-content">
  <header class="workspace-header meeting-header">
    <div>
      <p class="breadcrumb">{project.name} <span>›</span> {meeting.title}</p>
      <h1 tabindex="-1">Transcript review</h1>
      <p>{meeting.occurredAt} · {meeting.durationLabel} · Generic speaker labels</p>
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
      <section class="audio-transport" aria-label="Synthetic audio transport">
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
          max="102"
          bind:value={currentSeconds}
        />
        <span class="time-readout">01:42</span>
        <span class="speed-control">1×</span>
      </section>

      <div class="transcript-toolbar">
        <label class="search-field"
          ><Icon name="search" size={16} /><span class="sr-only">Search transcript</span><input
            bind:value={query}
            placeholder="Search transcript"
          /></label
        >
        <span class="review-summary"
          >{unclearCount ? `${unclearCount} segment needs review` : 'Review complete'}</span
        >
      </div>

      <section class="transcript-list" aria-label="Editable transcript">
        {#each filteredSegments as segment (segment.id)}
          <article class:needs-review={segment.needsReview} class="transcript-segment">
            <button class="timestamp" onclick={() => seek(segment.startSeconds)}
              >{segment.startLabel}</button
            >
            <span class="speaker-label">{segment.speaker}</span>
            <label
              ><span class="sr-only">Transcript text at {segment.startLabel}</span><textarea
                rows="2"
                value={segment.text}
                onblur={(event) => onUpdateSegment(segment.id, event.currentTarget.value)}
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
          <strong>Transcript edits save locally</strong><small
            >Speaker identity is never inferred in this demo.</small
          >
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
        <div class="inspector-section">
          <p class="eyebrow">Protocol style</p>
          <h3>Formal minutes</h3>
          <p>Project default · professional preset</p>
        </div>
      </aside>
    {/if}
  </div>
</main>
