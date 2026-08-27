<script lang="ts">
  import type { AppRoute, MeetingLifecycle } from '../workflow/types';
  import Icon from './Icon.svelte';
  import { t } from '../i18n';

  export let meetingId: string;
  export let lifecycle: MeetingLifecycle;
  export let onNavigate: (route: AppRoute) => void;

  const transcriptReady = () =>
    ['transcript_ready', 'protocol_draft', 'reviewed'].includes(lifecycle);
  const protocolReady = () => ['protocol_draft', 'reviewed'].includes(lifecycle);
</script>

<nav class="stage-rail" aria-label={$t.stages.label}>
  <button
    class:complete={lifecycle !== 'draft'}
    onclick={() => onNavigate({ name: 'meeting', meetingId })}
    ><span>{lifecycle !== 'draft' ? '✓' : '1'}</span>{$t.stages.source}</button
  >
  <Icon name="chevron" size={14} />
  <button
    class:complete={transcriptReady()}
    disabled={!transcriptReady()}
    onclick={() => onNavigate({ name: 'transcript', meetingId })}
    ><span>{transcriptReady() ? '✓' : '2'}</span>{$t.stages.transcript}</button
  >
  <Icon name="chevron" size={14} />
  <button
    class:complete={protocolReady()}
    disabled={!protocolReady()}
    onclick={() => onNavigate({ name: 'protocol', meetingId })}
    ><span>{protocolReady() ? '✓' : '3'}</span>{$t.stages.protocol}</button
  >
</nav>
