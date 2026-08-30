<script lang="ts">
  import type {
    NewMeetingInput,
    ProjectSummary,
    ProtocolStyle,
    SourceSelection,
  } from '../workflow/types';
  import {
    COMMON_MEETING_LANGUAGES,
    meetingLanguageField,
    meetingLanguageValue,
  } from '../workflow/languages';
  import Icon from './Icon.svelte';
  import { errorMessage } from '../errors';
  import { t } from '../i18n';

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
  /// Somebody who came here from "Record a meeting" has no file and is not going to
  /// choose one. The form asked for one anyway, and its submit guard silently
  /// refused — the button was enabled and pressing it did nothing at all.
  export let forRecording = false;
  export let droppedRecording: string | null = null;
  export let droppedRefusal = '';

  let projectId = initialProjectId ?? projects[0]?.id ?? '';
  let title = '';
  let occurredAt = new Date().toISOString().slice(0, 10);
  let sourceName = '';
  let sourcePath: string | null = null;
  /**
   * What the field shows, which is the language named in the reader's language.
   * `language` below is the identifier it means, and is what everything else uses:
   * the comparison against the project's default was made against the field once,
   * and quietly stopped matching the moment the field held a translated name.
   */
  let languageText = meetingLanguageField(
    projects.find((project) => project.id === projectId)?.defaultLanguage,
  );
  $: language = meetingLanguageValue(languageText);
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
    languageText = meetingLanguageField(selectedProject.defaultLanguage);
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
      submitError = errorMessage(error);
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
    if (!projectId || submitting) return;
    if (!forRecording && !sourceName) return;
    if (forRecording && !title.trim()) return;
    submitting = true;
    submitError = '';
    try {
      await onCreate({ projectId, title, occurredAt, language, sourceName, sourcePath, styleId });
    } catch (error) {
      submitError = errorMessage(error);
      submitting = false;
    }
  }
</script>

