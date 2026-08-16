<script lang="ts">
  import type {
    NewMeetingInput,
    ProjectSummary,
    ProtocolStyle,
    SourceSelection,
  } from '../workflow/types';
  import { COMMON_MEETING_LANGUAGES, DETECT_LANGUAGE_LABEL } from '../workflow/languages';
  import Icon from './Icon.svelte';

  export let projects: ProjectSummary[];
  export let initialProjectId: string | null;
  export let styles: ProtocolStyle[];
  export let onCancel: () => void;
  export let onCreateProject: () => void;
  export let onSelectNativeSource: (() => Promise<SourceSelection | null>) | undefined = undefined;
  export let onCreate: (input: NewMeetingInput) => Promise<void>;
  /// Drops are handled once, by the application, because the commonest moment to
  /// drop a recording is before this step exists — on the start screen, with
  /// nothing open yet.
  export let draggingFile = false;
  export let droppedRecording: string | null = null;
  export let droppedRefusal = '';

  let projectId = initialProjectId ?? projects[0]?.id ?? '';
  let title = '';
  let occurredAt = new Date().toISOString().slice(0, 10);
  let sourceName = '';
  let sourcePath: string | null = null;
  let language = projects.find((project) => project.id === projectId)?.defaultLanguage ?? '';
  let styleId =
    projects.find((project) => project.id === projectId)?.defaultStyleId ?? styles[0]?.id ?? '';
  let submitting = false;
  let submitError = '';
  $: draggingOver = draggingFile;
  $: dropError = droppedRefusal;
  $: if (droppedRecording && droppedRecording !== sourcePath) {
    sourcePath = droppedRecording;
    sourceName = droppedRecording.split('/').pop() ?? droppedRecording;
    if (!title) title = titleFromFile(sourceName);
  }

  $: selectedProject = projects.find((project) => project.id === projectId);

  function useProjectDefaults() {
    if (!selectedProject) return;
    language = selectedProject.defaultLanguage;
    styleId = selectedProject.defaultStyleId;
  }

  function selectFile(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    sourceName = input.files?.[0]?.name ?? '';
    sourcePath = null;
    if (!title && sourceName) title = titleFromFile(sourceName);
  }

  function useFixture() {
    sourceName = 'synthetic-design-coordination.wav';
    if (!title) title = 'Design coordination';
  }

  async function chooseNativeSource() {
    if (!onSelectNativeSource) return;
    submitError = '';
    try {
      const selection = await onSelectNativeSource();
      if (!selection) return;
      sourceName = selection.name;
      sourcePath = selection.path;
      if (!title) title = titleFromFile(sourceName);
    } catch (error) {
      submitError = error instanceof Error ? error.message : String(error);
    }
  }

  function titleFromFile(filename: string) {
    return filename
      .replace(/\.[^.]+$/, '')
      .replace(/[-_]+/g, ' ')
      .replace(/^\w/, (letter) => letter.toUpperCase());
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!projectId || !sourceName || submitting) return;
    submitting = true;
    submitError = '';
    try {
      await onCreate({ projectId, title, occurredAt, language, sourceName, sourcePath, styleId });
    } catch (error) {
      submitError = error instanceof Error ? error.message : String(error);
      submitting = false;
    }
  }
</script>

