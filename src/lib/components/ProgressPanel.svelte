<script lang="ts">
  import type { ActiveJob } from '../workflow/types';
  import Icon from './Icon.svelte';

  export let job: ActiveJob;
  export let onCancel: () => Promise<void>;
  export let onRetry: () => Promise<void>;

  $: kindLabel =
    job.kind === 'import'
      ? 'Importing recording'
      : job.kind === 'transcription'
        ? 'Transcribing locally'
        : 'Generating protocol';
</script>

<section class:failed={job.state === 'failed'} class="progress-panel" aria-live="polite">
  <div class="progress-copy">
    <p class="eyebrow">{job.state === 'failed' ? 'Needs attention' : 'Background work'}</p>
    <h2>{job.error?.title ?? kindLabel}</h2>
    <p>{job.error?.detail ?? job.stage}</p>
  </div>
  {#if job.state === 'failed' || job.state === 'interrupted'}
    <div class="progress-actions">
      <span class="safe-note"><Icon name="check" size={15} /> Latest stable work retained</span
      ><button class="primary-action" onclick={onRetry}>Retry</button>
    </div>
  {:else if job.state === 'completed'}
    <div class="progress-actions">
      <span class="safe-note"><Icon name="check" size={15} /> {job.stage}</span>
    </div>
  {:else}
    <div class="progress-meter-wrap">
      <div class="progress-meta"><span>{job.stage}</span><span>{job.progress}%</span></div>
      <div
        class="progress-track"
        role="progressbar"
        aria-label={kindLabel}
        aria-valuenow={job.progress}
        aria-valuemin="0"
        aria-valuemax="100"
      >
        <span style={`width: ${job.progress}%`}></span>
      </div>
      <button class="secondary-action" onclick={onCancel} disabled={job.state === 'cancelling'}
        >{job.state === 'cancelling' ? 'Cancelling safely…' : 'Cancel'}</button
      >
    </div>
  {/if}
</section>
