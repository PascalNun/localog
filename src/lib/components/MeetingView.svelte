<script lang="ts">
  import type {
    ActiveJob,
    AppRoute,
    MeetingSummary,
    ProjectSummary,
    SpeakerRequest,
    SpeakerSeparationStatus,
  } from '../workflow/types';
  import { COMMON_MEETING_LANGUAGES, meetingLanguageLabel } from '../workflow/languages';
  import Icon from './Icon.svelte';
  import ProgressPanel from './ProgressPanel.svelte';
  import StageRail from './StageRail.svelte';

  export let project: ProjectSummary;
  export let meeting: MeetingSummary;
  /** Resolved from the real transcription setting; never a fixed label. */
  export let presetLabel: string = 'Not selected';
  export let job: ActiveJob | null;
  export let onNavigate: (route: AppRoute) => void;
  export let onTranscribe: (speakers: SpeakerRequest) => Promise<void>;
  export let onCancel: () => Promise<void>;
  export let onRetry: () => Promise<void>;
  export let onConfirmDuplicate: () => Promise<void>;
  export let onReselectSource: () => Promise<void>;
  export let onRename: (title: string) => Promise<void>;
  export let onUpdateLanguage: (language: string) => Promise<void>;
  export let speakerStatus: SpeakerSeparationStatus = {
    modelsInstalled: false,
    runtimeConfigured: false,
    runtimeHealthy: false,
    runtimeVersion: null,
    runtimePath: null,
    downloadBytes: 0,
  };
  export let speakerPreparing = false;
  export let speakerDownloadPercent = 0;
  export let onPrepareSpeakerModels: () => Promise<void>;

  let editingTitle = false;
  let titleDraft = meeting.title;
  let startingTranscription = false;
  let transcriptionStartError = '';
  // A choice for this transcription run, not a global application setting.
  // '' leaves the speakers together, 'estimate' asks LocaLog to work out how many
  // there were, and a number says how many spoke.
  let speakerChoice = '';
  let editingLanguage = false;
  let languageDraft = meeting.language;
  let languageError = '';
  $: relevantJob = job?.meetingId === meeting.id ? job : null;
  $: transcriptionUnavailable = Boolean(relevantJob && relevantJob.state !== 'completed');
  $: speakerNeedsPreparation = Boolean(speakerChoice && !speakerStatus.modelsInstalled);

  async function startTranscription() {
    startingTranscription = true;
    transcriptionStartError = '';
    try {
      await onTranscribe(
        speakerChoice === ''
          ? 'together'
          : speakerChoice === 'estimate'
            ? 'estimate'
            : Number(speakerChoice),
      );
    } catch {
      transcriptionStartError = 'Transcription could not be started. Please try again.';
    } finally {
      startingTranscription = false;
    }
  }

  async function saveTitle() {
    await onRename(titleDraft);
    editingTitle = false;
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

  function formatBytes(bytes: number | null) {
    if (bytes === null) return 'In your workspace';
    if (bytes < 1_000_000) return `${Math.round(bytes / 1_000)} KB`;
    return `${(bytes / 1_000_000).toFixed(bytes >= 10_000_000 ? 0 : 1)} MB`;
  }
</script>

<main class="workspace" id="main-content">
  <header class="workspace-header meeting-header">
    <div>
      <p class="breadcrumb">{project.name} <span>›</span> Meeting</p>
      <div class="editable-title">
        {#if editingTitle}<input
            bind:value={titleDraft}
            aria-label="Meeting title"
            onkeydown={(event) => event.key === 'Enter' && saveTitle()}
          /> <button class="text-action" onclick={saveTitle}>Save</button>{:else}<h1 tabindex="-1">
            {meeting.title}
          </h1>
          <button
            class="icon-button compact"
            aria-label="Edit meeting title"
            onclick={() => (editingTitle = true)}>✎</button
          >{/if}
      </div>
      <p class="meeting-language-line">
        {meeting.occurredAt} ·
        {#if editingLanguage}<input
            bind:value={languageDraft}
            list="meeting-view-languages"
            aria-label="Meeting language"
          /><datalist id="meeting-view-languages">
            {#each COMMON_MEETING_LANGUAGES as language (language)}<option value={language}
              ></option>{/each}
          </datalist><button class="text-action" onclick={saveLanguage}>Save language</button
          ><button class="text-action" onclick={() => (editingLanguage = false)}>Cancel</button>
        {:else}<button
            class="inline-setting"
            disabled={transcriptionUnavailable}
            onclick={() => {
              languageDraft = meeting.language;
              editingLanguage = true;
            }}
            aria-label="Change meeting language">{meetingLanguageLabel(meeting.language)}</button
          >{/if} · {meeting.durationLabel ?? 'Duration pending'}
      </p>
      {#if languageError}<p class="form-error" role="alert">{languageError}</p>{/if}
    </div>
  </header>

  <StageRail meetingId={meeting.id} lifecycle={meeting.lifecycle} {onNavigate} />
  {#if relevantJob && relevantJob.state !== 'completed'}<ProgressPanel
      job={relevantJob}
      {onCancel}
      {onRetry}
      {onConfirmDuplicate}
      {onReselectSource}
    />{/if}

  <section class="meeting-stage">
    {#if meeting.lifecycle === 'draft'}
      <div class="stage-message">
        <p class="eyebrow">Source import</p>
        <h2>Your original remains unchanged</h2>
        <p>
          {#if relevantJob?.state === 'interrupted'}LocaLog was closed before the managed copy was
            committed. The meeting remains in Draft and the import can be retried safely.{:else if relevantJob?.state === 'cancelled'}The
            managed copy was cancelled. The meeting remains in Draft and the external file was not
            modified.{:else if relevantJob?.state === 'failed'}The managed copy could not be
            committed. The meeting remains in Draft and the external file was not modified.{:else}LocaLog
            is copying this source into private managed storage. It will become ready only after the
            copy has been validated and committed.{/if}
        </p>
      </div>
    {:else if meeting.lifecycle === 'source_ready'}
      <div class="stage-message">
        <p class="eyebrow">Source ready</p>
        <h2>Ready to transcribe</h2>
        <p>
          {#if meeting.sourceByteCount !== null}<strong>{meeting.sourceName}</strong> is safely stored
            with this meeting. The external original was not modified.{:else}<strong
              >{meeting.sourceName}</strong
            > is assigned to this synthetic browser meeting. No real media file was copied.{/if}
        </p>
        <dl class="resolved-settings">
          <div>
            <dt>Managed source</dt>
            <dd>
              {meeting.sourceByteCount === null
                ? 'Synthetic fixture'
                : formatBytes(meeting.sourceByteCount)}<small
                >{meeting.sourceMediaType ?? 'Browser preview'}</small
              >
            </dd>
          </div>
          <div>
            <dt>Language</dt>
            <dd>
              {meetingLanguageLabel(meeting.language)}<small
                >Meeting setting · change above before transcribing</small
              >
            </dd>
          </div>
          <div>
            <dt>Preset</dt>
            <dd>{presetLabel}<small>Global default</small></dd>
          </div>
        </dl>
        <label class="transcription-option">
          <span>People speaking</span>
          <select bind:value={speakerChoice}>
            <option value="">Do not separate speakers</option>
            <option value="estimate">Separate them, and work out how many</option>
            {#each Array.from({ length: 29 }, (_, index) => index + 2) as count (count)}
              <option value={count}>{count} people</option>
            {/each}
          </select>
          <small
            >{speakerChoice === 'estimate'
              ? 'LocaLog groups the voices it hears and counts them. An estimate, and one you can replace with a number if it reads wrong.'
              : speakerChoice
                ? 'Your best estimate is enough — it is the number of voices LocaLog looks for. Too many can split one person in two, too few can put two people together.'
                : 'The transcript keeps one speaker label.'}</small
          >
        </label>
        {#if speakerNeedsPreparation}
          <div class="speaker-preparation" role="status">
            <div>
              <strong>Prepare speaker separation</strong>
              <p>
                LocaLog needs two verified local model files before it can add provisional speaker
                labels. Your recording stays on this device.
              </p>
            </div>
            <button
              class="secondary-action"
              onclick={onPrepareSpeakerModels}
              disabled={speakerPreparing}
              >{speakerPreparing
                ? `Preparing ${speakerDownloadPercent}%`
                : `Prepare${speakerStatus.downloadBytes > 0 ? ` (${formatBytes(speakerStatus.downloadBytes)})` : ''}`}</button
            >
          </div>
        {:else if speakerChoice && !speakerStatus.runtimeHealthy}
          <p class="setting-hint speaker-runtime-note">
            The speaker runtime is not available in this installation. Transcription can continue,
            but the transcript will use editable generic speaker labels.
          </p>
        {/if}
        <button
          class="primary-action"
          onclick={startTranscription}
          disabled={startingTranscription || transcriptionUnavailable || speakerNeedsPreparation}
          >{startingTranscription
            ? 'Getting ready to transcribe…'
            : transcriptionUnavailable
              ? 'Use the job controls above'
              : speakerNeedsPreparation
                ? 'Prepare speaker separation first'
                : 'Transcribe'}
          <Icon name="arrow" /></button
        >
        {#if transcriptionStartError}<p class="form-error" role="alert">
            {transcriptionStartError}
          </p>{/if}
        <p class="setting-hint">
          <button
            class="text-action"
            onclick={() => onNavigate({ name: 'recording-review', meetingId: meeting.id })}
            >Review and trim the recording first</button
          > — cut the wait before the meeting starts and anything it does not need. Your recording is
          never changed.
        </p>
      </div>
    {:else if meeting.lifecycle === 'transcript_ready'}
      <div class="stage-message">
        <p class="eyebrow">Transcript ready</p>
        <h2>Review before generation</h2>
        <p>The timestamped transcript is ready for corrections and manual speaker mapping.</p>
        <button
          class="primary-action"
          onclick={() => onNavigate({ name: 'transcript', meetingId: meeting.id })}
          >Review transcript <Icon name="arrow" /></button
        >
      </div>
    {:else}
      <div class="stage-message">
        <p class="eyebrow">Protocol available</p>
        <h2>Continue in the document editor</h2>
        <p>The transcript remains available alongside the current protocol revision.</p>
        <button
          class="primary-action"
          onclick={() => onNavigate({ name: 'protocol', meetingId: meeting.id })}
          >Open protocol <Icon name="arrow" /></button
        >
      </div>
    {/if}
  </section>
</main>
