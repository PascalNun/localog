<script lang="ts">
  import type { NewProjectInput } from '../workflow/types';
  import { COMMON_MEETING_LANGUAGES, DETECT_LANGUAGE_LABEL } from '../workflow/languages';
  import Icon from './Icon.svelte';
  import { errorMessage } from '../errors';

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
      <p class="eyebrow">Projects</p>
      <h1 tabindex="-1">New project</h1>
      <p>Create the professional context that meetings and sources belong to.</p>
    </div>
  </header>
  <form class="editorial-form" onsubmit={submit}>
    <label
      ><span>Project name</span><input
        bind:value={name}
        placeholder="e.g. Community hall study"
        required
      /></label
    >
    <label
      ><span>Description <em>optional</em></span><textarea
        bind:value={description}
        rows="3"
        placeholder="A concise internal description"></textarea></label
    >
    <label
      ><span>Default meeting language</span><input
        bind:value={defaultLanguage}
        list="project-languages"
        placeholder={DETECT_LANGUAGE_LABEL}
      /><datalist id="project-languages">
        {#each COMMON_MEETING_LANGUAGES as language (language)}<option value={language}
          ></option>{/each}
      </datalist><small>Independent from the application interface language.</small></label
    >
    <details class="advanced-disclosure">
      <summary>Project defaults</summary>
      <p>
        A protocol style, and the names and terms this work uses, can be set for the project after
        it is created. The names are worth a minute: they are what transcription cannot guess.
      </p>
    </details>
    {#if submitError}<p class="form-error" role="alert">{submitError}</p>{/if}
    <footer class="form-actions">
      <button type="button" class="secondary-action" onclick={onCancel}>Cancel</button><button
        class="primary-action"
        type="submit"
        disabled={!name.trim() || submitting}
        >{submitting ? 'Creating…' : returnToImport ? 'Create and continue' : 'Create project'}
        <Icon name="arrow" /></button
      >
    </footer>
  </form>
</main>
