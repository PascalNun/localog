<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import type {
    ArchivedWork,
    BackupManifest,
    RestoreOutcome,
    FakeJobOutcome,
    ProtocolProviderStatus,
    TranscriptionCapability,
    TranscriptionPreset,
    TranscriptionRuntimeStatus,
    SpeakerSeparationStatus,
  } from '../workflow/types';
  import { PRESET_LABELS, SPEAKER_SEPARATION_UNREADY } from '../workflow/types';
  import { errorMessage } from '../errors';
  import {
    GENERATION_MODEL_CATALOG,
    browserMemoryGb,
    installedProviderModel,
    modelStatusLabel,
    recommendationFor,
  } from '../models/modelCatalog';
  import { formatModelSize } from '../models/modelSize';
  import Icon from './Icon.svelte';
  import type { IconName } from './Icon.svelte';

  export let theme: 'light' | 'dark';
  export let themeChoice: 'auto' | 'light' | 'dark' = 'auto';
  export let nextJobOutcome: FakeJobOutcome;
  export let onChooseTheme: (choice: 'auto' | 'light' | 'dark') => void;

  /// The three states the theme can be in, named. Automatic is first because it is
  /// what the application does unless somebody says otherwise.
  const THEME_CHOICES: { id: 'auto' | 'light' | 'dark'; label: string; icon: IconName }[] = [
    { id: 'auto', label: 'Automatic', icon: 'monitor' },
    { id: 'light', label: 'Light', icon: 'sun' },
    { id: 'dark', label: 'Dark', icon: 'moon' },
  ];
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
  export let onArchivedWork: () => Promise<ArchivedWork>;
  export let onUnarchiveProject: (projectId: string) => Promise<void>;
  export let onUnarchiveMeeting: (meetingId: string) => Promise<void>;
  export let onCreateBackup: (parent: string, folderName: string) => Promise<BackupManifest>;
  export let onInspectBackup: (folder: string) => Promise<BackupManifest>;
  export let onRestoreBackup: (folder: string) => Promise<RestoreOutcome>;
  export let onRefreshProvider: () => Promise<void>;
  export let onConfigureProvider: (model: string | null) => Promise<void>;
  export let providerError: string | null = null;
  export let speakerStatus: SpeakerSeparationStatus = SPEAKER_SEPARATION_UNREADY;
  export let speakerError: string | null = null;
  export let onRefreshSpeaker: () => Promise<void>;
  export let onDownloadSpeaker: () => Promise<void>;

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
  // The synthetic-failure control only affects the browser preview's fake adapter.
  const isNative = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
  let selectedProviderModel = '';
  /// What the backend measured, and only then what the browser guessed.
  ///
  /// The browser is asked last because in this shell it cannot answer:
  /// `navigator.deviceMemory` is not implemented in WebKit, so every macOS machine
  /// reported nothing, nothing was treated as the weakest supported machine, and a
  /// 16 GB laptop was recommended a model measured at 20 figures against the
  /// baseline's 31.
  $: memoryGb = providerStatus?.machineMemoryGb ?? browserMemoryGb();
  const memoryLabel = memoryGb
    ? `${memoryGb} GB memory reported`
    : 'Using the conservative 8 GB baseline';

  // Product language first: the user picks an outcome, not a model.

  $: executablePath = runtimeStatus?.executablePath ?? executablePath;
  $: selectedProviderModel = providerStatus?.selectedModel ?? selectedProviderModel;
  $: modelRecommendation = recommendationFor(providerStatus?.models ?? [], memoryGb);
  $: recommendedInstalled = modelRecommendation.installed;
  $: uncataloguedModels = (providerStatus?.models ?? []).filter(
    (model) => !GENERATION_MODEL_CATALOG.some((entry) => entry.providerNames.includes(model.name)),
  );

  async function chooseProviderModel(model: string) {
    selectedProviderModel = model;
    await onConfigureProvider(model);
  }

  /// What has been put away, and the way back.
  ///
  /// Read when somebody opens the disclosure rather than with the rest of
  /// settings: archiving exists so a workspace can hold years of work without
  /// showing all of it, and loading all of it to render a heading nobody expanded
  /// would undo the point.
  let archivedOpen = false;
  let archived: ArchivedWork = { projects: [], meetings: [] };
  let archivedError = '';

  async function loadArchived() {
    archivedError = '';
    try {
      archived = await onArchivedWork();
    } catch (cause) {
      archivedError = errorMessage(cause);
    }
  }

  async function toggleArchived() {
    archivedOpen = !archivedOpen;
    if (archivedOpen) await loadArchived();
  }

  async function unarchiveProject(projectId: string) {
    try {
      await onUnarchiveProject(projectId);
      await loadArchived();
    } catch (cause) {
      archivedError = errorMessage(cause);
    }
  }

  async function unarchiveMeeting(meetingId: string) {
    try {
      await onUnarchiveMeeting(meetingId);
      await loadArchived();
    } catch (cause) {
      archivedError = errorMessage(cause);
    }
  }

  /// Backing up, and putting one back.
  ///
  /// The application says the work stays on this device, which also means it
  /// leaves with the device. This is where somebody does something about that.
  ///
  /// Restoring is deliberately two steps. The first reads what a folder claims to
  /// be and shows it; only then is there a button that replaces the workspace.
  /// Nobody should be able to replace a year of minutes with one click on a
  /// folder they picked by accident.
  let backupBusy = false;
  let backupNote = '';
  let backupError = '';
  let pendingRestore: { folder: string; manifest: BackupManifest } | null = null;

  /// A name somebody can read in a file manager a year from now, and which sorts.
  function backupFolderName(): string {
    const now = new Date();
    const day = [
      now.getFullYear(),
      String(now.getMonth() + 1).padStart(2, '0'),
      String(now.getDate()).padStart(2, '0'),
    ].join('-');
    const time = [
      String(now.getHours()).padStart(2, '0'),
      String(now.getMinutes()).padStart(2, '0'),
    ].join('');
    return `LocaLog backup ${day} ${time}`;
  }

  async function backUpNow() {
    if (!('__TAURI_INTERNALS__' in window)) return;
    const parent = await open({
      directory: true,
      multiple: false,
      title: 'Where to keep the backup',
    });
    if (typeof parent !== 'string') return;
    backupBusy = true;
    backupError = '';
    backupNote = '';
    try {
      const manifest = await onCreateBackup(parent, backupFolderName());
      backupNote = `Backed up ${manifest.projectCount} ${
        manifest.projectCount === 1 ? 'project' : 'projects'
      } and ${manifest.meetingCount} ${
        manifest.meetingCount === 1 ? 'meeting' : 'meetings'
      } to ${manifest.folderName}.`;
    } catch (cause) {
      backupError = errorMessage(cause);
    } finally {
      backupBusy = false;
    }
  }

  async function chooseBackupToRestore() {
    if (!('__TAURI_INTERNALS__' in window)) return;
    const folder = await open({
      directory: true,
      multiple: false,
      title: 'Choose a LocaLog backup',
    });
    if (typeof folder !== 'string') return;
    backupBusy = true;
    backupError = '';
    backupNote = '';
    try {
      pendingRestore = { folder, manifest: await onInspectBackup(folder) };
    } catch (cause) {
      backupError = errorMessage(cause);
    } finally {
      backupBusy = false;
    }
  }

  async function confirmRestore() {
    if (!pendingRestore) return;
    backupBusy = true;
    backupError = '';
    try {
      const outcome = await onRestoreBackup(pendingRestore.folder);
      pendingRestore = null;
      backupNote =
        `Restored ${outcome.projectCount} projects and ${outcome.meetingCount} meetings. ` +
        `What was here was moved to ${outcome.previousWorkspace} rather than deleted. ` +
        `Quit and open LocaLog again to work with the restored workspace.`;
    } catch (cause) {
      backupError = errorMessage(cause);
    } finally {
      backupBusy = false;
    }
  }

  async function chooseExecutable() {
    if (!('__TAURI_INTERNALS__' in window)) return;
    const selected = await open({
      multiple: false,
      directory: false,
      title: 'Choose whisper-cli executable',
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
        <div class="model-setting-intro">
          <p class="eyebrow">Default for protocols</p>
          <h3>Choose once, then keep working</h3>
          <p>
            LocaLog uses this model for local protocol drafts until you change it. The normal
            workflow does not ask you to choose a model for every meeting.
          </p>
        </div>
        <article class="model-recommendation">
          <div>
            <p class="model-kicker">Recommended for this machine</p>
            <h3>{modelRecommendation.entry.name}</h3>
            <p>{modelRecommendation.entry.description}</p>
            <div class="model-meta">
              <span>{modelRecommendation.entry.originLabel}</span>
              <span>{modelRecommendation.entry.licenseLabel}</span>
              <span>{memoryLabel}</span>
            </div>
          </div>
          <div class="model-recommendation-action">
            {#if recommendedInstalled}
              <button
                class="secondary-action"
                onclick={() => chooseProviderModel(recommendedInstalled.name)}
                disabled={!providerStatus.serverReachable ||
                  selectedProviderModel === recommendedInstalled.name}
              >
                {selectedProviderModel === recommendedInstalled.name
                  ? 'Selected'
                  : 'Use this model'}
              </button>
            {:else}
              <span class="model-status">Not installed yet</span>
            {/if}
          </div>
        </article>

        <div class="model-catalog" aria-label="Curated protocol models">
          {#each GENERATION_MODEL_CATALOG as entry (entry.id)}
            {@const installed = installedProviderModel(entry, providerStatus.models)}
            {@const selected = installed !== null && selectedProviderModel === installed.name}
            <article class="model-card" class:active={selected}>
              <div class="model-card-copy">
                <div class="model-card-heading">
                  <h3>{entry.name}</h3>
                  {#if entry.status === 'baseline'}<span class="model-badge">Baseline</span>{/if}
                  {#if entry.originLabel === 'European model'}<span class="model-badge quiet"
                      >European</span
                    >{/if}
                </div>
                <p>{entry.description}</p>
                <div class="model-meta">
                  <span>{entry.sizeLabel}</span>
                  <span>{entry.licenseLabel}</span>
                  <span>{entry.languages.slice(0, 3).join(' · ')}</span>
                </div>
                <p class="model-evaluation">
                  {entry.testedLanguages.length
                    ? `Evaluated in ${entry.testedLanguages.join(' and ')}`
                    : 'Quality evaluation still pending'}
                </p>
              </div>
              <div class="model-card-action">
                <span class="model-status">{modelStatusLabel(entry, installed)}</span>
                {#if installed}
                  <button
                    class="quiet-action"
                    onclick={() => chooseProviderModel(installed.name)}
                    disabled={!providerStatus.serverReachable || selected}
                    >{selected ? 'Selected' : 'Use model'}</button
                  >
                {/if}
              </div>
            </article>
          {/each}
        </div>
        {#if providerError}<p class="setting-error" role="alert">{providerError}</p>{/if}
        <p class="setting-hint">{providerStatus.message}</p>
        <div class="setting-actions">
          <button class="secondary-action" onclick={onRefreshProvider}
            >Check installed models</button
          >
        </div>
        <details class="advanced-disclosure model-advanced">
          <summary>Use another installed model</summary>
          <p>
            This is for people who already know which local model they want to try. It is not
            evaluated or recommended by LocaLog, and it remains subject to the same local runtime
            and memory limits.
          </p>
          <div class="setting-field">
            <label for="other-ollama-model">Installed model</label>
            <select
              id="other-ollama-model"
              bind:value={selectedProviderModel}
              disabled={!providerStatus.serverReachable || uncataloguedModels.length === 0}
            >
              <option value="">Choose an installed model</option>
              {#each uncataloguedModels as model (model.name)}
                <option value={model.name}>{model.name}</option>
              {/each}
            </select>
          </div>
          <button
            class="secondary-action"
            onclick={() => onConfigureProvider(selectedProviderModel || null)}
            disabled={!providerStatus.serverReachable || !selectedProviderModel}
            >Use installed model</button
          >
        </details>
        <div class="notice-inline">
          The catalogue is intentionally curated. LocaLog does not silently download models or
          present an arbitrary model marketplace. New entries become selectable only after their
          runtime, licence, memory use and German/English quality have been checked.
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
                <span class="preset-name">{PRESET_LABELS[preset.preset].name}</span>
                <span class="preset-detail">
                  {PRESET_LABELS[preset.preset].detail}
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
                    aria-label="Downloading {PRESET_LABELS[preset.preset].name}"
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
                    >Download ({formatModelSize(preset.byteCount)})</button
                  >
                {/if}
              </div>
            </div>
          {/each}
        </div>
        {#if modelError}<p class="setting-error" role="alert">{modelError}</p>{/if}
        <details class="advanced-disclosure">
          <summary>Advanced details</summary>
          <div class="advanced-block">
            <p class="setting-hint">
              Models are stored in LocaLog’s application data folder and verified before use.
            </p>
            <label class="setting-field"
              >whisper-cli executable<input
                bind:value={executablePath}
                placeholder="/path/to/whisper-cli"
              /><button class="quiet-action" onclick={chooseExecutable}>Choose file</button></label
            >
            <p class="setting-hint">
              Choose the command-line transcription binary, not whisper-server.
            </p>
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
        </details>
        <div class="speaker-setting">
          <div class="setting-row">
            <div>
              <h3>Speaker differentiation</h3>
              <p>
                Optional voice-turn separation labels who spoke when. It never blocks a transcript,
                and labels remain editable during review.
              </p>
            </div>
            {#if speakerStatus.modelsInstalled && speakerStatus.runtimeHealthy}
              <span class="safe-note"><Icon name="check" size={15} /> Ready</span>
            {:else if speakerStatus.modelsInstalled}
              <span class="setting-value">Runtime not available in this installation</span>
            {:else}<span class="setting-value">Optional</span>{/if}
          </div>
          <div class="speaker-controls">
            {#if downloading['speaker-separation'] !== undefined}
              <span class="preset-progress-text">{downloading['speaker-separation']}%</span>
              <div
                class="preset-progress"
                role="progressbar"
                aria-valuenow={downloading['speaker-separation']}
                aria-valuemin="0"
                aria-valuemax="100"
                aria-label="Downloading speaker separation models"
              >
                <span style="width:{downloading['speaker-separation']}%"></span>
              </div>
            {:else if !speakerStatus.modelsInstalled}
              <button class="secondary-action" onclick={onDownloadSpeaker}>
                Prepare speaker separation
                {#if speakerStatus.downloadBytes > 0}({formatModelSize(
                    speakerStatus.downloadBytes,
                  )}){/if}
              </button>
            {:else if speakerStatus.runtimeConfigured}<button
                class="quiet-action"
                onclick={onRefreshSpeaker}>Check readiness</button
              >{:else if speakerStatus.modelsInstalled}<span
                class="setting-hint speaker-runtime-note"
                >The models are prepared, but this installation has no compatible speaker runtime.</span
              >{/if}
          </div>
          <details class="advanced-disclosure">
            <summary>Advanced details</summary>
            <p>
              LocaLog discovers the speaker runtime automatically from its bundled resources or the
              system path. The runtime is optional and never blocks transcription.
            </p>
            {#if speakerStatus.runtimePath}<p class="setting-hint">
                Discovered runtime: {speakerStatus.runtimePath}
              </p>{:else}<p class="setting-hint">
                No compatible speaker runtime was found on this machine yet.
              </p>{/if}
            {#if speakerStatus.runtimeVersion}<p class="setting-hint">
                Runtime version: {speakerStatus.runtimeVersion}
              </p>{/if}
            <p class="setting-hint">
              Readiness includes a bounded launch check, so an incompatible or broken executable is
              not presented as available.
            </p>
            {#if speakerError}<p class="setting-error" role="alert">{speakerError}</p>{/if}
          </details>
        </div>
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
        <div class="setting-row">
          <div>
            <h3>Backup</h3>
            <p>
              Everything stays on this device, which also means it leaves with the device. A backup
              is an ordinary folder you can put on a drive or wherever you keep things safe.
            </p>
          </div>
          <button class="secondary-action" onclick={backUpNow} disabled={backupBusy}>
            {backupBusy ? 'Working…' : 'Back up now'}
          </button>
        </div>
        <p class="setting-hint">
          Holds every project, meeting, transcript and protocol, and the recordings themselves. Two
          things are left out on purpose, because neither is your work and both are rebuilt when
          they are needed: downloaded models, and the prepared copy of each recording. Measured on a
          real workspace, that prepared audio alone was three quarters of the backup.
        </p>
        <div class="setting-row">
          <div>
            <h3>Restore</h3>
            <p>
              Puts a backup back. It is checked in full first, and what is here now is moved aside
              rather than deleted.
            </p>
          </div>
          <button class="quiet-action" onclick={chooseBackupToRestore} disabled={backupBusy}>
            Choose a backup…
          </button>
        </div>
        {#if pendingRestore}
          <!-- Shown before the button that replaces a workspace, never after. -->
          <div class="restore-confirm">
            <p>
              <strong>{pendingRestore.manifest.folderName}</strong> holds
              {pendingRestore.manifest.projectCount} projects and
              {pendingRestore.manifest.meetingCount} meetings, backed up from LocaLog
              {pendingRestore.manifest.applicationVersion}.
            </p>
            <p class="setting-hint">
              Restoring replaces the projects and meetings in this workspace with those. Nothing is
              deleted — what is here is kept in a folder beside it — but LocaLog will be showing the
              restored work, and you will need to quit and open it again.
            </p>
            <div class="restore-actions">
              <button class="secondary-action" onclick={confirmRestore} disabled={backupBusy}>
                {backupBusy ? 'Restoring…' : 'Replace this workspace'}
              </button>
              <button class="text-action" onclick={() => (pendingRestore = null)}>Cancel</button>
            </div>
          </div>
        {/if}
        <div class="setting-row">
          <div>
            <h3>Archived</h3>
            <p>
              Projects and meetings put out of the way. Nothing was deleted: every meeting,
              transcript and protocol under them is still here, and still in every backup.
            </p>
          </div>
          <button class="quiet-action" aria-expanded={archivedOpen} onclick={toggleArchived}>
            {archivedOpen ? 'Hide' : 'Show'}
          </button>
        </div>
        {#if archivedOpen}
          {#if archived.projects.length === 0 && archived.meetings.length === 0}
            <p class="setting-hint">Nothing has been archived.</p>
          {:else}
            <ul class="archived-list">
              {#each archived.projects as project (project.id)}
                <li>
                  <span><strong>{project.name}</strong><small>Project</small></span>
                  <button class="text-action" onclick={() => void unarchiveProject(project.id)}>
                    Bring back
                  </button>
                </li>
              {/each}
              {#each archived.meetings as meeting (meeting.id)}
                <li>
                  <span
                    ><strong>{meeting.title}</strong><small>Meeting · {meeting.occurredAt}</small
                    ></span
                  >
                  <button class="text-action" onclick={() => void unarchiveMeeting(meeting.id)}>
                    Bring back
                  </button>
                </li>
              {/each}
            </ul>
          {/if}
          {#if archivedError}<p class="setting-error" role="alert">{archivedError}</p>{/if}
        {/if}
        {#if backupNote}<p class="safe-note" role="status">{backupNote}</p>{/if}
        {#if backupError}<p class="setting-error" role="alert">{backupError}</p>{/if}
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
            <p>
              {themeChoice === 'auto'
                ? `Following this Mac, which is set to ${theme}.`
                : 'Set here, whatever this Mac is set to.'}
            </p>
          </div>
          <div class="choice-row" role="group" aria-label="Theme">
            {#each THEME_CHOICES as option (option.id)}
              <button
                class="choice"
                class:chosen={themeChoice === option.id}
                aria-pressed={themeChoice === option.id}
                onclick={() => onChooseTheme(option.id)}
                ><Icon name={option.icon} size={15} /> {option.label}</button
              >
            {/each}
          </div>
        </div>
        <div class="setting-row">
          <div>
            <h3>Typeface</h3>
            <p>The typeface ships with the application. No font is ever fetched.</p>
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
