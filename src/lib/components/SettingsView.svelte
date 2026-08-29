<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import type {
    ArchivedWork,
    ExportFormat,
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
  import { INTERFACE_LANGUAGES, chooseLanguage, language, t } from '../i18n';
  import { formatMeetingDate } from '../protocol/document';
  import Icon from './Icon.svelte';
  import type { IconName } from './Icon.svelte';

  export let theme: 'light' | 'dark';
  export let themeChoice: 'auto' | 'light' | 'dark' = 'auto';
  export let nextJobOutcome: FakeJobOutcome;
  export let onChooseTheme: (choice: 'auto' | 'light' | 'dark') => void;

  /// The three states the theme can be in, named. Automatic is first because it is
  /// what the application does unless somebody says otherwise.
  /// Reactive, not a constant. A constant reads the words once when the module
  /// loads and goes on saying them after somebody changes the language — the same
  /// fault the lifecycle labels and the header band names had.
  $: THEME_CHOICES = [
    { id: 'auto' as const, label: $t.settings.themeAutomatic, icon: 'monitor' as IconName },
    { id: 'light' as const, label: $t.settings.themeLight, icon: 'sun' as IconName },
    { id: 'dark' as const, label: $t.settings.themeDark, icon: 'moon' as IconName },
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
  export let defaultExport: ExportFormat;
  export let onChooseDefaultExport: (format: ExportFormat) => void;
  export let workspacePath: string | null;
  export let onRevealWorkspace: () => Promise<void>;
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

  /// The formats the protocol editor already exports, named as somebody would
  /// say them rather than as the code spells them.
  let EXPORT_CHOICES: { id: ExportFormat; label: string }[];
  $: EXPORT_CHOICES = [
    { id: 'pdf', label: $t.settings.formatPdf },
    { id: 'docx', label: $t.settings.formatWord },
    { id: 'markdown', label: $t.settings.formatMarkdown },
    { id: 'text', label: $t.settings.formatPlainText },
  ];

  let section = 'General';
  /// Privacy is gone from here, and Advanced only exists where it has anything in
  /// it.
  ///
  /// Privacy held two rows that were never settings: "LocaLog does not include
  /// analytics" and "content stays out of ordinary logs" are claims about the
  /// product, and a claim wearing a value on the right of a settings row reads as
  /// a switch somebody cannot move. The claims belong to the product and are made
  /// on the start screen, where the first thing anybody reads is that their work
  /// stays on this device.
  ///
  /// Advanced was worse: it was listed here and had no branch of its own, so the
  /// final `{:else}` caught it — which meant a development-only control in the
  /// browser and, in the built application, a tab that opened an empty page.
  // The synthetic-failure control only affects the browser preview's fake adapter.
  const isNative = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
  /// The id is what the branches below compare against and never changes; the
  /// label is what somebody reads. They were one string until there was a second
  /// language, at which point translating it would have silently switched off
  /// every section — `section === 'General'` is never true once the tab says
  /// Allgemein.
  $: sections = [
    { id: 'General', label: $t.settings.sectionGeneral },
    { id: 'Models', label: $t.settings.sectionModels },
    { id: 'Transcription', label: $t.settings.sectionTranscription },
    { id: 'Storage', label: $t.settings.sectionStorage },
    { id: 'Appearance', label: $t.settings.sectionAppearance },
    ...(isNative ? [] : [{ id: 'Advanced', label: $t.settings.sectionAdvanced }]),
  ];
  $: sectionLabel = sections.find((each) => each.id === section)?.label ?? section;
  let executablePath = '';
  let selectedProviderModel = '';
  /// What the backend measured, and only then what the browser guessed.
  ///
  /// The browser is asked last because in this shell it cannot answer:
  /// `navigator.deviceMemory` is not implemented in WebKit, so every macOS machine
  /// reported nothing, nothing was treated as the weakest supported machine, and a
  /// 16 GB laptop was recommended a model measured at 20 figures against the
  /// baseline's 31.
  $: memoryGb = providerStatus?.machineMemoryGb ?? browserMemoryGb();
  /// Reactive, and it was a `const` reading a `$:` value — which is assigned after
  /// the initialisers run, so this captured `undefined` and the line always claimed
  /// the conservative baseline even on a machine that had reported its memory.
  $: memoryLabel = memoryGb
    ? $t.settings.memoryReported(memoryGb)
    : $t.settings.conservativeBaseline;

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
      title: $t.settings.whereToKeepBackup,
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
      title: $t.settings.chooseBackupTitle,
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
      title: $t.settings.chooseWhisper,
    });
    if (typeof selected === 'string') executablePath = selected;
  }
