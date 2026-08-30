<script lang="ts">
  import type { NewProjectInput } from '../workflow/types';
  import { COMMON_MEETING_LANGUAGES, meetingLanguageValue } from '../workflow/languages';
  import Icon from './Icon.svelte';
  import { errorMessage } from '../errors';
  import { t } from '../i18n';
  import { namesFromFields, type NameKind } from '../workflow/names';

  export let returnToImport: boolean;
  export let onCancel: () => void;
  export let onCreate: (input: NewProjectInput) => Promise<void>;

  let name = '';
  let description = '';
  // Unset, not English. A default language is a claim about meetings that have
  // not happened yet, and the wrong claim is expensive: it transcribed a German
  // recording in English and was only discovered eleven minutes later.
  let defaultLanguage = '';
  /// Asked for by kind rather than as one box of words, and that is the whole
  /// design. "Names & terms" is not a question anybody can answer; "the people",
  /// "the client and the firms" are. The category comes free from which box a name
  /// was typed into, and it is not decoration — when the list outgrows the
  /// transcriber's short prompt, people and organisations are what survive the trim.
  let names: Record<NameKind, string> = {
    Person: '',
    Organisation: '',
    Project: '',
    'Technical term': '',
  };

  let submitting = false;
  let submitError = '';

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!name.trim() || submitting) return;
    submitting = true;
    submitError = '';
    try {
      await onCreate({
        name,
        description,
        // The field holds the language named in the reader's language; what is
        // stored is the identifier the transcription runtime is handed.
        defaultLanguage: meetingLanguageValue(defaultLanguage),
        names: namesFromFields(names),
      });
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
        placeholder={$t.dialog.detectFromRecording}
      /><datalist id="project-languages">
        {#each COMMON_MEETING_LANGUAGES as language (language)}<option
            value={$t.meetingLanguages[language]}
          ></option>{/each}
      </datalist><small>{$t.newProject.defaultLanguageDetail}</small></label
    >
    <fieldset class="names-fieldset">
      <legend>{$t.newProject.namesHeading}</legend>
      <p class="names-lead">{$t.newProject.namesLead}</p>
      <label
        ><span>{$t.newProject.namesPeople}</span><input bind:value={names.Person} /><small
          >{$t.newProject.namesPeopleHint}</small
        ></label
      >
      <label
        ><span>{$t.newProject.namesOrganisations}</span><input
          bind:value={names.Organisation}
        /><small>{$t.newProject.namesOrganisationsHint}</small></label
      >
      <label
        ><span>{$t.newProject.namesProject}</span><input bind:value={names.Project} /><small
          >{$t.newProject.namesProjectHint}</small
        ></label
      >
      <label
        ><span>{$t.newProject.namesTerms}</span><input bind:value={names['Technical term']} /><small
          >{$t.newProject.namesTermsHint}</small
        ></label
      >
      <p class="names-note">{$t.newProject.namesNote}</p>
    </fieldset>
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

<style>
  /* Its own block, because it is a group of questions rather than four more rows
     of the form: the lead has to be read before the first field is answered. */
  .names-fieldset {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    margin: 0;
    padding: 1.1rem 1.2rem 1.2rem;
    border: 1px solid var(--line-soft, rgba(120, 110, 95, 0.28));
    border-radius: 10px;
    background: var(--surface-raised, rgba(255, 255, 255, 0.03));
  }

  .names-fieldset legend {
    padding: 0 0.4rem;
    font-weight: 600;
    letter-spacing: 0.01em;
  }

  .names-lead,
  .names-note {
    margin: 0;
    font-size: 0.9rem;
    line-height: 1.5;
    color: var(--ink-soft, rgba(90, 82, 70, 0.85));
  }

  .names-note {
    font-size: 0.85rem;
  }
</style>
