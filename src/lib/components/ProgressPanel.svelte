<script lang="ts">
  import type { ActiveJob } from '../workflow/types';
  import Icon from './Icon.svelte';
  import { formatBytes } from '../bytes';
  import { t } from '../i18n';

  export let job: ActiveJob;
  export let onCancel: () => Promise<void>;
  export let onRetry: () => Promise<void>;
  export let onConfirmDuplicate: () => Promise<void> = async () => undefined;
  export let onReselectSource: () => Promise<void> = async () => undefined;

  $: kindLabel =
    job.kind === 'import'
      ? $t.progress.importing
      : job.kind === 'transcription'
        ? $t.progress.transcribing
        : $t.progress.generating;

  $: byteProgress =
    job.kind === 'import' && job.totalBytes !== null
      ? `${formatBytes(job.progressBytes)} of ${formatBytes(job.totalBytes)}`
      : job.stage.toLowerCase().includes('speaker')
        ? $t.progress.working
        : `${job.progress}%`;
  $: indeterminate = job.stage.toLowerCase().includes('speaker');
  $: stageLabel = indeterminate ? $t.progress.separatingSpeakers : job.stage;
  $: continueLabel =
    job.kind === 'import'
      ? $t.progress.continueImport
      : job.kind === 'transcription'
        ? $t.progress.transcribeAgain
        : $t.progress.generateAgain;
</script>

<section
  class:failed={['failed', 'interrupted'].includes(job.state)}
  class="progress-panel"
  aria-live="polite"
>
  <div class="progress-copy">
    <p class="eyebrow">
      {job.state === 'failed' ? $t.progress.needsAttention : $t.progress.backgroundWork}
    </p>
    <h2>{job.error?.title ?? kindLabel}</h2>
    <p>{job.error?.detail ?? stageLabel}</p>
  </div>
  {#if job.requiresDuplicateConfirmation}
    <div class="progress-actions duplicate-actions">
      <span class="safe-note">{$t.progress.duplicateNote}</span><button
        class="secondary-action"
        onclick={onCancel}>{$t.progress.cancelImport}</button
      ><button class="primary-action" onclick={onConfirmDuplicate}
        >{$t.progress.importAnotherCopy}</button
      >
    </div>
  {:else if ['failed', 'interrupted', 'cancelled'].includes(job.state) || (job.state === 'queued' && job.progressBytes === 0)}
    <div class="progress-actions">
      <span class="safe-note"
        ><Icon name="check" size={15} />
        {$t.progress.latestRetained}{job.kind === 'import'
          ? $t.progress.originalUnchanged
          : ''}</span
      >{#if ['source_missing', 'source_reselection_required'].includes(job.error?.code ?? '')}<button
          class="secondary-action"
          onclick={onReselectSource}>{$t.progress.chooseSourceAgain}</button
        >{/if}{#if job.error?.code !== 'source_reselection_required'}<button
          class="primary-action"
          onclick={onRetry}>{job.state === 'queued' ? continueLabel : $t.progress.retry}</button
        >{/if}
    </div>
  {:else if job.state === 'completed'}
    <div class="progress-actions">
      <span class="safe-note"><Icon name="check" size={15} /> {job.stage}</span>
    </div>
  {:else}
    <div class="progress-meter-wrap">
      <div class="progress-meta"><span>{stageLabel}</span><span>{byteProgress}</span></div>
      {#if indeterminate}
        <p class="progress-subnote">
          {$t.progress.speakerPassNote}
        </p>
      {/if}
      <div
        class:indeterminate
        class="progress-track"
        role="progressbar"
        aria-label={kindLabel}
        aria-valuenow={indeterminate ? undefined : job.progress}
        aria-valuetext={indeterminate ? $t.progress.separatingSpeakers : `${job.progress}%`}
        aria-valuemin="0"
        aria-valuemax="100"
      >
        <span class:indeterminate style={`width: ${indeterminate ? 35 : job.progress}%`}></span>
      </div>
      <button class="secondary-action" onclick={onCancel} disabled={job.state === 'cancelling'}
        >{job.state === 'cancelling' ? $t.progress.cancellingSafely : $t.progress.cancel}</button
      >
    </div>
  {/if}
</section>