<main class="workspace" id="main-content">
  <header class="workspace-header compact-header">
    <div>
      <p class="eyebrow">Structured import</p>
      <h1 tabindex="-1">New meeting</h1>
      <p>Choose the recording, confirm the details, and LocaLog takes it from there.</p>
    </div>
  </header>

  <form class="meeting-form" onsubmit={submit}>
    <section class="form-step" aria-labelledby="context-heading">
      <div class="step-number">1</div>
      <div class="step-content">
        <p class="eyebrow">Context</p>
        <h2 id="context-heading">Choose a project</h2>
        <div class="field-with-action">
          <label
            ><span>Project</span><select bind:value={projectId} onchange={useProjectDefaults}
              >{#each projects as project (project.id)}<option value={project.id}
                  >{project.name}</option
                >{/each}</select
            ></label
          >
          <button type="button" class="secondary-action" onclick={onCreateProject}
            ><Icon name="plus" size={15} /> New project</button
          >
        </div>
        <p class="field-note">
          Every source belongs to a meeting, and every meeting belongs to a project. There is no
          inbox.
        </p>
      </div>
    </section>

    <section class="form-step" aria-labelledby="source-heading">
      <div class="step-number">2</div>
      <div class="step-content">
        <div class="section-heading-row">
          <div>
            <p class="eyebrow">Source</p>
            <h2 id="source-heading">Import recording</h2>
          </div>
          <span class="privacy-note">Your original stays where it is</span>
        </div>
        {#if onSelectNativeSource}<button
            class:has-source={sourceName}
            class:dragging={draggingOver}
            class="drop-zone"
            type="button"
            onclick={chooseNativeSource}
          >
            <span class="drop-icon"><Icon name="upload" size={30} /></span>
            {#if sourceName}<strong>{sourceName}</strong><small
                >Ready to copy after you confirm this meeting</small
              >{:else if draggingOver}<strong>Let go to import</strong><small
                >The original stays where it is.</small
              >{:else}<strong>Drop a recording here, or click to choose one</strong><small
                >MP3, M4A, WAV, MP4, MOV and others. The original remains untouched — LocaLog copies
                it into its own storage.</small
              >{/if}
          </button>{:else}<label class:has-source={sourceName} class="drop-zone">
            <input type="file" accept="audio/*,video/*" onchange={selectFile} />
            <span class="drop-icon"><Icon name="upload" size={30} /></span>
            {#if sourceName}<strong>{sourceName}</strong><small
                >Ready to assign to this meeting</small
              >{:else}<strong>Choose an audio or video file</strong><small
                >The browser preview demonstrates the workflow without storing the file.</small
              >{/if}
          </label>
        {/if}
        {#if dropError}<p class="drop-error" role="alert">{dropError}</p>{/if}
        {#if !onSelectNativeSource}<button
            class="text-action demo-fixture-action"
            type="button"
            onclick={useFixture}>Use the synthetic demo recording</button
          >{/if}
      </div>
    </section>

    <section class="form-step" aria-labelledby="details-heading">
      <div class="step-number">3</div>
      <div class="step-content">
        <p class="eyebrow">Essentials</p>
        <h2 id="details-heading">Meeting information</h2>
        <div class="field-grid">
          <label
            ><span>Title</span><input
              bind:value={title}
              placeholder="Derived from the file if left empty"
            /></label
          >
          <label><span>Date</span><input type="date" bind:value={occurredAt} required /></label>
          <label
            ><span>Meeting language</span><input
              bind:value={language}
              list="meeting-languages"
              placeholder={DETECT_LANGUAGE_LABEL}
            /><datalist id="meeting-languages">
              {#each COMMON_MEETING_LANGUAGES as language (language)}<option value={language}
                ></option>{/each}
            </datalist><small
              >{selectedProject?.defaultLanguage === language
                ? 'Project default'
                : 'Meeting override'}</small
            ></label
          >
          <label
            ><span>Protocol style</span><select bind:value={styleId}
              >{#each styles as style (style.id)}<option value={style.id}>{style.name}</option
                >{/each}</select
            ><small
              >{selectedProject?.defaultStyleId === styleId
                ? 'Project default'
                : 'Meeting override'}</small
            ></label
          >
          <p class="field-note">
            Transcription quality is chosen once in Settings and applies to every meeting.
            Per-meeting overrides and choosing names & terms per meeting are not available yet.
          </p>
        </div>
        <details class="advanced-disclosure">
          <summary>Advanced processing options</summary>
          <p>
            Exact runtime and model controls remain intentionally absent until the focused
            transcription and provider spikes validate them.
          </p>
        </details>
      </div>
    </section>

    {#if submitError}<p class="form-error" role="alert">{submitError}</p>{/if}
    <footer class="form-actions">
      <button class="secondary-action" type="button" onclick={onCancel}>Cancel</button><button
        class="primary-action"
        type="submit"
        disabled={!projectId || !sourceName || submitting}
        >{submitting ? 'Bringing the recording in…' : 'Create meeting and import'}
        <Icon name="arrow" /></button
      >
    </footer>
  </form>
</main>
