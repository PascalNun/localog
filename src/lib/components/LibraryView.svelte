<script lang="ts">
  import type { ProjectSummary, ProtocolStyle, VocabularyEntry } from '../workflow/types';
  import Icon from './Icon.svelte';

  export let kind: 'styles' | 'vocabulary';
  export let styles: ProtocolStyle[];
  export let vocabulary: VocabularyEntry[];
  export let projects: ProjectSummary[];
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
    <section class="library-list" aria-label="Vocabulary entries">
      {#each vocabulary as entry (entry.id)}<article>
          <div>
            <h2>{entry.term}</h2>
            <p>{entry.category}</p>
          </div>
          <span class="meta"
            >{entry.scope === 'Project'
              ? projects.find((project) => project.id === entry.projectId)?.name
              : 'Global vocabulary'}</span
          >
        </article>{/each}
    </section>
  {/if}
</main>
