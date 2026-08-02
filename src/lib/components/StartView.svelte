<script lang="ts">
  import type { AppRoute } from '../workflow/types';
  import Icon from './Icon.svelte';

  export let onNavigate: (route: AppRoute) => void;
</script>

<main class="workspace start-workspace" id="main-content">
  <div class="start-hero">
    <svg class="sound-mark" viewBox="0 0 96 72" aria-hidden="true">
      <line x1="8" y1="27" x2="8" y2="45" />
      <line x1="21" y1="17" x2="21" y2="56" />
      <line x1="34" y1="5" x2="34" y2="67" />
      <line x1="48" y1="17" x2="48" y2="56" />
      <line x1="62" y1="25" x2="62" y2="48" />
      <line x1="75" y1="14" x2="75" y2="59" />
      <line x1="88" y1="27" x2="88" y2="45" />
    </svg>
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
