<script lang="ts">
  import type { ActiveJob } from '../workflow/types';
  import Icon from './Icon.svelte';

  export let job: ActiveJob;
  export let onCancel: () => Promise<void>;
  export let onRetry: () => Promise<void>;
  export let onConfirmDuplicate: () => Promise<void> = async () => undefined;
  export let onReselectSource: () => Promise<void> = async () => undefined;

  $: kindLabel =
    job.kind === 'import'
      ? 'Importing recording'
      : job.kind === 'transcription'
        ? 'Transcribing'
        : 'Generating protocol';

  $: byteProgress =
    job.kind === 'import' && job.totalBytes !== null
      ? `${formatBytes(job.progressBytes)} of ${formatBytes(job.totalBytes)}`
      : job.stage.toLowerCase().includes('speaker')
        ? 'Working…'
        : `${job.progress}%`;
  $: indeterminate = job.stage.toLowerCase().includes('speaker');
  $: stageLabel = indeterminate ? 'Separating speakers' : job.stage;
  $: continueLabel =
    job.kind === 'import'
      ? 'Continue import'
      : job.kind === 'transcription'
        ? 'Start transcription again'
        : 'Start generation again';

  function formatBytes(bytes: number) {
    if (bytes < 1_000_000) return `${Math.round(bytes / 1_000)} KB`;
    return `${(bytes / 1_000_000).toFixed(bytes >= 10_000_000 ? 0 : 1)} MB`;
  }
</script>

<section
  class:failed={['failed', 'interrupted'].includes(job.state)}
  class="progress-panel"
  aria-live="polite"
>
  <div class="progress-copy">
    <p class="eyebrow">{job.state === 'failed' ? 'Needs attention' : 'Background work'}</p>
    <h2>{job.error?.title ?? kindLabel}</h2>
    <p>{job.error?.detail ?? stageLabel}</p>
  </div>
  {#if job.requiresDuplicateConfirmation}
    <div class="progress-actions duplicate-actions">
      <span class="safe-note"
        >The same content is already stored in LocaLog. Nothing has been merged or discarded.</span
      ><button class="secondary-action" onclick={onCancel}>Cancel import</button><button
        class="primary-action"
        onclick={onConfirmDuplicate}>Import another copy</button
      >
    </div>
  {:else if ['failed', 'interrupted', 'cancelled'].includes(job.state) || (job.state === 'queued' && job.progressBytes === 0)}
    <div class="progress-actions">
      <span class="safe-note"
        ><Icon name="check" size={15} /> Latest stable work retained{job.kind === 'import'
          ? ' · external original unchanged'
          : ''}</span
      >{#if ['source_missing', 'source_reselection_required'].includes(job.error?.code ?? '')}<button
          class="secondary-action"
          onclick={onReselectSource}>Choose source again</button
        >{/if}{#if job.error?.code !== 'source_reselection_required'}<button
          class="primary-action"
          onclick={onRetry}>{job.state === 'queued' ? continueLabel : 'Retry'}</button
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
          This pass reads the full recording to compare voice turns. Long recordings can take a few
          minutes; you can cancel safely at any time.
        </p>
      {/if}
      <div
        class:indeterminate
        class="progress-track"
        role="progressbar"
        aria-label={kindLabel}
        aria-valuenow={indeterminate ? undefined : job.progress}
        aria-valuetext={indeterminate ? 'Separating speakers' : `${job.progress}%`}
        aria-valuemin="0"
        aria-valuemax="100"
      >
        <span class:indeterminate style={`width: ${indeterminate ? 35 : job.progress}%`}></span>
      </div>
      <button class="secondary-action" onclick={onCancel} disabled={job.state === 'cancelling'}
        >{job.state === 'cancelling' ? 'Cancelling safely…' : 'Cancel'}</button
      >
    </div>
  {/if}
</section>
