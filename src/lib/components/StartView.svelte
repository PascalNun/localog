<script lang="ts">
  import type { AppRoute, TranscriptionCapability } from '../workflow/types';
  import { formatModelSize } from '../models/modelSize';
  import { t } from '../i18n';
  import Icon from './Icon.svelte';

  export let onNavigate: (route: AppRoute) => void;
  export let capability: TranscriptionCapability;
  export let downloading: Record<string, number>;
  export let modelError: string | null;
  export let onDownloadModel: (modelId: string) => Promise<void>;
  export let onCancelDownload: (modelId: string) => Promise<void>;

  /// The one thing a new installation is missing, said before it is hit.
  ///
  /// LocaLog transcribes on this device, so the model has to be on this device.
  /// Reaching transcription without one is already handled and handled well —
  /// `missing_runtime_message` names only what is absent and gives one next
  /// action — but it says so *after* somebody has chosen a project, chosen a
  /// file, and waited for it to be probed. Everything needed to say it at the
  /// start was already there and nothing said it: readiness is computed per
  /// preset, the download works, and the start screen mentioned neither.
  ///
  /// So this is not a repair of a bad error. It moves a good one earlier, and
  /// offers the download rather than directions to it. It is deliberately not a
  /// gate: creating a project and importing a recording both work without a
  /// model, and somebody who would rather get on with it should be able to.
  $: chosen = capability.presets.find((preset) => preset.preset === capability.selectedPreset);
  // Any installed model means the application can transcribe. The chosen one is
  // what the download offers, but a person who fetched a different quality in
  // Settings is ready and should not be told otherwise.
  $: ready = capability.presets.some((preset) => preset.installed);
  $: percent = chosen ? downloading[chosen.modelId] : undefined;
  $: fetching = percent !== undefined;
</script>

<main class="workspace start-workspace" id="main-content">
  <div class="start-hero">
    <svg class="sound-mark" viewBox="0 0 96 72" aria-hidden="true">
      <line x1="8" y1="27" x2="8" y2="45" />
      <line x1="21" y1="17" x2="21" y2="56" />
      <line x1="34" y1="5" x2="34" y2="67" />
      <line x1="48" y1="17" x2="48" y2="56" />
      <line x1="62" y1="25" x2="62" y2="48" />
      <line x1="75" y1="14" x2="75" y2="59" />
      <line x1="88" y1="27" x2="88" y2="45" />
    </svg>
    <p class="eyebrow">{$t.start.eyebrow}</p>
    <h1 tabindex="-1">{$t.start.title}</h1>
    <p class="hero-copy">{$t.start.lead}</p>

    {#if !ready && chosen}
      <div class="start-setup" aria-live="polite">
        <p class="start-setup-title">{$t.start.setupTitle}</p>
        <p class="start-setup-copy">
          {$t.start.setupBody(
            $t.settings.transcriptionPreset[capability.selectedPreset].name,
            formatModelSize(chosen.byteCount),
          )}
        </p>
        {#if fetching}
          <div class="start-setup-progress">
            <div class="start-setup-track">
              <div class="start-setup-fill" style={`width: ${percent}%`}></div>
            </div>
            <span class="start-setup-percent">{percent}%</span>
            <!-- 141 MB started from here should be stoppable from here. -->
            <button class="start-setup-cancel" onclick={() => onCancelDownload(chosen.modelId)}>
              {$t.start.setupCancel}
            </button>
          </div>
        {:else}
          <button class="start-setup-action" onclick={() => onDownloadModel(chosen.modelId)}>
            {$t.start.setupDownload(formatModelSize(chosen.byteCount))}
          </button>
        {/if}
        {#if modelError}
          <p class="start-setup-error">{modelError}</p>
        {/if}
        <p class="start-setup-aside">{$t.start.setupAside}</p>
      </div>
    {/if}

    <button
      class="import-hero-action"
      onclick={() => onNavigate({ name: 'new-meeting', projectId: null })}
    >
      <span class="import-icon"><Icon name="upload" size={26} /></span>
      <span><strong>{$t.start.importTitle}</strong><small>{$t.start.importDetail}</small></span>
      <Icon name="arrow" />
    </button>

    <button
      class="import-hero-action is-secondary"
      onclick={() => onNavigate({ name: 'new-meeting', projectId: null, forRecording: true })}
    >
      <span class="import-icon"><Icon name="microphone" size={26} /></span>
      <span><strong>{$t.start.recordTitle}</strong><small>{$t.start.recordDetail}</small></span>
      <Icon name="arrow" />
    </button>
  </div>

  <div class="local-promise">
    <span class="lock-mark" aria-hidden="true">⌂</span>
    <span><strong>{$t.start.promiseTitle}</strong><small>{$t.start.promiseDetail}</small></span>
  </div>
</main>
