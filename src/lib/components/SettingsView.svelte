<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import type {
    FakeJobOutcome,
    ProtocolProviderStatus,
    TranscriptionCapability,
    TranscriptionPreset,
    TranscriptionRuntimeStatus,
  } from '../workflow/types';
  import Icon from './Icon.svelte';

  export let theme: 'light' | 'dark';
  export let nextJobOutcome: FakeJobOutcome;
  export let onToggleTheme: () => void;
  export let onSetNextJobOutcome: (outcome: FakeJobOutcome) => Promise<void>;
  export let runtimeStatus: TranscriptionRuntimeStatus;
  export let runtimeError: string | null = null;
  export let providerStatus: ProtocolProviderStatus;
  export let capability: TranscriptionCapability;
  export let downloading: Record<string, number> = {};
  export let modelError: string | null = null;
  export let onSelectPreset: (preset: TranscriptionPreset) => Promise<void>;
  export let onDownloadModel: (modelId: string) => Promise<void>;
  export let onCancelDownload: (modelId: string) => Promise<void>;
  export let onRemoveModel: (modelId: string) => Promise<void>;
  export let onConfigureRuntime: (executablePath: string) => Promise<void>;
  export let onRefreshProvider: () => Promise<void>;
  export let onConfigureProvider: (model: string | null) => Promise<void>;

  let section = 'General';
  const sections = [
    'General',
    'Models',
    'Transcription',
    'Storage',
    'Privacy',
    'Appearance',
    'Advanced',
  ];
  let executablePath = '';
  let showAdvanced = false;
  // The synthetic-failure control only affects the browser preview's fake adapter.
  const isNative = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
  let selectedProviderModel = '';

  // Product language first: the user picks an outcome, not a model.
  const presetLabels: Record<TranscriptionPreset, { name: string; detail: string }> = {
    fast: { name: 'Fast', detail: 'Quick drafts, lightest on memory' },
    balanced: { name: 'Balanced', detail: 'Everyday meetings' },
    accurate: { name: 'Accurate', detail: 'Best quality, slowest' },
  };

  function formatSize(bytes: number): string {
    if (bytes >= 1024 ** 3) return `${(bytes / 1024 ** 3).toFixed(1)} GB`;
    return `${Math.round(bytes / 1024 ** 2)} MB`;
  }
  $: executablePath = runtimeStatus?.executablePath ?? executablePath;
  $: selectedProviderModel = providerStatus?.selectedModel ?? selectedProviderModel;

  async function chooseExecutable() {
    if (!('__TAURI_INTERNALS__' in window)) return;
    const selected = await open({
      multiple: false,
      directory: false,
      title: 'Choose whisper.cpp executable',
    });
    if (typeof selected === 'string') executablePath = selected;
  }
</script>

