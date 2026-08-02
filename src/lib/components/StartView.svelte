<script lang="ts">
  import type { AppRoute } from '../workflow/types';
  import Icon from './Icon.svelte';

  export let onNavigate: (route: AppRoute) => void;
</script>

<main class="workspace start-workspace" id="main-content">
  <div class="start-hero">
    <div class="sound-mark" aria-hidden="true">
      <span></span><span></span><span></span><span></span><span></span>
    </div>
    <p class="eyebrow">Private meeting records, on your Mac</p>
    <h1 tabindex="-1">Start a meeting</h1>
    <p class="hero-copy">
      Import an audio or video file. Review every step before it becomes a protocol.
    </p>

    <button
      class="import-hero-action"
      onclick={() => onNavigate({ name: 'new-meeting', projectId: null })}
    >
      <span class="import-icon"><Icon name="upload" size={26} /></span>
      <span
        ><strong>Import recording</strong><small
          >Choose a project, then keep every artifact in context</small
        ></span
      >
      <Icon name="arrow" />
    </button>
  </div>

  <div class="local-promise">
    <span class="lock-mark" aria-hidden="true">⌂</span>
    <span
      ><strong>Everything in this demo stays local.</strong><small
        >No account, cloud service, or telemetry.</small
      ></span
    >
  </div>
</main>
