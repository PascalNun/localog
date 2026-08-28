<script lang="ts">
  import type {
    ActiveJob,
    AppRoute,
    MeetingSummary,
    ProjectSummary,
    SpeakerRequest,
    SpeakerSeparationStatus,
  } from '../workflow/types';
  import { SPEAKER_SEPARATION_UNREADY } from '../workflow/types';
  import { COMMON_MEETING_LANGUAGES, meetingLanguageLabel } from '../workflow/languages';
  import Icon from './Icon.svelte';
  import ProgressPanel from './ProgressPanel.svelte';
  import StageRail from './StageRail.svelte';
  import { errorMessage } from '../errors';
  import { formatBytes } from '../bytes';
  // Both, and they are not interchangeable: an imported recording's size is decimal
  // because that is what the file manager showing it says, and a model's size is
  // binary because that is how the model files are published. This screen shows one
  // of each, and showed the model in the recording's units until it was noticed that
  // the same two files read 46 MB here and 43 MB in Settings.
  import { formatModelSize } from '../models/modelSize';
  import { t } from '../i18n';

  export let project: ProjectSummary;
  export let meeting: MeetingSummary;
  /** Resolved from the real transcription setting; never a fixed label. */
  export let presetLabel: string = $t.meeting.notSelected;
  export let job: ActiveJob | null;
  export let onNavigate: (route: AppRoute) => void;
  export let onTranscribe: (speakers: SpeakerRequest) => Promise<void>;
  export let onCancel: () => Promise<void>;
  export let onRetry: () => Promise<void>;
  export let onConfirmDuplicate: () => Promise<void>;
  export let onReselectSource: () => Promise<void>;
  export let onRename: (title: string) => Promise<void>;
  export let onUpdateLanguage: (language: string) => Promise<void>;
  export let speakerStatus: SpeakerSeparationStatus = SPEAKER_SEPARATION_UNREADY;
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
      transcriptionStartError = $t.meeting.transcriptionFailedToStart;
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
      languageError = errorMessage(error);
    }
  }
</script>