<main class="workspace" id="main-content">
  <header class="workspace-header compact-header">
    <div>
      <p class="eyebrow">
        {forRecording ? $t.newMeeting.titleRecording : $t.newMeeting.titleImport}
      </p>
      <h1 tabindex="-1">{$t.newMeeting.heading}</h1>
      <p>
        {forRecording ? $t.newMeeting.leadRecording : $t.newMeeting.leadImport}
      </p>
    </div>
  </header>

  <form class="meeting-form" onsubmit={submit}>
    <section class="form-step" aria-labelledby="context-heading">
      <div class="step-number">1</div>
      <div class="step-content">
        <p class="eyebrow">{$t.newMeeting.context}</p>
        <h2 id="context-heading">{$t.newMeeting.chooseProject}</h2>
        <div class="field-with-action">
          <label
            ><span>{$t.newMeeting.project}</span><select
              bind:value={projectId}
              onchange={useProjectDefaults}
              >{#each projects as project (project.id)}<option value={project.id}
                  >{project.name}</option
                >{/each}</select
            ></label
          >
          <button type="button" class="secondary-action" onclick={onCreateProject}
            ><Icon name="plus" size={15} /> {$t.newMeeting.newProject}</button
          >
        </div>
        <p class="field-note">
          {$t.newMeeting.noInbox}
        </p>
      </div>
    </section>

    {#if !forRecording}
      <section class="form-step" aria-labelledby="source-heading">
        <div class="step-number">2</div>
        <div class="step-content">
          <div class="section-heading-row">
            <div>
              <p class="eyebrow">{$t.newMeeting.source}</p>
              <h2 id="source-heading">{$t.newMeeting.importRecording}</h2>
            </div>
            <span class="privacy-note">{$t.newMeeting.originalStays}</span>
          </div>
          {#if onSelectNativeSource}<button
              class:has-source={sourceName}
              class:dragging={draggingOver}
              class="drop-zone"
              type="button"
              onclick={chooseNativeSource}
            >
              <span class="drop-icon"><Icon name="upload" size={30} /></span>
              {#if sourceName}<strong>{sourceName}</strong><small>{$t.newMeeting.readyToCopy}</small
                >{:else if draggingOver}<strong>{$t.newMeeting.letGoToImport}</strong><small
                  >{$t.newMeeting.originalStaysShort}</small
                >{:else}<strong>{$t.newMeeting.dropHere}</strong><small
                  >{$t.newMeeting.dropDetail}</small
                >{/if}
            </button>{:else}<label class:has-source={sourceName} class="drop-zone">
              <input type="file" accept="audio/*,video/*" onchange={selectFile} />
              <span class="drop-icon"><Icon name="upload" size={30} /></span>
              {#if sourceName}<strong>{sourceName}</strong><small
                  >{$t.newMeeting.readyToAssign}</small
                >{:else}<strong>{$t.newMeeting.chooseFile}</strong><small
                  >{$t.newMeeting.previewNote}</small
                >{/if}
            </label>
          {/if}
          {#if dropError}<p class="drop-error" role="alert">{dropError}</p>{/if}
          {#if !onSelectNativeSource}<button
              class="text-action demo-fixture-action"
              type="button"
              onclick={useFixture}>{$t.newMeeting.useDemoRecording}</button
            >{/if}
        </div>
      </section>
    {/if}

    <section class="form-step" aria-labelledby="details-heading">
      <div class="step-number">3</div>
      <div class="step-content">
        <p class="eyebrow">{$t.newMeeting.essentials}</p>
        <h2 id="details-heading">{$t.newMeeting.meetingInformation}</h2>
        <div class="field-grid">
          <label
            ><span>{$t.newMeeting.title}</span><input
              bind:value={title}
              placeholder={$t.newMeeting.titlePlaceholder}
            /></label
          >
          <label
            ><span>{$t.newMeeting.date}</span><input
              type="date"
              bind:value={occurredAt}
              required
            /></label
          >
          <label
            ><span>{$t.newMeeting.language}</span><input
              bind:value={languageText}
              list="meeting-languages"
              placeholder={$t.dialog.detectFromRecording}
            /><datalist id="meeting-languages">
              {#each COMMON_MEETING_LANGUAGES as choice (choice)}<option
                  value={$t.meetingLanguages[choice]}
                ></option>{/each}
            </datalist><small
              >{selectedProject?.defaultLanguage === language
                ? $t.newMeeting.projectDefault
                : $t.newMeeting.meetingOverride}</small
            ></label
          >
          <label
            ><span>{$t.newMeeting.protocolStyle}</span><select bind:value={styleId}
              >{#each styles as style (style.id)}<option value={style.id}>{style.name}</option
                >{/each}</select
            ><small
              >{selectedProject?.defaultStyleId === styleId
                ? $t.newMeeting.projectDefault
                : $t.newMeeting.meetingOverride}</small
            ></label
          >
          <p class="field-note">
            {$t.newMeeting.qualityNote}
            {$t.newMeeting.noPerMeetingOverrides}
          </p>
        </div>
        <details class="advanced-disclosure">
          <summary>{$t.newMeeting.advanced}</summary>
          <p>
            {$t.newMeeting.chosenOnceNote}
          </p>
        </details>
      </div>
    </section>

    {#if submitError}<p class="form-error" role="alert">{submitError}</p>{/if}
    <footer class="form-actions">
      <button class="secondary-action" type="button" onclick={onCancel}
        >{$t.newMeeting.cancel}</button
      ><button
        class="primary-action"
        type="submit"
        disabled={!projectId || (forRecording ? !title.trim() : !sourceName) || submitting}
        >{submitting
          ? forRecording
            ? $t.newMeeting.preparing
            : $t.newMeeting.bringingRecordingIn
          : forRecording
            ? $t.newMeeting.createAndRecord
            : $t.newMeeting.createAndImport}
        <Icon name="arrow" /></button
      >
    </footer>
  </form>
</main>
