<script lang="ts">
  import type { NewProjectInput } from '../workflow/types';
  import Icon from './Icon.svelte';

  export let returnToImport: boolean;
  export let onCancel: () => void;
  export let onCreate: (input: NewProjectInput) => Promise<void>;

  let name = '';
  let description = '';
  let defaultLanguage = 'English';
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
      submitError = error instanceof Error ? error.message : String(error);
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
        placeholder="Any language"
      /><datalist id="project-languages">
        <option value="English"></option>
        <option value="German"></option>
        <option value="French"></option>
        <option value="Spanish"></option>
        <option value="Italian"></option>
        <option value="Dutch"></option>
        <option value="Portuguese"></option>
        <option value="Polish"></option>
        <option value="Danish"></option>
        <option value="Swedish"></option>
        <option value="Norwegian"></option>
        <option value="Finnish"></option>
        <option value="Czech"></option>
        <option value="Turkish"></option>
      </datalist><small>Independent from the application interface language.</small></label
    >
    <details class="advanced-disclosure">
      <summary>Project defaults</summary>
      <p>
        Vocabulary, recurring participants, and a protocol style can be configured in the project
        after creation.
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
