<script lang="ts">
  import type { AppRoute, ProjectSummary } from '../workflow/types';
  import Icon from './Icon.svelte';

  export let projects: ProjectSummary[];
  export let route: AppRoute;
  export let currentProjectId: string | null;
  export let open = false;
  export let theme: 'light' | 'dark';
  export let onNavigate: (route: AppRoute) => void;
  export let onClose: () => void;
  export let onToggleTheme: () => void;

  function navigate(nextRoute: AppRoute) {
    onNavigate(nextRoute);
    onClose();
  }
</script>

{#if open}
  <button class="sidebar-scrim" aria-label="Close navigation" onclick={onClose}></button>
{/if}

<aside class:open class="sidebar" aria-label="Primary navigation">
  <div class="window-dots" aria-hidden="true"><span></span><span></span><span></span></div>
  <button class="wordmark" onclick={() => navigate({ name: 'start' })}>LocaLog</button>

  <nav class="sidebar-nav">
    <section aria-labelledby="projects-label">
      <div class="nav-section-heading">
        <span id="projects-label">Projects</span>
        <button
          class="icon-button compact"
          aria-label="Create project"
          onclick={() => navigate({ name: 'new-project', returnToImport: false })}
          ><Icon name="plus" size={15} /></button
        >
      </div>
      <div class="project-links">
        {#each projects as project (project.id)}
          <button
            class:active={currentProjectId === project.id}
            class="project-link"
            onclick={() => navigate({ name: 'project', projectId: project.id })}
          >
            <span>{project.name}</span><span class="count">{project.meetingCount}</span>
          </button>
        {/each}
      </div>
      <button
        class="text-action sidebar-action"
        onclick={() => navigate({ name: 'new-project', returnToImport: false })}
        ><Icon name="plus" size={15} /> New project</button
      >
    </section>

    <section aria-labelledby="library-label">
      <div class="nav-section-heading"><span id="library-label">Library</span></div>
      <button
        class:active={route.name === 'styles'}
        class="nav-link"
        onclick={() => navigate({ name: 'styles' })}
        ><Icon name="document" size={17} /> Protocol styles</button
      >
      <button
        class:active={route.name === 'vocabulary'}
        class="nav-link"
        onclick={() => navigate({ name: 'vocabulary' })}
        ><Icon name="book" size={17} /> Vocabulary</button
      >
    </section>

    <section aria-labelledby="settings-label">
      <div class="nav-section-heading"><span id="settings-label">Settings</span></div>
      <button
        class:active={route.name === 'settings'}
        class="nav-link"
        onclick={() => navigate({ name: 'settings' })}
        ><Icon name="settings" size={17} /> Settings</button
      >
    </section>
  </nav>

  <div class="sidebar-footer">
    <div class="local-status">
      <span class="status-dot success"></span>
      <span><strong>Local mode active</strong><small>Fake runtime · device only</small></span>
    </div>
    <button
      class="icon-button"
      onclick={onToggleTheme}
      aria-label={`Use ${theme === 'light' ? 'dark' : 'light'} theme`}
      ><Icon name={theme === 'light' ? 'moon' : 'sun'} /></button
    >
  </div>
</aside>
