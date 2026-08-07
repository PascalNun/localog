<script lang="ts">
  import type {
    ProjectSummary,
    ProtocolStyle,
    VocabularyDraft,
    VocabularyEntry,
  } from '../workflow/types';
  import Icon from './Icon.svelte';

  export let kind: 'styles' | 'vocabulary';
  export let styles: ProtocolStyle[];
  export let vocabulary: VocabularyEntry[];
  export let projects: ProjectSummary[];
  export let onSaveTerm: (entry: VocabularyDraft) => Promise<void> = async () => undefined;
  export let onDeleteTerm: (entryId: string) => Promise<void> = async () => undefined;

  // Ordered by how much a transcriber gains from being told. Names cannot be
  // guessed; field terminology is usually already known. The runtime is given
  // only a short prompt, so this order decides what survives.
  const CATEGORIES = [
    'Person',
    'Organisation',
    'Project',
    'Abbreviation',
    'Technical term',
    'Other',
  ] as const;

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
      error = cause instanceof Error ? cause.message : String(cause);
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
      error = cause instanceof Error ? cause.message : String(cause);
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
    if (entry.scope !== 'Project') return 'Global vocabulary';
    return projects.find((project) => project.id === entry.projectId)?.name ?? 'Unknown project';
  }
</script>

<main class="workspace" id="main-content">
  <header class="workspace-header library-header">
    <div>
      <p class="eyebrow">Library</p>
      <h1 tabindex="-1">{kind === 'styles' ? 'Protocol styles' : 'Vocabulary'}</h1>
      <p>
        {kind === 'styles'
          ? 'Reusable professional document presets—not raw prompts.'
          : 'Preferred terminology shared globally or within a project.'}
      </p>
    </div>
    {#if kind === 'vocabulary' && !draft}
      <button class="secondary-action" onclick={startAdding} disabled={busy}>Add term</button>
    {/if}
  </header>

  {#if kind === 'styles'}
    <section class="library-list" aria-label="Protocol styles">
      {#each styles as style (style.id)}<article>
          <div class="library-icon"><Icon name="document" /></div>
          <div>
            <h2>{style.name}</h2>
            <p>{style.description}</p>
          </div>
          <span class="meta">{style.language}</span>
        </article>{/each}
    </section>
  {:else}
    <div class="library-scope-note">
      <strong>Ownership is automatic.</strong><span
        >Project vocabulary applies to its meetings without repeated selection.</span
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
        <h2>No vocabulary entries yet</h2>
        <p>
          Add the names, firms and abbreviations this work uses so they are transcribed correctly.
        </p>
      </div>
    {:else}
      <section class="library-list" aria-label="Vocabulary entries">
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
