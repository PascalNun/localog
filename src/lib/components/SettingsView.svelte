<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import type {
    FakeJobOutcome,
    ProtocolProviderStatus,
    TranscriptionRuntimeStatus,
  } from '../workflow/types';
  import Icon from './Icon.svelte';

  export let theme: 'light' | 'dark';
  export let nextJobOutcome: FakeJobOutcome;
  export let onToggleTheme: () => void;
  export let onSetNextJobOutcome: (outcome: FakeJobOutcome) => Promise<void>;
  export let runtimeStatus: TranscriptionRuntimeStatus;
  export let providerStatus: ProtocolProviderStatus;
  export let onConfigureRuntime: (executablePath: string, modelPath: string) => Promise<void>;
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
  let modelPath = '';
  let selectedProviderModel = '';
  $: executablePath = runtimeStatus?.executablePath ?? executablePath;
  $: modelPath = runtimeStatus?.modelPath ?? modelPath;
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

  async function chooseModel() {
    if (!('__TAURI_INTERNALS__' in window)) return;
    const selected = await open({
      multiple: false,
      directory: false,
      title: 'Choose whisper.cpp model',
    });
    if (typeof selected === 'string') modelPath = selected;
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
          <select><option>English</option></select>
        </div>
        <div class="setting-row">
          <div>
            <h3>Default export</h3>
            <p>The format offered first in the protocol editor.</p>
          </div>
          <select><option>Markdown</option><option>Plain text</option></select>
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
            <h3>whisper.cpp runtime</h3>
            <p>
              Choose an already installed whisper.cpp executable and model. LocaLog never downloads
              models.
            </p>
          </div>
          <span class:setting-value={runtimeStatus?.executableFound} class="setting-value">
            {runtimeStatus?.executableFound ? 'Ready' : 'Not configured'}
          </span>
        </div>
        <label class="setting-field"
          >Executable path<input
            bind:value={executablePath}
            placeholder="/path/to/whisper-cli"
          /><button class="quiet-action" onclick={chooseExecutable}>Choose file</button></label
        >
        <label class="setting-field"
          >Model path<input bind:value={modelPath} placeholder="/path/to/ggml-model.bin" /><button
            class="quiet-action"
            onclick={chooseModel}>Choose file</button
          ></label
        >
        <button
          class="secondary-action"
          onclick={() => onConfigureRuntime(executablePath, modelPath)}
          disabled={!executablePath || !modelPath}>Save local runtime</button
        >
        {#if runtimeStatus?.runtimeVersion}<p class="setting-hint">
            Detected: {runtimeStatus.runtimeVersion}
          </p>{/if}
        <div class="setting-row">
          <div>
            <h3>Default quality</h3>
            <p>Human-readable preset; exact model mapping remains advanced.</p>
          </div>
          <select><option>Balanced</option><option>Fast</option><option>Accurate</option></select>
        </div>
      {:else if section === 'Storage'}
        <div class="setting-row">
          <div>
            <h3>Working storage</h3>
            <p>App-managed for v0.1, with explicit exports and a documented location.</p>
          </div>
          <span class="setting-value">Application data</span>
        </div>
        <div class="notice-inline">No real files are stored by this Phase 0 fake workflow.</div>
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
      {:else}
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
