<script lang="ts">
  import type {
    ProjectSummary,
    ProtocolDensity,
    StyleEdit,
    ProtocolStyle,
    ProtocolStyleDetail,
    VocabularyDraft,
    VocabularyEntry,
  } from '../workflow/types';
  import Icon from './Icon.svelte';
  import { errorMessage } from '../errors';
  import { t } from '../i18n';

  export let kind: 'styles' | 'vocabulary';
  export let styles: ProtocolStyle[];
  export let vocabulary: VocabularyEntry[];
  export let projects: ProjectSummary[];
  export let onOpenStyle: (styleId: string) => Promise<ProtocolStyleDetail> = async () => {
    throw new Error($t.library.stylesUnreadable);
  };
  export let onDuplicateStyle: (styleId: string, name: string) => Promise<void> = async () =>
    undefined;
  export let onUpdateStyle: (styleId: string, edit: StyleEdit) => Promise<void> = async () =>
    undefined;
  export let onDeleteStyle: (styleId: string) => Promise<void> = async () => undefined;
  export let onSaveTerm: (entry: VocabularyDraft) => Promise<void> = async () => undefined;
  export let onDeleteTerm: (entryId: string) => Promise<void> = async () => undefined;

  // Ordered by how much a transcriber gains from being told. Names cannot be
  // guessed; field terminology is usually already known. The runtime is given
  // only a short prompt, so this order decides what survives.
  // What a style asks for, said plainly. The value steers the model and sizes the
  // answer budget, so it belongs where someone choosing a style can see it.
  let DENSITY_LABEL: Record<string, string>;
  $: DENSITY_LABEL = {
    comprehensive: $t.library.densityFull,
    concise: $t.library.densityPlain,
    terse: $t.library.densityLine,
  };

  /// The values stored in the database, which stay English whatever the
  /// interface is in: translating them would write German into a column and
  /// break the same list opened in English. Only the label is translated.
  const CATEGORIES = [
    'Person',
    'Organisation',
    'Project',
    'Abbreviation',
    'Technical term',
    'Other',
  ] as const;

  /// The style being read, and what it took to get it.
  ///
  /// Opened rather than expanded in place: the instructions are long enough that
  /// three of them open at once would be a wall, and short enough that a separate
  /// page would be ceremony.
  let openStyle: ProtocolStyleDetail | null = null;
  let openingStyleId = '';
  let styleError = '';

  /// Editing a style, which is editing what it asks for and nothing else.
  ///
  /// The fidelity rules are not here to edit: they are not stored with the style at
  /// all. They are shown beneath, so that what cannot be changed is visible rather
  /// than merely absent.
  let editingStyle = false;
  let editName = '';
  let editDescription = '';
  let editInstructions: string[] = [];
  let editDensity: ProtocolDensity = 'concise';
  let styleBusy = false;

  function startStyleEditing(detail: ProtocolStyleDetail) {
    editingStyle = true;
    editName = detail.name;
    editDescription = detail.description;
    editInstructions = [...detail.instructions];
    editDensity = detail.density;
  }

  /**
   * Run one action on a style, with the busy flag and the error line all three
   * share. Editing a style is the one thing here that reaches the desktop and
   * can be refused, so all three need to say so and none may leave the panel
   * disabled: a `finally` written out three times is a `finally` that can be
   * forgotten once.
   */
  async function styleAction(run: () => Promise<void>) {
    styleBusy = true;
    styleError = '';
    try {
      await run();
    } catch (cause) {
      styleError = errorMessage(cause);
    } finally {
      styleBusy = false;
    }
  }

  const saveStyle = (styleId: string) =>
    styleAction(async () => {
      await onUpdateStyle(styleId, {
        name: editName,
        description: editDescription,
        instructions: editInstructions.filter((line) => line.trim() !== ''),
        density: editDensity,
      });
      editingStyle = false;
      openStyle = await onOpenStyle(styleId);
    });

  const duplicateStyle = (detail: ProtocolStyleDetail) =>
    styleAction(async () => {
      await onDuplicateStyle(detail.id, `${detail.name} (copy)`);
      openStyle = null;
    });

  const removeStyle = (styleId: string) =>
    styleAction(async () => {
      await onDeleteStyle(styleId);
      openStyle = null;
    });

  let DENSITY_CHOICES: { value: ProtocolDensity; label: string }[];
  $: DENSITY_CHOICES = [
    { value: 'comprehensive', label: $t.library.densityFull },
    { value: 'concise', label: $t.library.densityPlain },
    { value: 'terse', label: $t.library.densityLine },
  ];

  async function openStyleDetail(styleId: string) {
    if (openStyle?.id === styleId) {
      openStyle = null;
      editingStyle = false;
      return;
    }
    editingStyle = false;
    openingStyleId = styleId;
    styleError = '';
    try {
      openStyle = await onOpenStyle(styleId);
    } catch (cause) {
      openStyle = null;
      styleError = errorMessage(cause);
    } finally {
      openingStyleId = '';
    }
  }

  /// What a density setting means where somebody is deciding between them.
  let DENSITY_MEANING: Record<string, string>;
  $: DENSITY_MEANING = {
    comprehensive: $t.library.densityFullMeaning,
    concise: $t.library.densityPlainMeaning,
    terse: $t.library.densityLineMeaning,
  };

  let draft: VocabularyDraft | null = null;
  let error = '';
  let busy = false;
  let confirmingDelete = '';

  function blankDraft(): VocabularyDraft {
    return {
      id: null,
      term: '',
      category: 'Person',
      scope: projects.length > 0 ? 'Project' : 'Global',
      projectId: projects[0]?.id ?? null,
      enabled: true,
    };
  }

  function startAdding() {
    error = '';
    confirmingDelete = '';
    draft = blankDraft();
  }

  function startEditing(entry: VocabularyEntry) {
    error = '';
    confirmingDelete = '';
    draft = { ...entry };
  }

  function cancel() {
    draft = null;
    error = '';
  }

  async function commit(next: VocabularyDraft) {
    busy = true;
    error = '';
    try {
      await onSaveTerm(next);
      draft = null;
    } catch (cause) {
      error = errorMessage(cause);
    } finally {
      busy = false;
    }
  }

  async function save() {
    if (!draft) return;
    if (!draft.term.trim()) {
      error = $t.library.enterATerm;
      return;
    }
    await commit(draft);
  }

  /** Switching a term off keeps it in the library but withholds it from every run. */
  async function toggleEnabled(entry: VocabularyEntry) {
    confirmingDelete = '';
    await commit({ ...entry, enabled: !entry.enabled });
  }

  async function remove(entryId: string) {
    busy = true;
    error = '';
    try {
      await onDeleteTerm(entryId);
      confirmingDelete = '';
    } catch (cause) {
      error = errorMessage(cause);
    } finally {
      busy = false;
    }
  }

  // The `autofocus` attribute is ignored when the browser considers something
  // else already focused, which is exactly the case here: the editor is opened by
  // a button click. Moving focus explicitly puts the cursor where typing belongs.
  function takeFocus(node: HTMLInputElement) {
    node.focus();
    node.select();
  }

  function ownerLabel(entry: VocabularyEntry): string {
    if (entry.scope !== 'Project') return $t.library.everyProject;
    return (
      projects.find((project) => project.id === entry.projectId)?.name ?? $t.library.unknownProject
    );
  }
