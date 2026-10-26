<script lang="ts">
  import type { ProjectDetailsInput, ProjectSummary, ProtocolStyle } from '../workflow/types';
  import {
    COMMON_MEETING_LANGUAGES,
    meetingLanguageField,
    meetingLanguageValue,
  } from '../workflow/languages';
  import { errorMessage } from '../errors';
  import { t } from '../i18n';

  export let project: ProjectSummary;
  export let styles: ProtocolStyle[];
  export let onSave: (input: ProjectDetailsInput) => Promise<void>;
  export let onCancel: () => void;

  let name = project.name;
  let description = project.description;
  // The field holds the language named in the reader's language; what is stored is
  // the identifier the transcription runtime is handed. The same translation the
  // creation form uses, so opening this and saving cannot turn English into
  // "English" as a stored value.
  let languageText = meetingLanguageField(project.defaultLanguage);
  let styleId = styles.some((style) => style.id === project.defaultStyleId)
    ? project.defaultStyleId
    : (styles[0]?.id ?? '');
  let saving = false;
  let saveError = '';

  $: languageListId = `project-language-${project.id}`;

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!name.trim() || saving) return;
    saving = true;
    saveError = '';
    try {
      await onSave({
        name,
        description,
        defaultLanguage: meetingLanguageValue(languageText),
        defaultStyleId: styleId,
      });
    } catch (cause) {
      saveError = errorMessage(cause);
      saving = false;
    }
  }
</script>

<section class="project-details" aria-labelledby={`project-details-heading-${project.id}`}>
  <div class="section-heading-row">
    <h2 id={`project-details-heading-${project.id}`}>{$t.project.detailsHeading}</h2>
  </div>
  <p class="setting-hint">{$t.project.detailsLead}</p>
  <form class="editorial-form project-details-form" onsubmit={submit}>
    <label
      ><span>{$t.project.projectNameLabel}</span><input
        bind:value={name}
        placeholder={$t.newProject.namePlaceholder}
        required
      /></label
    >
    <label
      ><span>{$t.project.descriptionLabel} <em>{$t.project.descriptionOptional}</em></span><textarea
        bind:value={description}
        rows="3"
        placeholder={$t.newProject.descriptionPlaceholder}></textarea></label
    >
    <label
      ><span>{$t.project.defaultLanguageLabel}</span><input
        bind:value={languageText}
        list={languageListId}
        placeholder={$t.dialog.detectFromRecording}
      /><datalist id={languageListId}>
        {#each COMMON_MEETING_LANGUAGES as language (language)}<option
            value={$t.meetingLanguages[language]}
          ></option>{/each}
      </datalist><small>{$t.project.languageHint}</small></label
    >
    <label
      ><span>{$t.project.defaultStyleLabel}</span><select bind:value={styleId}>
        {#each styles as style (style.id)}<option value={style.id}>{style.name}</option>{/each}
      </select><small>{$t.project.defaultStyleHint}</small></label
    >
    {#if saveError}<p class="form-error" role="alert">{saveError}</p>{/if}
    <footer class="form-actions">
      <button type="button" class="secondary-action" onclick={onCancel}>{$t.project.cancel}</button
      ><button class="primary-action" type="submit" disabled={!name.trim() || saving}
        >{saving ? $t.project.saving : $t.project.save}</button
      >
    </footer>
  </form>
</section>

<style>
  .project-details-form {
    max-width: 620px;
  }
</style>
