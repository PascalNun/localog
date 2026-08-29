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
    Introduction,
    CorrectionMatch,
    AppliedCorrection,
  } from '../workflow/types';
  import { COMMON_MEETING_LANGUAGES } from '../workflow/languages';
  import Icon from './Icon.svelte';
  import ProgressPanel from './ProgressPanel.svelte';
  import StageRail from './StageRail.svelte';
  import { errorMessage } from '../errors';
  import { t } from '../i18n';
  import { formatMeetingDate } from '../protocol/document';

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
  /// Take the transcript out of the application as a file.
  ///
  /// The protocol has left four ways since it existed and the transcript could
  /// not leave at all — though it is the thing somebody spent an hour correcting,
  /// and the record closest to what was actually said. Somebody sending it to a
  /// participant to check had to copy it out of the screen by hand.
  export let onExportTranscript: (format: 'markdown' | 'text') => Promise<void> = async () =>
    undefined;
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
  ) => Promise<number> = async () => 0;

  /// Reading who is in the meeting, once per project.
  ///
  /// A first meeting has no names, and asking somebody to write them from memory
  /// before they have heard the recording asks for the wrong thing at the wrong
  /// time. Most meetings open with people saying who they are, and every name comes
  /// back misspelt — which is what makes this work: correcting a list you recognise
  /// is easier than composing one you have to remember.
  ///
  /// Behind a button rather than automatic: it is a minute of model work, and this
  /// application runs one heavy task at a time.
  export let projectHasNames = false;
  export let onFindIntroductions: (meetingId: string) => Promise<Introduction[]> = async () => [];

  let exportError = '';

  async function exportTranscript(format: 'markdown' | 'text') {
    exportError = '';
    try {
      await onExportTranscript(format);
    } catch (cause) {
      exportError = errorMessage(cause);
    }
  }

  let introductions: Introduction[] | null = null;
  let spellings: Record<string, string> = {};
  let reading = false;
  let readError = '';
  let introductionsSaved = '';

  async function readIntroductions() {
    reading = true;
    readError = '';
    try {
      const found = await onFindIntroductions(meeting.id);
      introductions = found;
      spellings = Object.fromEntries(found.map((person) => [person.heard, person.heard]));
    } catch (cause) {
      readError = errorMessage(cause);
    } finally {
      reading = false;
    }
  }

  /** Only the ones somebody actually changed are worth applying. */
  $: correctedNames = (introductions ?? []).filter((person) => {
    const corrected = spellings[person.heard]?.trim();
    return Boolean(corrected) && corrected !== person.heard;
  });

  async function saveIntroductions() {
    reading = true;
    try {
      let changed = 0;
      for (const person of correctedNames) {
        const corrected = spellings[person.heard]?.trim();
        if (!corrected) continue;
        changed += await onApplyCorrection(meeting.id, {
          wrong: person.heard,
          right: corrected,
          keptSegmentIds: [],
          remember: true,
        });
      }
      introductionsSaved = `${correctedNames.length} ${
        correctedNames.length === 1 ? 'name' : 'names'
      } corrected in ${changed} ${changed === 1 ? 'place' : 'places'}, and kept for this project.`;
      introductions = null;
    } finally {
      reading = false;
    }
  }

  /// Correcting a name the transcriber never heard right.
  ///
  /// The panel this replaces said "322 to check" on a real meeting, which is not a
  /// task anybody starts. These are the words it was never sure of — a handful, not
  /// a third of the transcript.
  let candidates: NameCandidate[] = [];
  let candidatesFor = '';
  $: if (transcript && candidatesFor !== meeting.id) {
    // Read by this block's own condition on the next run, which the linter cannot
    // see; and assigning the candidates cannot retrigger a condition that depends
    // only on the meeting.
    // eslint-disable-next-line no-useless-assignment
    candidatesFor = meeting.id;
    correcting = null;
    // eslint-disable-next-line svelte/infinite-reactive-loop
    void onFindNameCandidates(meeting.id).then((found) => (candidates = found));
  }

  /** The candidate being corrected, and what somebody is typing for it. */
  let correcting: { heard: string; spelling: string } | null = null;
  let matches: CorrectionMatch[] = [];
  /** Occurrences to leave alone, because a wrong spelling can be an ordinary word. */
  let declined: string[] = [];
  let remember = true;
  let applying = false;
  let applied = '';

  async function startCorrecting(candidate: NameCandidate) {
    applied = '';
    correcting = { heard: candidate.heard, spelling: candidate.heard };
    declined = [];
    matches = await onPreviewCorrection(meeting.id, candidate.heard, candidate.heard);
  }

  function toggleDeclined(segmentId: string) {
    declined = declined.includes(segmentId)
      ? declined.filter((id) => id !== segmentId)
      : [...declined, segmentId];
  }

  $: keptMatches = matches.filter((match) => !declined.includes(match.segmentId));

  async function applyCorrection() {
    if (!correcting || !correcting.spelling.trim() || !keptMatches.length) return;
    applying = true;
    const wrong = correcting.heard;
    const right = correcting.spelling.trim();
    try {
      // What it reports is what the transcript says it did, not what was asked of
      // it: a place whose text moved since the review is skipped rather than
      // corrected blindly, and saying otherwise would be a lie this screen has no
      // way to notice.
      const changed = await onApplyCorrection(meeting.id, {
        wrong,
        right,
        keptSegmentIds: declined.length ? keptMatches.map((match) => match.segmentId) : [],
        remember,
      });
      applied = changed
        ? `${wrong} → ${right} in ${changed} ${changed === 1 ? 'place' : 'places'}${
            remember ? ', and kept for this project' : ''
          }.`
        : `Nothing was changed — the transcript has moved since these were found.`;
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
  /// Open beside the document where there is room, closed where the drawer would
  /// cover it.
  ///
  /// Below 900px the inspector is a drawer laid over the workspace rather than a
  /// column beside it, so starting open means arriving at a document with its middle
  /// hidden — the find bar and the right-hand third of every line were behind it.
  /// The direction is plain that the document must remain usable, so at that size it
  /// is opened deliberately rather than by default.
  let inspectorOpen = typeof window === 'undefined' || window.innerWidth > 900;
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
  /// Where the recording stops, and what it was in the middle of.
  ///
  /// A recording can end before the meeting does — the project's own reference
  /// recording stops mid-discussion, and about a fifth of the protocol written for
  /// that meeting describes what was said afterwards. A protocol generated from it
  /// is silently partial and reads as complete.
  ///
  /// No attempt is made to detect this. Whether a meeting had finished is a judgement
  /// about a room, and the person who was in it can make it in a second from the last
  /// thing that was said. Guessing at it in the application would be a heuristic in
  /// one language that is wrong in others.
  $: lastSegment = segments.length ? segments[segments.length - 1] : null;

  $: speakers = [...new Set(segments.map((segment) => segment.speaker))];
  $: speakerResolution = transcript?.speakerResolution ?? 'unavailable';
  $: speakerResolutionCopy =
    speakerResolution === 'resolved'
      ? $t.transcript.speakersResolved
      : speakerResolution === 'failed'
        ? $t.transcript.speakersFailed
        : speakerResolution === 'unknown'
          ? $t.transcript.speakersUnknown
          : $t.transcript.speakersUnavailable;
  $: unclearCount = segments.filter((segment) => segment.needsReview).length;

  // Whisper reports how sure it was of each word. Where it was not sure, the word
  // is named rather than merely marked, so the question put to the reader is one
  // they can answer from memory of the meeting.
  function uncertainLabel(segment: TranscriptSegment): string {
    const words = segment.uncertainWords ?? [];
    if (words.length === 0) return $t.transcript.checkWording;
    return `Check ${words.map((word) => `“${word}”`).join(', ')}`;
  }

  function togglePlayback() {
    if (!audioElement) return;
    if (audioElement.paused) {
      audioElement.play().catch(() => {
        audioError = $t.transcript.audioUnplayable;
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
      languageError = errorMessage(error);
    }
  }

  async function rerunTranscription() {
    if (rerunning || !transcript) return;
    const confirmed = window.confirm($t.transcript.rerunConfirm(meeting.language));
    if (!confirmed) return;
    rerunning = true;
    rerunError = '';
    try {
      await onRerunTranscription();
    } catch (error) {
      rerunError = errorMessage(error);
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

<main class="workspace stage-workspace" id="main-content">
  <header class="workspace-header meeting-header">
    <div>
      <p class="breadcrumb">{project.name} <span>›</span> {meeting.title}</p>
      <h1 tabindex="-1">{$t.transcript.heading}</h1>
      <p>
        {formatMeetingDate(meeting.occurredAt, $t)} ·
        {meeting.durationLabel ?? $t.transcript.durationPending}
      </p>
    </div>
    <div class="transcript-header-actions">
      <select
        class="transcript-export"
        value=""
        aria-label={$t.transcript.exportLabel}
        disabled={!transcript || transcript.segments.length === 0}
        onchange={(event) => {
          const chosen = event.currentTarget.value;
          event.currentTarget.value = '';
          if (chosen === 'markdown' || chosen === 'text') void exportTranscript(chosen);
        }}
      >
        <option value="">{$t.transcript.exportTranscript}</option>
        <option value="markdown">{$t.transcript.asMarkdown}</option>
        <option value="text">{$t.transcript.asPlainText}</option>
      </select>
      <button
        class="secondary-action inspector-toggle"
        onclick={() => (inspectorOpen = !inspectorOpen)}>{$t.transcript.reviewDetails}</button
      >
    </div>
  </header>
  {#if exportError}<p class="setting-error" role="alert">{exportError}</p>{/if}

  <StageRail meetingId={meeting.id} lifecycle={meeting.lifecycle} {onNavigate} />
  {#if relevantJob && relevantJob.state !== 'completed'}<ProgressPanel
      job={relevantJob}
      {onCancel}
      {onRetry}
    />{/if}

  <div class:without-inspector={!inspectorOpen} class="context-layout">
    <div class="transcript-main">
      <section class="audio-transport" aria-label={$t.transcript.sourceContext}>
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
            aria-label={$t.transcript.seekAudio}
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
            title={$t.transcript.followLabel}
            onclick={() => (followPlayback = !followPlayback)}>{$t.transcript.follow}</button
          >
        {:else}
          <p class="transport-empty">
            {$t.transcript.workingAudioLater}
          </p>
        {/if}
      </section>
      {#if audioError}<p class="setting-error" role="alert">{audioError}</p>{/if}

      <div class="transcript-toolbar">
        <label class="search-field"
          ><Icon name="search" size={16} /><span class="sr-only"
            >{$t.transcript.searchTranscript}</span
          ><input bind:value={query} placeholder={$t.transcript.searchTranscript} /></label
        >
        {#if unclearCount}
          <button
            class="text-action review-summary"
            aria-pressed={onlyFlagged}
            onclick={() => (onlyFlagged = !onlyFlagged)}
            >{onlyFlagged ? $t.transcript.showing : $t.transcript.show}
            {unclearCount === 1
              ? $t.transcript.onePassage
              : $t.transcript.manyPassages(unclearCount)}</button
          >
        {:else}
          <span class="review-summary">{$t.transcript.nothingFlagged}</span>
        {/if}
      </div>

      <section class="transcript-list" aria-label={$t.transcript.editableTranscript}>
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
              title={$t.transcript.removeLine}
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
          ><small>{$t.transcript.speakerHint}</small>
        </div>
        <button class="primary-action" onclick={onGenerate} disabled={generationUnavailable}
          >{$t.transcript.generateProtocol} <Icon name="arrow" /></button
        >
      </footer>
    </div>

    {#if inspectorOpen}
      <aside class="context-inspector" aria-label={$t.transcript.detailsLabel}>
        <div class="inspector-heading">
          <div>
            <p class="eyebrow">{$t.shell.breadcrumbReview}</p>
            <h2>{$t.transcript.speakers}</h2>
          </div>
          <button
            class="icon-button compact"
            aria-label={$t.transcript.closeInspector}
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
        {#if lastSegment}
          <div class="inspector-section">
            <p class="eyebrow">{$t.transcript.whereRecordingStops}</p>
            <h3>{timeLabel(lastSegment.endMs / 1000)}</h3>
            <p class="recording-last-words">“{lastSegment.text}”</p>
            <p>
              {$t.transcript.recordingEndsNote}
            </p>
          </div>
        {/if}

        <div class="inspector-section">
          <p class="eyebrow">{$t.transcript.transcriptionInput}</p>
          <h3>{$t.transcript.language}</h3>
          {#if editingLanguage}<label class="setting-field"
              ><span class="sr-only">{$t.transcript.meetingLanguage}</span><input
                bind:value={languageDraft}
                list="transcript-languages"
                aria-label={$t.transcript.meetingLanguage}
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
                >{$t.transcript.saveLanguage}</button
              >
              <button class="quiet-action" onclick={() => (editingLanguage = false)}
                >{$t.transcript.cancel}</button
              >
            </div>{:else}<p>{meeting.language}</p>
            <button
              class="quiet-action"
              disabled={Boolean(relevantJob && relevantJob.state !== 'completed')}
              onclick={() => {
                languageDraft = meeting.language;
                editingLanguage = true;
              }}>{$t.transcript.changeLanguage}</button
            >{/if}
          {#if languageError}<p class="setting-error" role="alert">{languageError}</p>{/if}
          <button
            class="secondary-action rerun-transcription"
            onclick={rerunTranscription}
            disabled={rerunning || Boolean(relevantJob && relevantJob.state !== 'completed')}
            >{rerunning ? $t.transcript.rerunPreparing : $t.transcript.rerun}</button
          >
          <small class="inspector-note">{$t.transcript.rerunNote}</small>
          {#if rerunError}<p class="setting-error" role="alert">{rerunError}</p>{/if}
        </div>
        {#if !projectHasNames || introductions}
          <div class="inspector-section">
            <p class="eyebrow">{$t.transcript.whoIsHere}</p>
            {#if introductions}
              <h3>{$t.transcript.introducedThemselves(introductions.length)}</h3>
              <p>
                {$t.transcript.speltAsHeard}
              </p>
              <ul class="introduction-list">
                {#each introductions as person (person.heard)}
                  <li>
                    <input
                      type="text"
                      bind:value={spellings[person.heard]}
                      aria-label="Name heard as {person.heard}"
                    />
                    <span class="introduction-role">{person.role}</span>
                  </li>
                {/each}
              </ul>
              <div class="correction-actions">
                <button
                  class="secondary-action"
                  disabled={reading || !correctedNames.length}
                  onclick={saveIntroductions}
                >
                  {correctedNames.length
                    ? `Correct ${correctedNames.length}`
                    : 'Nothing changed yet'}
                </button>
                <button class="text-action" onclick={() => (introductions = null)}
                  >{$t.transcript.close}</button
                >
              </div>
            {:else}
              <h3>{$t.transcript.noNamesYet(project.name)}</h3>
              <p>
                {$t.transcript.openingNote}
              </p>
              <button class="secondary-action" disabled={reading} onclick={readIntroductions}>
                {reading ? 'Reading the opening…' : 'Read who is in this meeting'}
              </button>
              <small class="inspector-note">{$t.transcript.aboutAMinute}</small>
            {/if}
            {#if readError}<p class="setting-error" role="alert">{readError}</p>{/if}
            {#if introductionsSaved}<p class="correction-applied" role="status">
                {introductionsSaved}
              </p>{/if}
          </div>
        {/if}

        <div class="inspector-section">
          <p class="eyebrow">{$t.transcript.unsureNames}</p>

          {#if correcting}
            {@const editing = correcting}
            <h3>{$t.transcript.whatShouldItSay}</h3>
            <label class="correction-field">
              <span>Heard as “{editing.heard}”</span>
              <input
                type="text"
                bind:value={editing.spelling}
                placeholder={$t.transcript.correctSpelling}
                onkeydown={(event) => {
                  if (event.key === 'Enter') void applyCorrection();
                  if (event.key === 'Escape') correcting = null;
                }}
              />
            </label>

            {#if matches.length}
              <p class="correction-note">
                {$t.transcript.foundInPlaces(matches.length)}
              </p>
              <ul class="correction-matches">
                {#each matches as match (match.segmentId)}
                  <li>
                    <label>
                      <input
                        type="checkbox"
                        checked={!declined.includes(match.segmentId)}
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
              <span>{$t.transcript.rememberForProject}</span>
            </label>

            <div class="correction-actions">
              <button
                class="secondary-action"
                disabled={applying || !keptMatches.length || !editing.spelling.trim()}
                onclick={applyCorrection}
              >
                {applying ? 'Correcting…' : `Correct ${keptMatches.length}`}
              </button>
              <button class="text-action" onclick={() => (correcting = null)}
                >{$t.transcript.cancel}</button
              >
            </div>
          {:else if candidates.length}
            <h3>{candidates.length} never got right</h3>
            <p>{$t.transcript.areAnyNames}</p>
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
            <h3>{$t.transcript.nothingToCheck}</h3>
            <p>
              {#if unclearCount}
                {$t.transcript.noneMisheardEveryTime(unclearCount)}
              {:else}
                {$t.transcript.nothingFlaggedNote}
              {/if}
            </p>
          {/if}

          {#if applied}<p class="correction-applied" role="status">{applied}</p>{/if}
        </div>
        {#if protocolStyle}
          <div class="inspector-section">
            <p class="eyebrow">{$t.transcript.protocolStyle}</p>
            <h3>{protocolStyle.name}</h3>
            <p>{protocolStyle.description}</p>
          </div>
        {/if}
      </aside>
    {/if}
  </div>
</main>