<main class="workspace" id="main-content">
  <header class="workspace-header compact-header">
    <div>
      <p class="eyebrow">Application</p>
      <h1 tabindex="-1">Settings</h1>
      <p>Professional defaults first. Runtime details stay progressively disclosed.</p>
    </div>
  </header>
  <div class="settings-layout">
    <nav class="settings-nav" aria-label="Settings sections">
      {#each sections as item (item)}<button
          class:active={section === item}
          onclick={() => (section = item)}>{item}</button
        >{/each}
    </nav>
    <section class="settings-panel" aria-live="polite">
      <p class="eyebrow">{section}</p>
      <h2>{section}</h2>
      {#if section === 'General'}
        <div class="setting-row">
          <div>
            <h3>Interface language</h3>
            <p>Independent from each meeting’s transcription and protocol language.</p>
          </div>
          <span class="setting-value">English</span>
        </div>
        <div class="setting-row">
          <div>
            <h3>Default export</h3>
            <p>Both formats are offered in the protocol editor.</p>
          </div>
          <span class="setting-value">Markdown and plain text</span>
        </div>
      {:else if section === 'Models'}
        <div class="setting-row">
          <div>
            <h3>Protocol provider</h3>
            <p>Phase 0 discovers installed providers only. No model downloads.</p>
          </div>
          <span class:setting-value={providerStatus.serverReachable} class="setting-value">
            {providerStatus.serverReachable
              ? providerStatus.selectedModelReady
                ? 'Ready'
                : 'Choose a model'
              : 'Not running'}
          </span>
        </div>
        <p class="setting-hint">{providerStatus.message}</p>
        <div class="setting-field">
          <label for="ollama-model">Installed Ollama model</label>
          <select
            id="ollama-model"
            bind:value={selectedProviderModel}
            disabled={!providerStatus.serverReachable}
          >
            <option value="">No model selected</option>
            {#each providerStatus.models as model (model.name)}
              <option value={model.name}>{model.name}</option>
            {/each}
          </select>
        </div>
        <div class="setting-actions">
          <button class="secondary-action" onclick={onRefreshProvider}
            >Refresh local provider</button
          >
          <button
            class="secondary-action"
            onclick={() => onConfigureProvider(selectedProviderModel || null)}
            disabled={!providerStatus.serverReachable}>Save selected model</button
          >
        </div>
        <div class="notice-inline">
          Ollama is a development and technical-preview baseline, not the accepted public
          distribution model. LocaLog never starts Ollama or downloads models.
        </div>
      {:else if section === 'Transcription'}
        <div class="setting-row">
          <div>
            <h3>Transcription quality</h3>
            <p>
              Choose the quality you want. LocaLog downloads what it needs the first time and keeps
              it on this device.
            </p>
          </div>
        </div>
        <div class="preset-list" role="radiogroup" aria-label="Transcription quality">
          {#each capability.presets as preset (preset.preset)}
            {@const active = capability.selectedPreset === preset.preset}
            {@const busy = downloading[preset.modelId] !== undefined}
            <div class="preset-row" class:active>
              <button
                class="preset-choice"
                role="radio"
                aria-checked={active}
                onclick={() => onSelectPreset(preset.preset)}
              >
                <span class="preset-name">{presetLabels[preset.preset].name}</span>
                <span class="preset-detail">
                  {presetLabels[preset.preset].detail}
                  {#if showAdvanced}<span class="preset-model">· {preset.modelId}</span>{/if}
                </span>
              </button>
              <div class="preset-state">
                {#if busy}
                  <span class="preset-progress-text">{downloading[preset.modelId]}%</span>
                  <div
                    class="preset-progress"
                    role="progressbar"
                    aria-valuenow={downloading[preset.modelId]}
                    aria-valuemin="0"
                    aria-valuemax="100"
                    aria-label="Downloading {presetLabels[preset.preset].name}"
                  >
                    <span style="width:{downloading[preset.modelId]}%"></span>
                  </div>
                  <button class="quiet-action" onclick={() => onCancelDownload(preset.modelId)}
                    >Cancel</button
                  >
                {:else if preset.installed}
                  <span class="safe-note"><Icon name="check" size={15} /> Ready</span>
                  <button class="quiet-action" onclick={() => onRemoveModel(preset.modelId)}
                    >Remove</button
                  >
                {:else}
                  <button class="secondary-action" onclick={() => onDownloadModel(preset.modelId)}
                    >Download ({formatSize(preset.byteCount)})</button
                  >
                {/if}
              </div>
            </div>
          {/each}
        </div>
        {#if modelError}<p class="setting-error" role="alert">{modelError}</p>{/if}
        <button class="quiet-action" onclick={() => (showAdvanced = !showAdvanced)}>
          {showAdvanced ? 'Hide advanced details' : 'Show advanced details'}
        </button>
        {#if showAdvanced}
          <div class="advanced-block">
            <p class="setting-hint">
              Models are stored in LocaLog’s application data folder and verified before use.
            </p>
            <label class="setting-field"
              >whisper.cpp executable<input
                bind:value={executablePath}
                placeholder="/path/to/whisper-cli"
              /><button class="quiet-action" onclick={chooseExecutable}>Choose file</button></label
            >
            <button
              class="secondary-action"
              onclick={() => onConfigureRuntime(executablePath)}
              disabled={!executablePath}>Save runtime</button
            >
            {#if runtimeError}<p class="setting-error" role="alert">{runtimeError}</p>{/if}
            {#if runtimeStatus?.runtimeVersion}<p class="setting-hint">
                Detected: {runtimeStatus.runtimeVersion}
              </p>{/if}
          </div>
        {/if}
      {:else if section === 'Storage'}
        <div class="setting-row">
          <div>
            <h3>Working storage</h3>
            <p>App-managed for v0.1, with explicit exports and a documented location.</p>
          </div>
          <span class="setting-value">Application data</span>
        </div>
        <div class="notice-inline">
          LocaLog keeps managed copies of imported recordings, prepared audio, transcripts,
          protocols and downloaded models in its application-data folder. Exports are written only
          to the location you choose.
        </div>
      {:else if section === 'Privacy'}
        <div class="setting-row">
          <div>
            <h3>Telemetry</h3>
            <p>LocaLog does not include analytics, remote crash reporting, or cloud sync.</p>
          </div>
          <span class="safe-note"><Icon name="check" size={15} /> Off</span>
        </div>
        <div class="setting-row">
          <div>
            <h3>Content logging</h3>
            <p>Transcript, protocol, and audio content stays out of ordinary logs.</p>
          </div>
          <span class="safe-note"><Icon name="check" size={15} /> Excluded</span>
        </div>
      {:else if section === 'Appearance'}
        <div class="setting-row">
          <div>
            <h3>Theme</h3>
            <p>Warm cream light mode or designed warm-charcoal dark mode.</p>
          </div>
          <button class="secondary-action" onclick={onToggleTheme}
            ><Icon name={theme === 'light' ? 'moon' : 'sun'} size={16} /> Use {theme === 'light'
              ? 'dark'
              : 'light'}</button
          >
        </div>
        <div class="setting-row">
          <div>
            <h3>Typeface</h3>
            <p>Barlow is bundled locally; no remote font request.</p>
          </div>
          <span class="setting-value">Barlow</span>
        </div>
      {:else if !isNative}
        <div class="setting-row">
          <div>
            <h3>Next fake job</h3>
            <p>Development-only control for reviewing failure and retry states.</p>
          </div>
          <select
            value={nextJobOutcome}
            onchange={(event) => onSetNextJobOutcome(event.currentTarget.value as FakeJobOutcome)}
            ><option value="success">Complete normally</option><option value="failure"
              >Fail once, then allow retry</option
            ></select
          >
        </div>
        <div class="notice-inline warning">
          <Icon name="warning" size={16} /> This affects only the in-memory synthetic runtime.
        </div>
      {/if}
    </section>
  </div>
</main>
