<script lang="ts">
  import type {
    ProjectSummary,
    ExportTemplate,
    PageFurniture,
    ProtocolDensity,
    StyleEdit,
    ProtocolStyle,
    ProtocolStyleDetail,
    VocabularyDraft,
    VocabularyEntry,
  } from '../workflow/types';
  import Icon from './Icon.svelte';
  import { APPEARANCE_CHOICES } from '../protocol/appearance';
  import { fieldLabel, furnitureIsEmpty } from '../protocol/furniture';
  import { errorMessage } from '../errors';

  export let kind: 'styles' | 'vocabulary' | 'export-templates';
  export let templates: ExportTemplate[] = [];
  export let onDeleteTemplate: (templateId: string) => Promise<void> = async () => undefined;
  export let styles: ProtocolStyle[];
  export let vocabulary: VocabularyEntry[];
  export let projects: ProjectSummary[];
  export let onOpenStyle: (styleId: string) => Promise<ProtocolStyleDetail> = async () => {
    throw new Error('Styles cannot be read here.');
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
  const DENSITY_LABEL: Record<string, string> = {
    comprehensive: 'Full prose',
    concise: 'Plain statements',
    terse: 'A line per point',
  };

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

  async function saveStyle(styleId: string) {
    styleBusy = true;
    styleError = '';
    try {
      await onUpdateStyle(styleId, {
        name: editName,
        description: editDescription,
        instructions: editInstructions.filter((line) => line.trim() !== ''),
        density: editDensity,
      });
      editingStyle = false;
      openStyle = await onOpenStyle(styleId);
    } catch (cause) {
      styleError = errorMessage(cause);
    } finally {
      styleBusy = false;
    }
  }

  async function duplicateStyle(detail: ProtocolStyleDetail) {
    styleBusy = true;
    styleError = '';
    try {
      await onDuplicateStyle(detail.id, `${detail.name} (copy)`);
      openStyle = null;
    } catch (cause) {
      styleError = errorMessage(cause);
    } finally {
      styleBusy = false;
    }
  }

  async function removeStyle(styleId: string) {
    styleBusy = true;
    styleError = '';
    try {
      await onDeleteStyle(styleId);
      openStyle = null;
    } catch (cause) {
      styleError = errorMessage(cause);
    } finally {
      styleBusy = false;
    }
  }

  const DENSITY_CHOICES: { value: ProtocolDensity; label: string }[] = [
    { value: 'comprehensive', label: 'Full prose' },
    { value: 'concise', label: 'Plain statements' },
    { value: 'terse', label: 'A line per point' },
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
  const DENSITY_MEANING: Record<string, string> = {
    comprehensive: 'Full prose. A reader who was absent can follow the discussion.',
    concise: 'Plain statements. What was said, without the retelling.',
    terse: 'A line per point. The record, and nothing around it.',
  };

  function describeFurniture(furniture: PageFurniture) {
    const rows = [furniture.header, furniture.footer]
      .map((row) => [...row.left, ...row.centre, ...row.right].map(fieldLabel).join(', '))
      .filter((part) => part !== '');
    return rows.join(' / ');
  }

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
      error = 'Enter a term.';
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
    if (entry.scope !== 'Project') return 'Every project';
    return projects.find((project) => project.id === entry.projectId)?.name ?? 'Unknown project';
  }
</script>

<main class="workspace" id="main-content">
  <header class="workspace-header library-header">
    <div>
      <p class="eyebrow">Library</p>
      <h1 tabindex="-1">
        {kind === 'styles'
          ? 'Protocol styles'
          : kind === 'export-templates'
            ? 'Export templates'
            : 'Names & terms'}
      </h1>
      <p>
        {kind === 'styles'
          ? 'What a protocol says, and in what order. Not how it is set — that is an export template.'
          : kind === 'export-templates'
            ? 'How a protocol is set: the typeface and sizes, and what repeats at the top and bottom of every page. Applied to a project from its protocol editor.'
            : 'The names transcription cannot guess: your project, the firms, the people. Measured on a real meeting, these are worth more than any other setting here.'}
      </p>
    </div>
    {#if kind === 'vocabulary' && !draft}
      <button class="secondary-action" onclick={startAdding} disabled={busy}>Add term</button>
    {/if}
  </header>

  {#if kind === 'export-templates'}
    <section class="library-list" aria-label="Export templates">
      {#each templates as template (template.id)}
        <article>
          <div class="library-icon"><Icon name="download" /></div>
          <div>
            <h2>{template.name}</h2>
            <p>{template.description}</p>
            <p class="template-detail">
              {APPEARANCE_CHOICES.font.find((choice) => choice.value === template.appearance.font)
                ?.label}
              · {template.appearance.bodySize} pt ·
              {APPEARANCE_CHOICES.pageWidth.find(
                (choice) => choice.value === template.appearance.pageWidth,
              )?.label}
              ·
              {furnitureIsEmpty(template.furniture)
                ? 'nothing repeated on the page'
                : describeFurniture(template.furniture)}
            </p>
          </div>
          {#if template.builtIn}
            <span class="meta">Shipped</span>
          {:else}
            <button class="text-action" onclick={() => void onDeleteTemplate(template.id)}
              >Remove</button
            >
          {/if}
        </article>
      {/each}
      {#if templates.length === 0}
        <article>
          <div>
            <p>
              No export templates yet. Save one from a protocol's Export section once its appearance
              and its header and footer are how you want them.
            </p>
          </div>
        </article>
      {/if}
    </section>
  {:else if kind === 'styles'}
    {#if styleError}<p class="setting-error" role="alert">{styleError}</p>{/if}
    <section class="library-list" aria-label="Protocol styles">
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
                ? 'Reading…'
                : (DENSITY_LABEL[style.density] ?? style.language)}</span
            >
            <Icon name={isOpen ? 'chevron-down' : 'chevron'} size={15} />
          </button>

          {#if isOpen && openStyle}
            {@const detail = openStyle}
            <div class="style-detail">
              <div class="style-density">
                <h3>Length</h3>
                <p>{DENSITY_MEANING[detail.density] ?? detail.density}</p>
              </div>

              {#if editingStyle}
                <div class="style-editor">
                  <label>
                    <span>Name</span>
                    <input bind:value={editName} maxlength="120" />
                  </label>
                  <label>
                    <span>Description</span>
                    <input bind:value={editDescription} maxlength="240" />
                  </label>
                  <label>
                    <span>Length</span>
                    <select bind:value={editDensity}>
                      {#each DENSITY_CHOICES as choice (choice.value)}
                        <option value={choice.value}>{choice.label}</option>
                      {/each}
                    </select>
                  </label>
                  <div class="style-instruction-fields">
                    <span class="style-fields-label">What this style asks for</span>
                    {#each editInstructions as _, at (at)}
                      <div class="style-instruction-row">
                        <textarea bind:value={editInstructions[at]} rows="2"></textarea>
                        <button
                          class="icon-button compact"
                          aria-label="Remove this instruction"
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
                      ><Icon name="plus" size={14} /> Add an instruction</button
                    >
                  </div>
                </div>
              {:else}
                <div class="style-instructions">
                  <h3>What this style asks for</h3>
                  <p class="style-note">
                    These are the instructions the model is given, in the order it is given them{detail.asShipped
                      ? ', exactly as this style shipped'
                      : ''}.
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
                  <h3>Checked on the finished protocol</h3>
                  <ul>
                    {#each detail.checks as check, at (at)}
                      <li>{check}</li>
                    {/each}
                  </ul>
                </div>
              {/if}

              <div class="style-fidelity">
                <h3>Always, in every style</h3>
                <p class="style-note">
                  These are not part of this style and cannot be edited here — they are not stored
                  with a style at all. They are added to every protocol as it is written, because a
                  document that reports a decision nobody made is not a differently-styled protocol
                  but a wrong one.
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
                      onclick={() => void saveStyle(detail.id)}>Save style</button
                    >
                    <button class="secondary-action" onclick={() => (editingStyle = false)}
                      >Cancel</button
                    >
                    <button
                      class="text-action"
                      disabled={styleBusy}
                      onclick={() => void removeStyle(detail.id)}>Delete</button
                    >
                  {:else}
                    <button class="secondary-action" onclick={() => startStyleEditing(detail)}
                      >Edit this style</button
                    >
                    <button
                      class="text-action"
                      disabled={styleBusy}
                      onclick={() => void duplicateStyle(detail)}>Duplicate</button
                    >
                  {/if}
                {:else}
                  <button
                    class="secondary-action"
                    disabled={styleBusy}
                    onclick={() => void duplicateStyle(detail)}>Duplicate to edit</button
                  >
                  <span class="style-note"
                    >A style that shipped stays as it is, so a protocol written last year can be
                    written the same way again. Copy it to make your own.</span
                  >
                {/if}
              </div>
            </div>
          {/if}
        </article>
      {/each}
    </section>
  {:else}
    <div class="library-scope-note">
      <strong>Ownership is automatic.</strong><span
        >A project's names and terms apply to its meetings without repeated selection.</span
      >
    </div>

    {#if draft}
      {@const editing = draft}
      <section class="vocabulary-editor" aria-label={editing.id ? 'Edit term' : 'Add term'}>
        <div class="vocabulary-fields">
          <label>
            <span>Term</span>
            <input
              type="text"
              bind:value={editing.term}
              placeholder="Spelling as it should appear"
              maxlength="200"
              use:takeFocus
              onkeydown={(event) => {
                if (event.key === 'Enter') void save();
                if (event.key === 'Escape') cancel();
              }}
            />
          </label>
          <label>
            <span>Category</span>
            <select bind:value={editing.category}>
              {#each CATEGORIES as category (category)}
                <option value={category}>{category}</option>
              {/each}
            </select>
          </label>
          <label>
            <span>Applies to</span>
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
              <option value="global">Every project</option>
            </select>
          </label>
        </div>
        <p class="vocabulary-hint">
          Names, firms and abbreviations help most. Ordinary professional terminology is usually
          transcribed correctly without being listed.
        </p>
        <div class="vocabulary-actions">
          <button class="secondary-action" onclick={save} disabled={busy}>
            {editing.id ? 'Save term' : 'Add term'}
          </button>
          <button class="text-action" onclick={cancel} disabled={busy}>Cancel</button>
        </div>
      </section>
    {/if}

    {#if error}
      <p class="vocabulary-error" role="alert">{error}</p>
    {/if}

    {#if vocabulary.length === 0}
      <div class="empty-inline">
        <Icon name="book" size={22} />
        <h2>No names or terms yet</h2>
        <p>
          Add the names, firms and abbreviations this work uses so they are transcribed correctly.
          On a real eighty-minute meeting this took the project's own name from never spelled
          correctly to always.
        </p>
      </div>
    {:else}
      <section class="library-list" aria-label="Names and terms">
        {#each vocabulary as entry (entry.id)}<article class:is-disabled={!entry.enabled}>
            <div>
              <h2>{entry.term}</h2>
              <p>{entry.category}{entry.enabled ? '' : ' · not in use'}</p>
            </div>
            <span class="meta">{ownerLabel(entry)}</span>
            <div class="vocabulary-row-actions">
              {#if confirmingDelete === entry.id}
                <span class="meta">Delete this term?</span>
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
                  {entry.enabled ? 'In use' : 'Not in use'}
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
