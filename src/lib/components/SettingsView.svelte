<script lang="ts">
  import type { FakeJobOutcome } from '../workflow/types';
  import Icon from './Icon.svelte';

  export let theme: 'light' | 'dark';
  export let nextJobOutcome: FakeJobOutcome;
  export let onToggleTheme: () => void;
  export let onSetNextJobOutcome: (outcome: FakeJobOutcome) => Promise<void>;

  let section = 'General';
  const sections = [
    'General',
    'Models',
    'Transcription',
    'Storage',
    'Privacy',
    'Appearance',
    'Advanced',
  ];
</script>

<main class="workspace" id="main-content">
  <header class="workspace-header compact-header">
    <div>
      <p class="eyebrow">Application</p>
      <h1 tabindex="-1">Settings</h1>
      <p>Professional defaults first. Runtime details stay progressively disclosed.</p>
    </div>
  </header>
  <div class="settings-layout">
    <nav class="settings-nav" aria-label="Settings sections">
      {#each sections as item (item)}<button
          class:active={section === item}
          onclick={() => (section = item)}>{item}</button
        >{/each}
    </nav>
    <section class="settings-panel" aria-live="polite">
      <p class="eyebrow">{section}</p>
      <h2>{section}</h2>
      {#if section === 'General'}
        <div class="setting-row">
          <div>
            <h3>Interface language</h3>
            <p>Independent from each meeting’s transcription and protocol language.</p>
          </div>
          <select><option>English</option></select>
        </div>
        <div class="setting-row">
          <div>
            <h3>Default export</h3>
            <p>The format offered first in the protocol editor.</p>
          </div>
          <select><option>Markdown</option><option>Plain text</option></select>
        </div>
      {:else if section === 'Models'}
        <div class="setting-row">
          <div>
            <h3>Protocol provider</h3>
            <p>Phase 0 discovers installed providers only. No model downloads.</p>
          </div>
          <span class="setting-value">Not checked</span>
        </div>
        <div class="notice-inline">
          Ollama is a development-spike baseline, not the accepted public distribution model.
        </div>
      {:else if section === 'Transcription'}
        <div class="setting-row">
          <div>
            <h3>Default quality</h3>
            <p>Human-readable preset; exact model mapping remains advanced.</p>
          </div>
          <select><option>Balanced</option><option>Fast</option><option>Accurate</option></select>
        </div>
      {:else if section === 'Storage'}
        <div class="setting-row">
          <div>
            <h3>Working storage</h3>
            <p>App-managed for v0.1, with explicit exports and a documented location.</p>
          </div>
          <span class="setting-value">Application data</span>
        </div>
        <div class="notice-inline">No real files are stored by this Phase 0 fake workflow.</div>
      {:else if section === 'Privacy'}
        <div class="setting-row">
          <div>
            <h3>Telemetry</h3>
            <p>LocaLog does not include analytics, remote crash reporting, or cloud sync.</p>
          </div>
          <span class="safe-note"><Icon name="check" size={15} /> Off</span>
        </div>
        <div class="setting-row">
          <div>
            <h3>Content logging</h3>
            <p>Transcript, protocol, and audio content stays out of ordinary logs.</p>
          </div>
          <span class="safe-note"><Icon name="check" size={15} /> Excluded</span>
        </div>
      {:else if section === 'Appearance'}
        <div class="setting-row">
          <div>
            <h3>Theme</h3>
            <p>Warm cream light mode or designed warm-charcoal dark mode.</p>
          </div>
          <button class="secondary-action" onclick={onToggleTheme}
            ><Icon name={theme === 'light' ? 'moon' : 'sun'} size={16} /> Use {theme === 'light'
              ? 'dark'
              : 'light'}</button
          >
        </div>
        <div class="setting-row">
          <div>
            <h3>Typeface</h3>
            <p>Barlow is bundled locally; no remote font request.</p>
          </div>
          <span class="setting-value">Barlow</span>
        </div>
      {:else}
        <div class="setting-row">
          <div>
            <h3>Next fake job</h3>
            <p>Development-only control for reviewing failure and retry states.</p>
          </div>
          <select
            value={nextJobOutcome}
            onchange={(event) => onSetNextJobOutcome(event.currentTarget.value as FakeJobOutcome)}
            ><option value="success">Complete normally</option><option value="failure"
              >Fail once, then allow retry</option
            ></select
          >
        </div>
        <div class="notice-inline warning">
          <Icon name="warning" size={16} /> This affects only the in-memory synthetic runtime.
        </div>
      {/if}
    </section>
  </div>
</main>