</script>

<main class="workspace" id="main-content">
  <header class="workspace-header compact-header">
    <div>
      <p class="eyebrow">{$t.settings.application}</p>
      <h1 tabindex="-1">{$t.settings.title}</h1>
      <p>{$t.settings.lead}</p>
    </div>
  </header>
  <div class="settings-layout">
    <nav class="settings-nav" aria-label={$t.settings.sectionsLabel}>
      {#each sections as item (item.id)}<button
          class:active={section === item.id}
          onclick={() => (section = item.id)}>{item.label}</button
        >{/each}
    </nav>
    <section class="settings-panel" aria-live="polite">
      <p class="eyebrow">{sectionLabel}</p>
      <h2>{sectionLabel}</h2>
      {#if section === 'General'}
        <!--
          Back, and real. This row said "English" with no way to change it until
          there was a second language to change it to; now there is.

          Separate from the language a meeting is transcribed and written in. A
          German office regularly minutes an English meeting, and the application
          it does that in should still be in German.
        -->
        <div class="setting-row">
          <div>
            <h3>{$t.settings.interfaceLanguage}</h3>
            <p>{$t.settings.interfaceLanguageDetail}</p>
          </div>
          <div class="choice-row" role="group" aria-label={$t.settings.interfaceLanguage}>
            {#each INTERFACE_LANGUAGES as option (option.id)}
              <button
                class="choice"
                class:chosen={$language === option.id}
                aria-pressed={$language === option.id}
                onclick={() => chooseLanguage(option.id)}>{option.label}</button
              >
            {/each}
          </div>
        </div>
        <div class="setting-row">
          <div>
            <h3>{$t.settings.defaultExport}</h3>
            <p>{$t.settings.defaultExportDetail}</p>
          </div>
          <div class="choice-row" role="group" aria-label={$t.settings.defaultExportLabel}>
            {#each EXPORT_CHOICES as option (option.id)}
              <button
                class="choice"
                class:chosen={defaultExport === option.id}
                aria-pressed={defaultExport === option.id}
                onclick={() => onChooseDefaultExport(option.id)}>{option.label}</button
              >
            {/each}
          </div>
        </div>
      {:else if section === 'Models'}
        <div class="model-setting-intro">
          <p class="eyebrow">{$t.settings.defaultForProtocols}</p>
          <h3>{$t.settings.chooseOnce}</h3>
          <p>
            {$t.settings.modelLead}
          </p>
        </div>
        <article class="model-recommendation">
          <div>
            <p class="model-kicker">{$t.settings.recommendedForMachine}</p>
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
                  ? $t.settings.modelSelected
                  : $t.settings.useThisModel}
              </button>
            {:else}
              <span class="model-status">{$t.settings.notInstalledYet}</span>
            {/if}
          </div>
        </article>

        <div class="model-catalog" aria-label={$t.settings.curatedModels}>
          {#each GENERATION_MODEL_CATALOG as entry (entry.id)}
            {@const installed = installedProviderModel(entry, providerStatus.models)}
            {@const selected = installed !== null && selectedProviderModel === installed.name}
            <article class="model-card" class:active={selected}>
              <div class="model-card-copy">
                <div class="model-card-heading">
                  <h3>{entry.name}</h3>
                  {#if entry.status === 'baseline'}<span class="model-badge"
                      >{$t.settings.baseline}</span
                    >{/if}
                  {#if entry.originLabel === 'European model'}<span class="model-badge quiet"
                      >{$t.settings.european}</span
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
                    ? $t.settings.evaluatedIn(entry.testedLanguages.join(' and '))
                    : $t.settings.evaluationPending}
                </p>
              </div>
              <div class="model-card-action">
                <span class="model-status">{modelStatusLabel(entry, installed)}</span>
                {#if installed}
                  <button
                    class="quiet-action"
                    onclick={() => chooseProviderModel(installed.name)}
                    disabled={!providerStatus.serverReachable || selected}
                    >{selected ? $t.settings.modelSelected : $t.settings.useModel}</button
                  >
                {/if}
              </div>
            </article>
          {/each}
        </div>
        {#if providerError}<p class="setting-error" role="alert">{providerError}</p>{/if}
        <!-- Through the funnel, so a code is rendered in the current language while
             a sentence Rust still writes for itself passes through unchanged. Most of
             what lands here is the second kind; see the note in PLAN.md. -->
        <p class="setting-hint">{errorMessage(providerStatus.message)}</p>
        <div class="setting-actions">
          <button class="secondary-action" onclick={onRefreshProvider}
            >{$t.settings.checkInstalled}</button
          >
        </div>
        <details class="advanced-disclosure model-advanced">
          <summary>{$t.settings.useAnotherModel}</summary>
          <p>
            {$t.settings.otherModelNote}
          </p>
          <div class="setting-field">
            <label for="other-ollama-model">{$t.settings.installedModel}</label>
            <select
              id="other-ollama-model"
              bind:value={selectedProviderModel}
              disabled={!providerStatus.serverReachable || uncataloguedModels.length === 0}
            >
              <option value="">{$t.settings.chooseInstalledModel}</option>
              {#each uncataloguedModels as model (model.name)}
                <option value={model.name}>{model.name}</option>
              {/each}
            </select>
          </div>
          <button
            class="secondary-action"
            onclick={() => onConfigureProvider(selectedProviderModel || null)}
            disabled={!providerStatus.serverReachable || !selectedProviderModel}
            >{$t.settings.useInstalledModel}</button
          >
        </details>
        <div class="notice-inline">
          {$t.settings.catalogueNote}
        </div>
      {:else if section === 'Transcription'}
        <div class="setting-row">
          <div>
            <h3>{$t.settings.transcriptionQuality}</h3>
            <p>
              {$t.settings.qualityLead}
            </p>
          </div>
        </div>
        <div class="preset-list" role="radiogroup" aria-label={$t.settings.transcriptionQuality}>
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
                    >{$t.settings.cancel}</button
                  >
                {:else if preset.installed}
                  <span class="safe-note"><Icon name="check" size={15} /> {$t.settings.ready}</span>
                  <button class="quiet-action" onclick={() => onRemoveModel(preset.modelId)}
                    >{$t.settings.remove}</button
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
          <summary>{$t.settings.advancedDetails}</summary>
          <div class="advanced-block">
            <p class="setting-hint">
              {$t.settings.modelsStoredNote}
            </p>
            <label class="setting-field"
              >{$t.settings.whisperExecutable}<input
                bind:value={executablePath}
                placeholder="/path/to/whisper-cli"
              /><button class="quiet-action" onclick={chooseExecutable}
                >{$t.settings.chooseFile}</button
              ></label
            >
            <p class="setting-hint">
              {$t.settings.whisperNote}
            </p>
            <button
              class="secondary-action"
              onclick={() => onConfigureRuntime(executablePath)}
              disabled={!executablePath}>{$t.settings.saveRuntime}</button
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
              <h3>{$t.settings.speakerDifferentiation}</h3>
              <p>
                {$t.settings.speakerLead}
              </p>
            </div>
            {#if speakerStatus.modelsInstalled && speakerStatus.runtimeHealthy}
              <span class="safe-note"><Icon name="check" size={15} /> {$t.settings.ready}</span>
            {:else if speakerStatus.modelsInstalled}
              <span class="setting-value">{$t.settings.runtimeUnavailable}</span>
            {:else}<span class="setting-value">{$t.settings.optional}</span>{/if}
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
                aria-label={$t.settings.downloadingSpeakerModels}
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
                onclick={onRefreshSpeaker}>{$t.settings.checkReadiness}</button
              >{:else if speakerStatus.modelsInstalled}<span
                class="setting-hint speaker-runtime-note">{$t.settings.speakerRuntimeMissing}</span
              >{/if}
          </div>
          <details class="advanced-disclosure">
            <summary>{$t.settings.advancedDetails}</summary>
            <p>
              {$t.settings.speakerDiscovery}
            </p>
            {#if speakerStatus.runtimePath}<p class="setting-hint">
                {$t.settings.discoveredRuntime(speakerStatus.runtimePath)}
              </p>{:else}<p class="setting-hint">
                {$t.settings.noSpeakerRuntime}
              </p>{/if}
            {#if speakerStatus.runtimeVersion}<p class="setting-hint">
                {$t.settings.runtimeVersion(speakerStatus.runtimeVersion)}
              </p>{/if}
            <p class="setting-hint">
              {$t.settings.readinessNote}
            </p>
            {#if speakerError}<p class="setting-error" role="alert">{speakerError}</p>{/if}
          </details>
        </div>
      {:else if section === 'Storage'}
        <!--
          The path, not the category.

          This said "Application data", which answers "is it managed?" for
          somebody asking "where are my files?". Local-first means the files are
          theirs, and nobody believes that about a folder they cannot name or
          reach. The location stays app-managed — a workspace somebody can put in
          a synced folder is a SQLite database in a synced folder, which is a
          known way to lose one — but it is no longer a secret.
        -->
        <div class="setting-row">
          <div>
            <h3>{$t.settings.whereWorkIsKept}</h3>
            <p>
              {$t.settings.workspaceNote}
            </p>
          </div>
          {#if workspacePath}
            <button class="quiet-action" onclick={() => void onRevealWorkspace()}>
              {$t.settings.showInFinder}
            </button>
          {/if}
        </div>
        {#if workspacePath}
          <p class="setting-hint workspace-path">{workspacePath}</p>
        {/if}
        <div class="notice-inline">
          {$t.settings.managedCopiesNote}
        </div>
        <div class="setting-row">
          <div>
            <h3>{$t.settings.backup}</h3>
            <p>
              {$t.settings.backupLead}
            </p>
          </div>
          <button class="secondary-action" onclick={backUpNow} disabled={backupBusy}>
            {backupBusy ? $t.settings.working : $t.settings.backUpNow}
          </button>
        </div>
        <p class="setting-hint">
          {$t.settings.backupContents}
        </p>
        <div class="setting-row">
          <div>
            <h3>{$t.settings.restore}</h3>
            <p>
              {$t.settings.restoreLead}
            </p>
          </div>
          <button class="quiet-action" onclick={chooseBackupToRestore} disabled={backupBusy}>
            {$t.settings.chooseBackup}
          </button>
        </div>
        {#if pendingRestore}
          <!-- Shown before the button that replaces a workspace, never after. -->
          <div class="restore-confirm">
            <p>
              {$t.settings.restoreSummary(
                pendingRestore.manifest.folderName,
                pendingRestore.manifest.projectCount,
                pendingRestore.manifest.meetingCount,
                pendingRestore.manifest.applicationVersion,
              )}
            </p>
            <p class="setting-hint">
              {$t.settings.restoreWarning}
            </p>
            <div class="restore-actions">
              <button class="secondary-action" onclick={confirmRestore} disabled={backupBusy}>
                {backupBusy ? $t.settings.restoring : $t.settings.replaceWorkspace}
              </button>
              <button class="text-action" onclick={() => (pendingRestore = null)}
                >{$t.settings.cancel}</button
              >
            </div>
          </div>
        {/if}
        <div class="setting-row">
          <div>
            <h3>{$t.settings.archived}</h3>
            <p>
              {$t.settings.archivedLead}
            </p>
          </div>
          <button class="quiet-action" aria-expanded={archivedOpen} onclick={toggleArchived}>
            {archivedOpen ? $t.settings.hide : $t.settings.show}
          </button>
        </div>
        {#if archivedOpen}
          {#if archived.projects.length === 0 && archived.meetings.length === 0}
            <p class="setting-hint">{$t.settings.nothingArchived}</p>
          {:else}
            <ul class="archived-list">
              {#each archived.projects as project (project.id)}
                <li>
                  <span><strong>{project.name}</strong><small>{$t.settings.project}</small></span>
                  <button class="text-action" onclick={() => void unarchiveProject(project.id)}>
                    Bring back
                  </button>
                </li>
              {/each}
              {#each archived.meetings as meeting (meeting.id)}
                <li>
                  <span
                    ><strong>{meeting.title}</strong><small
                      >{$t.settings.meeting} · {formatMeetingDate(meeting.occurredAt, $t)}</small
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
      {:else if section === 'Appearance'}
        <div class="setting-row">
          <div>
            <h3>{$t.settings.theme}</h3>
            <p>
              {themeChoice === 'auto'
                ? $t.settings.themeFollowing(theme)
                : $t.settings.themeSetHere}
            </p>
          </div>
          <div class="choice-row" role="group" aria-label={$t.settings.theme}>
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
        <!-- Barlow ships with the application and no font is ever fetched, which is
             a property of the build rather than something to choose. It sat here as a
             row with a value on the right and no way to change it. -->
      {:else if section === 'Advanced'}
        <div class="setting-row">
          <div>
            <h3>{$t.settings.nextFakeJob}</h3>
            <p>{$t.settings.nextFakeJobDetail}</p>
          </div>
          <select
            value={nextJobOutcome}
            onchange={(event) => onSetNextJobOutcome(event.currentTarget.value as FakeJobOutcome)}
            ><option value="success">{$t.settings.completeNormally}</option><option value="failure"
              >{$t.settings.failOnce}</option
            ></select
          >
        </div>
        <div class="notice-inline warning">
          <Icon name="warning" size={16} />
          {$t.settings.syntheticNote}
        </div>
      {/if}
    </section>
  </div>
</main>