</script>

<main class="workspace" id="main-content">
  <header class="workspace-header library-header">
    <div>
      <p class="eyebrow">{$t.library.eyebrow}</p>
      <h1 tabindex="-1">
        {kind === 'styles' ? $t.library.protocolStyles : $t.library.namesAndTerms}
      </h1>
      <p>
        {kind === 'styles' ? $t.library.stylesLead : $t.library.termsLead}
      </p>
    </div>
    {#if kind === 'vocabulary' && !draft}
      <button class="secondary-action" onclick={startAdding} disabled={busy}
        >{$t.library.addTerm}</button
      >
    {/if}
  </header>

  {#if kind === 'styles'}
    {#if styleError}<p class="setting-error" role="alert">{styleError}</p>{/if}
    <section class="library-list" aria-label={$t.library.protocolStyles}>
      {#each styles as style (style.id)}
        {@const isOpen = openStyle?.id === style.id}
        <article class:open={isOpen}>
          <button
            class="library-open"
            aria-expanded={isOpen}
            onclick={() => void openStyleDetail(style.id)}
          >
            <div class="library-icon"><Icon name="document" /></div>
            <div>
              <h2>{style.name}</h2>
              <p>{style.description}</p>
            </div>
            <span class="meta"
              >{openingStyleId === style.id
                ? $t.library.reading
                : (DENSITY_LABEL[style.density] ?? style.language)}</span
            >
            <Icon name={isOpen ? 'chevron-down' : 'chevron'} size={15} />
          </button>

          {#if isOpen && openStyle}
            {@const detail = openStyle}
            <div class="style-detail">
              <div class="style-density">
                <h3>{$t.library.length}</h3>
                <p>{DENSITY_MEANING[detail.density] ?? detail.density}</p>
              </div>

              {#if editingStyle}
                <div class="style-editor">
                  <label>
                    <span>{$t.library.name}</span>
                    <input bind:value={editName} maxlength="120" />
                  </label>
                  <label>
                    <span>{$t.library.description}</span>
                    <input bind:value={editDescription} maxlength="240" />
                  </label>
                  <label>
                    <span>{$t.library.length}</span>
                    <select bind:value={editDensity}>
                      {#each DENSITY_CHOICES as choice (choice.value)}
                        <option value={choice.value}>{choice.label}</option>
                      {/each}
                    </select>
                  </label>
                  <div class="style-instruction-fields">
                    <span class="style-fields-label">{$t.library.whatItAsksFor}</span>
                    {#each editInstructions as _, at (at)}
                      <div class="style-instruction-row">
                        <textarea bind:value={editInstructions[at]} rows="2"></textarea>
                        <button
                          class="icon-button compact"
                          aria-label={$t.library.removeInstruction}
                          onclick={() =>
                            (editInstructions = editInstructions.filter(
                              (__, index) => index !== at,
                            ))}><Icon name="close" size={14} /></button
                        >
                      </div>
                    {/each}
                    <button
                      class="text-action"
                      onclick={() => (editInstructions = [...editInstructions, ''])}
                      ><Icon name="plus" size={14} /> {$t.library.addInstruction}</button
                    >
                  </div>
                </div>
              {:else}
                <div class="style-instructions">
                  <h3>{$t.library.whatItAsksFor}</h3>
                  <p class="style-note">
                    {$t.library.instructionsGiven}{detail.asShipped ? $t.library.asShipped : ''}.
                  </p>
                  <ol>
                    {#each detail.instructions as instruction, at (at)}
                      <li>{instruction}</li>
                    {/each}
                  </ol>
                </div>
              {/if}

              {#if detail.checks.length > 0}
                <div class="style-sections">
                  <h3>{$t.library.checkedOnProtocol}</h3>
                  <ul>
                    {#each detail.checks as check, at (at)}
                      <li>{check}</li>
                    {/each}
                  </ul>
                </div>
              {/if}

              <div class="style-fidelity">
                <h3>{$t.library.alwaysEveryStyle}</h3>
                <p class="style-note">
                  {$t.library.invariantsNote}
                </p>
                <ul>
                  {#each detail.fidelity as rule, at (at)}
                    <li>{rule}</li>
                  {/each}
                </ul>
              </div>

              <div class="style-actions">
                {#if detail.editable}
                  {#if editingStyle}
                    <button
                      class="primary-action"
                      disabled={styleBusy}
                      onclick={() => void saveStyle(detail.id)}>{$t.library.saveStyle}</button
                    >
                    <button class="secondary-action" onclick={() => (editingStyle = false)}
                      >{$t.library.cancel}</button
                    >
                    <button
                      class="text-action"
                      disabled={styleBusy}
                      onclick={() => void removeStyle(detail.id)}>{$t.library.delete}</button
                    >
                  {:else}
                    <button class="secondary-action" onclick={() => startStyleEditing(detail)}
                      >{$t.library.editThisStyle}</button
                    >
                    <button
                      class="text-action"
                      disabled={styleBusy}
                      onclick={() => void duplicateStyle(detail)}>{$t.library.duplicate}</button
                    >
                  {/if}
                {:else}
                  <button
                    class="secondary-action"
                    disabled={styleBusy}
                    onclick={() => void duplicateStyle(detail)}>{$t.library.duplicateToEdit}</button
                  >
                  <span class="style-note">{$t.library.shippedStyleNote}</span>
                {/if}
              </div>
            </div>
          {/if}
        </article>
      {/each}
    </section>
  {:else}
    <div class="library-scope-note">
      <strong>{$t.library.ownershipAutomatic}</strong><span>{$t.library.termsScopeNote}</span>
    </div>

    {#if draft}
      {@const editing = draft}
      <section
        class="vocabulary-editor"
        aria-label={editing.id ? $t.library.editTerm : $t.library.addTerm}
      >
        <div class="vocabulary-fields">
          <label>
            <span>{$t.library.term}</span>
            <input
              type="text"
              bind:value={editing.term}
              placeholder={$t.library.spellingAsShown}
              maxlength="200"
              use:takeFocus
              onkeydown={(event) => {
                if (event.key === 'Enter') void save();
                if (event.key === 'Escape') cancel();
              }}
            />
          </label>
          <label>
            <span>{$t.library.category}</span>
            <select bind:value={editing.category}>
              {#each CATEGORIES as category (category)}
                <option value={category}>{category}</option>
              {/each}
            </select>
          </label>
          <label>
            <span>{$t.library.appliesTo}</span>
            <select
              value={editing.scope === 'Project' ? (editing.projectId ?? '') : 'global'}
              onchange={(event) => {
                const chosen = event.currentTarget.value;
                if (chosen === 'global') {
                  editing.scope = 'Global';
                  editing.projectId = null;
                } else {
                  editing.scope = 'Project';
                  editing.projectId = chosen;
                }
              }}
            >
              {#each projects as project (project.id)}
                <option value={project.id}>{project.name}</option>
              {/each}
              <option value="global">{$t.library.everyProject}</option>
            </select>
          </label>
        </div>
        <p class="vocabulary-hint">
          {$t.library.whichTermsHelp}
        </p>
        <div class="vocabulary-actions">
          <button class="secondary-action" onclick={save} disabled={busy}>
            {editing.id ? $t.library.saveTerm : $t.library.addTerm}
          </button>
          <button class="text-action" onclick={cancel} disabled={busy}>{$t.library.cancel}</button>
        </div>
      </section>
    {/if}

    {#if error}
      <p class="vocabulary-error" role="alert">{error}</p>
    {/if}

    {#if vocabulary.length === 0}
      <div class="empty-inline">
        <Icon name="book" size={22} />
        <h2>{$t.library.noTerms}</h2>
        <p>
          {$t.library.termsLeadLong}
        </p>
      </div>
    {:else}
      <section class="library-list" aria-label={$t.library.namesAndTerms}>
        {#each vocabulary as entry (entry.id)}<article class:is-disabled={!entry.enabled}>
            <div>
              <h2>{entry.term}</h2>
              <p>{entry.category}{entry.enabled ? '' : ' · not in use'}</p>
            </div>
            <span class="meta">{ownerLabel(entry)}</span>
            <div class="vocabulary-row-actions">
              {#if confirmingDelete === entry.id}
                <span class="meta">{$t.library.deleteThisTerm}</span>
                <button class="text-action" onclick={() => remove(entry.id)} disabled={busy}>
                  Delete
                </button>
                <button class="text-action" onclick={() => (confirmingDelete = '')} disabled={busy}>
                  Keep
                </button>
              {:else}
                <button
                  class="text-action"
                  aria-pressed={entry.enabled}
                  onclick={() => toggleEnabled(entry)}
                  disabled={busy}
                >
                  {entry.enabled ? $t.library.inUse : $t.library.notInUse}
                </button>
                <button class="text-action" onclick={() => startEditing(entry)} disabled={busy}>
                  Edit
                </button>
                <button
                  class="text-action"
                  onclick={() => (confirmingDelete = entry.id)}
                  disabled={busy}
                >
                  Remove
                </button>
              {/if}
            </div>
          </article>{/each}
      </section>
    {/if}
  {/if}
</main>
