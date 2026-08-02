<script lang="ts">
  import type { AppRoute, MeetingLifecycle } from '../workflow/types';
  import Icon from './Icon.svelte';

  export let meetingId: string;
  export let lifecycle: MeetingLifecycle;
  export let onNavigate: (route: AppRoute) => void;

  const transcriptReady = () =>
    ['transcript_ready', 'protocol_draft', 'reviewed'].includes(lifecycle);
  const protocolReady = () => ['protocol_draft', 'reviewed'].includes(lifecycle);
</script>

<nav class="stage-rail" aria-label="Meeting stages">
  <button
    class:complete={lifecycle !== 'draft'}
    onclick={() => onNavigate({ name: 'meeting', meetingId })}
    ><span>{lifecycle !== 'draft' ? '✓' : '1'}</span>Source</button
  >
  <Icon name="chevron" size={14} />
  <button
    class:complete={transcriptReady()}
    disabled={!transcriptReady()}
    onclick={() => onNavigate({ name: 'transcript', meetingId })}
    ><span>{transcriptReady() ? '✓' : '2'}</span>Transcript</button
  >
  <Icon name="chevron" size={14} />
  <button
    class:complete={protocolReady()}
    disabled={!protocolReady()}
    onclick={() => onNavigate({ name: 'protocol', meetingId })}
    ><span>{protocolReady() ? '✓' : '3'}</span>Protocol</button
  >
</nav>