<main class="workspace stage-workspace" id="main-content">
  <header class="workspace-header meeting-header">
    <div>
      <p class="breadcrumb">{project.name} <span>›</span> {$t.shell.breadcrumbMeeting}</p>
      <div class="editable-title">
        {#if editingTitle}<input
            bind:value={titleDraft}
            aria-label={$t.meeting.titleLabel}
            onkeydown={(event) => event.key === 'Enter' && saveTitle()}
          /> <button class="text-action" onclick={saveTitle}>{$t.meeting.save}</button>{:else}<h1
            tabindex="-1"
          >
            {meeting.title}
          </h1>
          <button
            class="icon-button compact"
            aria-label={$t.meeting.editTitle}
            onclick={() => (editingTitle = true)}>✎</button
          >{/if}
      </div>
      <p class="meeting-language-line">
        {meeting.occurredAt} ·
        {#if editingLanguage}<input
            bind:value={languageDraft}
            list="meeting-view-languages"
            aria-label={$t.meeting.languageLabel}
          /><datalist id="meeting-view-languages">
            {#each COMMON_MEETING_LANGUAGES as language (language)}<option value={language}
              ></option>{/each}
          </datalist><button class="text-action" onclick={saveLanguage}
            >{$t.meeting.saveLanguage}</button
          ><button class="text-action" onclick={() => (editingLanguage = false)}
            >{$t.meeting.cancel}</button
          >
        {:else}<button
            class="inline-setting"
            disabled={transcriptionUnavailable}
            onclick={() => {
              languageDraft = meeting.language;
              editingLanguage = true;
            }}
            aria-label={$t.meeting.changeLanguage}>{meetingLanguageLabel(meeting.language)}</button
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
    {#if meeting.lifecycle === 'draft' && !meeting.sourceName}
      <!-- A meeting created to be recorded rather than imported: no file was chosen,
           so there is nothing being copied and the next step is the recorder. -->
      <div class="stage-message">
        <p class="eyebrow">{$t.meeting.recordingEyebrow}</p>
        <h2>{$t.meeting.nothingRecorded}</h2>
        <p>
          {$t.meeting.recordLead}
        </p>
        <button
          class="primary-action"
          onclick={() => onNavigate({ name: 'recording', meetingId: meeting.id })}
        >
          {$t.meeting.recordThisMeeting}
        </button>
      </div>
    {:else if meeting.lifecycle === 'draft'}
      <div class="stage-message">
        <p class="eyebrow">{$t.meeting.sourceImport}</p>
        <h2>{$t.meeting.originalUnchanged}</h2>
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
        <p class="eyebrow">{$t.meeting.sourceReady}</p>
        <h2>{$t.meeting.readyToTranscribe}</h2>
        <p>
          {#if meeting.sourceByteCount !== null}<strong>{meeting.sourceName}</strong>
            {$t.meeting.sourceStored}{:else}<strong>{meeting.sourceName}</strong>
            {$t.meeting.sourceSynthetic}{/if}
        </p>
        <dl class="resolved-settings">
          <div>
            <dt>{$t.meeting.managedSource}</dt>
            <dd>
              {meeting.sourceByteCount === null
                ? $t.meeting.syntheticFixture
                : formatBytes(meeting.sourceByteCount)}<small
                >{meeting.sourceMediaType ?? 'Browser preview'}</small
              >
            </dd>
          </div>
          <div>
            <dt>{$t.meeting.language}</dt>
            <dd>
              {meetingLanguageLabel(meeting.language)}<small>{$t.meeting.languageHint}</small>
            </dd>
          </div>
          <div>
            <dt>{$t.meeting.preset}</dt>
            <dd>{presetLabel}<small>{$t.meeting.globalDefault}</small></dd>
          </div>
        </dl>
        <label class="transcription-option">
          <span>{$t.meeting.peopleSpeaking}</span>
          <select bind:value={speakerChoice}>
            <option value="">{$t.meeting.doNotSeparate}</option>
            <option value="estimate">{$t.meeting.separateAndCount}</option>
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
              <strong>{$t.meeting.prepareSpeakers}</strong>
              <p>
                {$t.meeting.prepareSpeakersDetail}
              </p>
            </div>
            <button
              class="secondary-action"
              onclick={onPrepareSpeakerModels}
              disabled={speakerPreparing}
              >{speakerPreparing
                ? $t.meeting.preparing(speakerDownloadPercent)
                : speakerStatus.downloadBytes > 0
                  ? $t.meeting.prepareWithSize(formatModelSize(speakerStatus.downloadBytes))
                  : $t.meeting.prepare}</button
            >
          </div>
        {:else if speakerChoice && !speakerStatus.runtimeHealthy}
          <p class="setting-hint speaker-runtime-note">
            {$t.meeting.speakerRuntimeMissing}
          </p>
        {/if}
        <button
          class="primary-action"
          onclick={startTranscription}
          disabled={startingTranscription || transcriptionUnavailable || speakerNeedsPreparation}
          >{startingTranscription
            ? $t.meeting.gettingReady
            : transcriptionUnavailable
              ? $t.meeting.useJobControls
              : speakerNeedsPreparation
                ? $t.meeting.prepareSpeakersFirst
                : $t.meeting.transcribe}
          <Icon name="arrow" /></button
        >
        {#if transcriptionStartError}<p class="form-error" role="alert">
            {transcriptionStartError}
          </p>{/if}
        <p class="setting-hint">
          <button
            class="text-action"
            onclick={() => onNavigate({ name: 'recording-review', meetingId: meeting.id })}
            >{$t.meeting.reviewAndTrim}</button
          >
          {$t.meeting.trimDetail}
        </p>
      </div>
    {:else if meeting.lifecycle === 'transcript_ready'}
      <div class="stage-message">
        <p class="eyebrow">{$t.meeting.transcriptReady}</p>
        <h2>{$t.meeting.reviewBeforeGeneration}</h2>
        <p>{$t.meeting.transcriptReadyDetail}</p>
        <button
          class="primary-action"
          onclick={() => onNavigate({ name: 'transcript', meetingId: meeting.id })}
          >{$t.meeting.reviewTranscript} <Icon name="arrow" /></button
        >
      </div>
    {:else}
      <div class="stage-message">
        <p class="eyebrow">{$t.meeting.protocolAvailable}</p>
        <h2>{$t.meeting.continueInEditor}</h2>
        <p>{$t.meeting.protocolDetail}</p>
        <button
          class="primary-action"
          onclick={() => onNavigate({ name: 'protocol', meetingId: meeting.id })}
          >{$t.meeting.openProtocol} <Icon name="arrow" /></button
        >
      </div>
    {/if}
  </section>
</main>
