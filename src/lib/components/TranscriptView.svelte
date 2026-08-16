<script lang="ts">
  import { onDestroy } from 'svelte';
  import type {
    ActiveJob,
    AppRoute,
    MeetingSummary,
    ProjectSummary,
    ProtocolStyle,
    TranscriptDocument,
    TranscriptSegment,
    NameCandidate,
    CorrectionMatch,
    AppliedCorrection,
  } from '../workflow/types';
  import { COMMON_MEETING_LANGUAGES } from '../workflow/languages';
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
  export let onRerunTranscription: () => Promise<void>;
  export let onUpdateLanguage: (language: string) => Promise<void>;
  export let onUpdateSegment: (segmentId: string, text: string) => Promise<void>;
  export let onDeleteSegment: (segmentId: string) => Promise<void>;
  export let onUpdateSpeaker: (speaker: string, replacement: string) => Promise<void>;
  export let onLoadAudio: (
    meetingId: string,
  ) => Promise<{ source: string; durationMs: number | null } | null>;
  export let onFindNameCandidates: (meetingId: string) => Promise<NameCandidate[]> = async () => [];
  export let onPreviewCorrection: (
    meetingId: string,
    wrong: string,
    right: string,
  ) => Promise<CorrectionMatch[]> = async () => [];
  export let onApplyCorrection: (
    meetingId: string,
    correction: AppliedCorrection,
  ) => Promise<void> = async () => undefined;

  /// Correcting a name the transcriber never heard right.
  ///
  /// The panel this replaces said "322 to check" on a real meeting, which is not a
  /// task anybody starts. These are the words it was never sure of — a handful, not
  /// a third of the transcript.
  let candidates: NameCandidate[] = [];
  let candidatesFor = '';
  $: if (transcript && candidatesFor !== meeting.id) {
    candidatesFor = meeting.id;
    correcting = null;
    void onFindNameCandidates(meeting.id).then((found) => (candidates = found));
  }

  /** The candidate being corrected, and what somebody is typing for it. */
  let correcting: { heard: string; spelling: string } | null = null;
  let matches: CorrectionMatch[] = [];
  /** Occurrences to leave alone, because a wrong spelling can be an ordinary word. */
  let declined = new Set<string>();
  let remember = true;
  let applying = false;
  let applied = '';

  async function startCorrecting(candidate: NameCandidate) {
    applied = '';
    correcting = { heard: candidate.heard, spelling: candidate.heard };
    declined = new Set();
    matches = await onPreviewCorrection(meeting.id, candidate.heard, candidate.heard);
  }

  function toggleDeclined(segmentId: string) {
    const next = new Set(declined);
    if (next.has(segmentId)) next.delete(segmentId);
    else next.add(segmentId);
    declined = next;
  }

  $: keptMatches = matches.filter((match) => !declined.has(match.segmentId));

  async function applyCorrection() {
    if (!correcting || !correcting.spelling.trim() || !keptMatches.length) return;
    applying = true;
    const wrong = correcting.heard;
    const right = correcting.spelling.trim();
    try {
      await onApplyCorrection(meeting.id, {
        wrong,
        right,
        keptSegmentIds: declined.size ? keptMatches.map((match) => match.segmentId) : [],
        remember,
      });
      applied = `${wrong} → ${right} in ${keptMatches.length} ${
        keptMatches.length === 1 ? 'place' : 'places'
      }${remember ? ', and kept for this project' : ''}.`;
      candidates = candidates.filter((candidate) => candidate.heard !== wrong);
      correcting = null;
      matches = [];
    } finally {
      applying = false;
    }
  }

  let isPlaying = false;
  let currentSeconds = 0;
  let query = '';
  /** Narrows the list to the passages whisper reported as unclear. */
  let onlyFlagged = false;
  let inspectorOpen = true;
  let saveState: 'saved' | 'saving' | 'failed' = transcript?.saveState ?? 'saved';
  let audioElement: HTMLAudioElement | null = null;
  let audioSource: string | null = null;
  let audioDuration = 0;
  let followPlayback = true;
  let audioError: string | null = null;
  let isScrubbing = false;
  let loadedAudioFor: string | null = null;
  let editingLanguage = false;
  let languageDraft = meeting.language;
  let languageError = '';
  let rerunError = '';
  let rerunning = false;

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

  $: relevantJob = job?.meetingId === meeting.id ? job : null;
  // A transcript rerun and generation both use the single heavy-work lane.
  // Keep the editor calm while either job is active.
  $: generationUnavailable = Boolean(relevantJob && relevantJob.state !== 'completed');
  $: filteredSegments = segments
    .filter((segment) => !onlyFlagged || segment.needsReview)
    .filter(
      (segment) =>
        !query.trim() ||
        `${segment.speaker} ${segment.text}`.toLowerCase().includes(query.toLowerCase()),
    );
  $: speakers = [...new Set(segments.map((segment) => segment.speaker))];
  $: speakerResolution = transcript?.speakerResolution ?? 'unavailable';
  $: speakerResolutionCopy =
    speakerResolution === 'resolved'
      ? 'Speaker turns were resolved locally. Labels are provisional—rename them only when you know the participant.'
      : speakerResolution === 'failed'
        ? 'Speaker separation did not produce usable turns for this run. The transcript is intact and uses neutral labels; you can continue with manual labels.'
        : speakerResolution === 'unknown'
          ? 'This older transcript does not record whether speaker separation ran. Its neutral labels are not evidence that there was only one speaker.'
          : 'Speaker separation was not available for this run. The transcript is intact and uses a neutral label; you can still rename it manually.';
  $: unclearCount = segments.filter((segment) => segment.needsReview).length;

  // Whisper reports how sure it was of each word. Where it was not sure, the word
  // is named rather than merely marked, so the question put to the reader is one
  // they can answer from memory of the meeting.
  function uncertainLabel(segment: TranscriptSegment): string {
    const words = segment.uncertainWords ?? [];
    if (words.length === 0) return 'Check wording';
    return `Check ${words.map((word) => `“${word}”`).join(', ')}`;
  }

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

  let deletingSegment = '';

  // Removing a line is no more permanent than rewriting one, and rewriting asks
  // nobody's permission: the committed revision is the way back from both. A
  // dialog here would suggest this is the dangerous edit, which it is not.
  async function removeSegment(segmentId: string) {
    deletingSegment = segmentId;
    saveState = 'saving';
    try {
      await onDeleteSegment(segmentId);
      saveState = 'saved';
    } catch {
      saveState = 'failed';
    } finally {
      deletingSegment = '';
    }
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

  async function saveLanguage() {
    languageError = '';
    try {
      await onUpdateLanguage(languageDraft);
      editingLanguage = false;
    } catch (error) {
      languageError = error instanceof Error ? error.message : String(error);
    }
  }

  async function rerunTranscription() {
    if (rerunning || !transcript) return;
    const confirmed = window.confirm(
      `Rerun transcription in ${meeting.language}? The current transcript will stay until the new result is committed, then this working transcript will be replaced.`,
    );
    if (!confirmed) return;
    rerunning = true;
    rerunError = '';
    try {
      await onRerunTranscription();
    } catch (error) {
      rerunError = error instanceof Error ? error.message : String(error);
    } finally {
      rerunning = false;
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
        {#if unclearCount}
          <button
            class="text-action review-summary"
            aria-pressed={onlyFlagged}
            onclick={() => (onlyFlagged = !onlyFlagged)}
            >{onlyFlagged ? 'Showing' : 'Show'}
            {unclearCount === 1 ? '1 unclear passage' : `${unclearCount} unclear passages`}</button
          >
        {:else}
          <span class="review-summary">Nothing flagged as unclear</span>
        {/if}
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
            {#if segment.needsReview}<span class="review-flag" title={uncertainLabel(segment)}
                ><Icon name="warning" size={14} /> {uncertainLabel(segment)}</span
              >{/if}
            <button
              class="segment-remove"
              onclick={() => removeSegment(segment.id)}
              disabled={deletingSegment === segment.id || (transcript?.segments.length ?? 0) <= 1}
              title="Remove this line from the transcript"
              aria-label="Remove the line at {segmentTimeLabel(segment.startMs)}"
              ><Icon name="close" size={14} /></button
            >
          </article>
        {/each}
      </section>

      <footer class="workspace-action-bar">
        <div>
          <strong
            >{saveState === 'saving'
              ? 'Saving…'
              : saveState === 'failed'
                ? 'Autosave failed — your last saved work is intact'
                : transcript?.isDirty
                  ? 'Edits saved'
                  : 'Transcript revision saved'}</strong
          ><small>Speaker labels are a starting point—rename them to the people who spoke.</small>
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
          {speakerResolutionCopy}
        </p>
        {#if speakerResolution !== 'resolved'}
          <p class="speaker-status" role="status">
            <Icon name="info" size={14} />
            {speakerResolution === 'failed'
              ? 'Speaker separation is not available in this installation yet. You can continue with manual labels.'
              : speakerResolution === 'unknown'
                ? 'Rerun this transcript to record a current speaker-separation result.'
                : 'Speaker separation was not available for this run. You can continue with manual labels.'}
          </p>
        {/if}
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
          <p class="eyebrow">Transcription input</p>
          <h3>Language</h3>
          {#if editingLanguage}<label class="setting-field"
              ><span class="sr-only">Meeting language</span><input
                bind:value={languageDraft}
                list="transcript-languages"
                aria-label="Meeting language"
              /><datalist id="transcript-languages">
                {#each COMMON_MEETING_LANGUAGES as language (language)}<option value={language}
                  ></option>{/each}
              </datalist></label
            >
            <div class="inspector-actions">
              <button
                class="secondary-action"
                onclick={saveLanguage}
                disabled={Boolean(relevantJob && relevantJob.state !== 'completed')}
                >Save language</button
              >
              <button class="quiet-action" onclick={() => (editingLanguage = false)}>Cancel</button>
            </div>{:else}<p>{meeting.language}</p>
            <button
              class="quiet-action"
              disabled={Boolean(relevantJob && relevantJob.state !== 'completed')}
              onclick={() => {
                languageDraft = meeting.language;
                editingLanguage = true;
              }}>Change language</button
            >{/if}
          {#if languageError}<p class="setting-error" role="alert">{languageError}</p>{/if}
          <button
            class="secondary-action rerun-transcription"
            onclick={rerunTranscription}
            disabled={rerunning || Boolean(relevantJob && relevantJob.state !== 'completed')}
            >{rerunning ? 'Preparing a new transcript…' : 'Rerun transcription'}</button
          >
          <small class="inspector-note"
            >Use this after changing the language or transcription settings. The new run is recorded
            as a separate revision.</small
          >
          {#if rerunError}<p class="setting-error" role="alert">{rerunError}</p>{/if}
        </div>
        <div class="inspector-section">
          <p class="eyebrow">Names the transcriber was unsure of</p>

          {#if correcting}
            {@const editing = correcting}
            <h3>What should it say?</h3>
            <label class="correction-field">
              <span>Heard as “{editing.heard}”</span>
              <input
                type="text"
                bind:value={editing.spelling}
                placeholder="Correct spelling"
                onkeydown={(event) => {
                  if (event.key === 'Enter') void applyCorrection();
                  if (event.key === 'Escape') correcting = null;
                }}
              />
            </label>

            {#if matches.length}
              <p class="correction-note">
                Found in {matches.length}
                {matches.length === 1 ? 'place' : 'places'}. Untick any that should stay as they are.
              </p>
              <ul class="correction-matches">
                {#each matches as match (match.segmentId)}
                  <li>
                    <label>
                      <input
                        type="checkbox"
                        checked={!declined.has(match.segmentId)}
                        onchange={() => toggleDeclined(match.segmentId)}
                      />
                      <span>{match.context}</span>
                    </label>
                  </li>
                {/each}
              </ul>
            {/if}

            <label class="correction-remember">
              <input type="checkbox" bind:checked={remember} />
              <span>Remember for this project, so the next meeting spells it correctly</span>
            </label>

            <div class="correction-actions">
              <button
                class="secondary-action"
                disabled={applying || !keptMatches.length || !editing.spelling.trim()}
                onclick={applyCorrection}
              >
                {applying ? 'Correcting…' : `Correct ${keptMatches.length}`}
              </button>
              <button class="text-action" onclick={() => (correcting = null)}>Cancel</button>
            </div>
          {:else if candidates.length}
            <h3>{candidates.length} never got right</h3>
            <p>Are any of these names? Correcting one repairs this transcript and remembers it.</p>
            <ul class="correction-candidates">
              {#each candidates as candidate (candidate.heard)}
                <li>
                  <button class="candidate" onclick={() => startCorrecting(candidate)}>
                    <span class="candidate-word">{candidate.heard}</span>
                    <span class="candidate-count">{candidate.occurrences}×</span>
                    <span class="candidate-context">{candidate.context}</span>
                  </button>
                </li>
              {/each}
            </ul>
          {:else}
            <h3>Nothing to check</h3>
            <p>
              Every name the transcriber was unsure of has been dealt with. {unclearCount
                ? `${unclearCount} passages are still flagged as unclear for other reasons.`
                : ''}
            </p>
          {/if}

          {#if applied}<p class="correction-applied" role="status">{applied}</p>{/if}
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
