<script lang="ts">
  import type { NewProjectInput } from '../workflow/types';
  import { COMMON_MEETING_LANGUAGES, DETECT_LANGUAGE_LABEL } from '../workflow/languages';
  import Icon from './Icon.svelte';
  import { errorMessage } from '../errors';
  import { t } from '../i18n';

  export let returnToImport: boolean;
  export let onCancel: () => void;
  export let onCreate: (input: NewProjectInput) => Promise<void>;

  let name = '';
  let description = '';
  // Unset, not English. A default language is a claim about meetings that have
  // not happened yet, and the wrong claim is expensive: it transcribed a German
  // recording in English and was only discovered eleven minutes later.
  let defaultLanguage = '';
  let submitting = false;
  let submitError = '';

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!name.trim() || submitting) return;
    submitting = true;
    submitError = '';
    try {
      await onCreate({ name, description, defaultLanguage });
    } catch (error) {
      submitError = errorMessage(error);
      submitting = false;
    }
  }
</script>

<main class="workspace narrow-workspace" id="main-content">
  <header class="workspace-header compact-header">
    <div>
      <p class="eyebrow">{$t.newProject.eyebrow}</p>
      <h1 tabindex="-1">{$t.newProject.title}</h1>
      <p>{$t.newProject.lead}</p>
    </div>
  </header>
  <form class="editorial-form" onsubmit={submit}>
    <label
      ><span>{$t.newProject.name}</span><input
        bind:value={name}
        placeholder={$t.newProject.namePlaceholder}
        required
      /></label
    >
    <label
      ><span>{$t.newProject.description} <em>{$t.newProject.descriptionOptional}</em></span
      ><textarea
        bind:value={description}
        rows="3"
        placeholder={$t.newProject.descriptionPlaceholder}></textarea></label
    >
    <label
      ><span>{$t.newProject.defaultLanguage}</span><input
        bind:value={defaultLanguage}
        list="project-languages"
        placeholder={DETECT_LANGUAGE_LABEL}
      /><datalist id="project-languages">
        {#each COMMON_MEETING_LANGUAGES as language (language)}<option value={language}
          ></option>{/each}
      </datalist><small>{$t.newProject.defaultLanguageDetail}</small></label
    >
    <details class="advanced-disclosure">
      <summary>{$t.newProject.defaults}</summary>
      <p>
        {$t.newProject.afterCreated}
      </p>
    </details>
    {#if submitError}<p class="form-error" role="alert">{submitError}</p>{/if}
    <footer class="form-actions">
      <button type="button" class="secondary-action" onclick={onCancel}
        >{$t.newProject.cancel}</button
      ><button class="primary-action" type="submit" disabled={!name.trim() || submitting}
        >{submitting
          ? $t.newProject.creating
          : returnToImport
            ? $t.newProject.createAndContinue
            : $t.sidebar.createProject}
        <Icon name="arrow" /></button
      >
    </footer>
  </form>
</main>
