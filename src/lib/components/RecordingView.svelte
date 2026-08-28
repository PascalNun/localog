<script lang="ts">
  import { onMount } from 'svelte';
  import type {
    AppRoute,
    MeetingSummary,
    RecordingPermissions,
    RecordingStatus,
  } from '../workflow/types';
  import { errorMessage } from '../errors';
  import { clock } from '../time';
  import { t } from '../i18n';

  export let meeting: MeetingSummary;
  export let onNavigate: (route: AppRoute) => void;
  /// Read once for the whole application and handed down, rather than polled again
  /// here. Two pollers asking the same question at different rates is two answers,
  /// and the sidebar and this screen would disagree by up to a second.
  export let status: RecordingStatus;
  export let onStart: (meetingId: string) => Promise<void>;
  export let onStop: () => Promise<void>;
  export let onCheckPermissions: () => Promise<RecordingPermissions>;
  export let onOpenSettings: (pane: 'screen' | 'microphone') => Promise<void>;

  let error = '';
  let working = false;

  /// What the machine will allow, before anything is recorded.
  ///
  /// The warnings further down are the same hazard caught the other way round: they
  /// watch a track that stays silent and conclude something is wrong. That reading
  /// is right and it comes twelve seconds into a meeting, which is twelve seconds
  /// too late — the permission is granted in System Settings, and nobody wants to
  /// go there while people are waiting. This asks first instead.
  ///
  /// Both remain. A granted permission can still produce silence for reasons this
  /// cannot foresee — the wrong input selected, another application holding the
  /// device — and only the live reading catches those.
  let permissions: RecordingPermissions | null = null;

  async function check() {
    try {
      permissions = await onCheckPermissions();
    } catch {
      // A question that could not be put is not a refusal, and the screen says
      // nothing rather than sending somebody to fix what may not be broken.
      permissions = null;
    }
  }

  onMount(() => {
    void check();
    // Returning from System Settings is a window focus, and it is exactly when the
    // answer may have changed. Cheaper than polling, and it lands at the only
    // moment somebody could have changed their mind.
    const again = () => void check();
    window.addEventListener('focus', again);
    return () => window.removeEventListener('focus', again);
  });

  /// Only what is known. `unavailable` means the recorder could not be asked, which
  /// is a broken installation rather than a permission somebody has withheld.
  $: answered = permissions !== null && !permissions.unavailable;
  $: systemBlocked = answered && permissions?.systemAudio !== 'granted';
  // "Undetermined" is a normal first run: macOS puts up its own dialog the first
  // time the microphone is opened. Warning about it would be warning about the
  // permission system working.
  $: microphoneBlocked =
    answered && (permissions?.microphone === 'denied' || permissions?.microphone === 'restricted');

  /// The sound mark, drawn from what is actually being recorded.
  ///
  /// One stroke per second of the recent past rather than a level meter: the mark on
  /// the start page is a waveform, and a recording in progress should make that mark
  /// the live one instead of introducing a second visual language for audio. It also
  /// says something a meter cannot — a stretch of near-silence stays visible for
  /// half a minute, so somebody glancing over sees that nothing has been arriving.
  const STROKES = 28;
  const strokes = Array.from({ length: STROKES }, (_, at) => at);
  let recent: number[] = [];
  let lastSecond = -1;

  function remember(next: RecordingStatus) {
    if (!next.recording || next.seconds === lastSecond) return;
    lastSecond = next.seconds;
    recent = [...recent, Math.max(next.systemPeak, next.microphonePeak)].slice(-STROKES);
  }

  /// Whether a track has produced anything at all yet.
  ///
  /// This is the honest reading of a real hazard rather than a nicety. macOS hands an
  /// application that has not been granted Screen & System Audio Recording *silence*
  /// rather than refusing it, so a recording can run for ninety minutes and capture
  /// nothing from the call, with no error anywhere. Somebody has to be told while
  /// they can still do something about it.
  let systemHeard = false;
  let microphoneHeard = false;
  const SILENCE_IS_SUSPICIOUS_AFTER = 12;

  /// Each new reading feeds the mark and the two track readouts.
  ///
  /// Driven by the status arriving rather than by a timer of its own: the reading is
  /// already taken once a second for the whole application, and taking it twice would
  /// let this screen and the sidebar disagree.
  $: if (status) {
    remember(status);
    if (status.systemPeak > 0.001) systemHeard = true;
    if (status.microphonePeak > 0.001) microphoneHeard = true;
  }

  async function start() {
    working = true;
    error = '';
    recent = [];
    systemHeard = false;
    microphoneHeard = false;
    try {
      await onStart(meeting.id);
    } catch (cause) {
      error = errorMessage(cause);
    } finally {
      working = false;
    }
  }

  async function stop() {
    working = true;
    try {
      await onStop();
      onNavigate({ name: 'meeting', meetingId: meeting.id });
    } catch (cause) {
      error = errorMessage(cause);
    } finally {
      working = false;
    }
  }

  $: quietSystem =
    status.recording && !systemHeard && status.seconds >= SILENCE_IS_SUSPICIOUS_AFTER;
  $: quietMicrophone =
    status.recording && !microphoneHeard && status.seconds >= SILENCE_IS_SUSPICIOUS_AFTER;
</script>

<main class="workspace" id="main-content">
  <header class="workspace-header">
    <div>
      <p class="breadcrumb">{meeting.title} <span>›</span> {$t.shell.breadcrumbRecording}</p>
      <h1 tabindex="-1">
        {status.recording ? $t.record.recordingNow : $t.record.recordThisMeeting}
      </h1>
      <p class="page-lead">
        {$t.record.lead}
      </p>
    </div>
  </header>

  <section class="recording-stage">
    <div class="recording-mark" class:is-live={status.recording}>
      <svg viewBox="0 0 {STROKES * 12} 72" aria-hidden="true">
        {#each strokes as index (index)}
          {@const peak = recent[recent.length - STROKES + index] ?? 0}
          {@const height = Math.max(2, peak * 62)}
          <line x1={index * 12 + 6} y1={36 - height / 2} x2={index * 12 + 6} y2={36 + height / 2} />
        {/each}
      </svg>
      <p class="recording-elapsed" aria-live="polite">
        {status.recording ? clock(status.seconds) : $t.record.notRecording}
      </p>
    </div>

    {#if status.recording}
      <dl class="recording-tracks">
        <div>
          <dt>{$t.record.microphone}</dt>
          <dd class:is-quiet={quietMicrophone}>
            {microphoneHeard
              ? $t.record.trackRecording
              : quietMicrophone
                ? $t.record.trackSilent
                : $t.record.trackListening}
          </dd>
        </div>
        <div>
          <dt>{$t.record.theCall}</dt>
          <dd class:is-quiet={quietSystem}>
            {systemHeard
              ? $t.record.trackRecording
              : quietSystem
                ? $t.record.trackSilent
                : $t.record.trackListening}
          </dd>
        </div>
      </dl>

      {#if quietSystem}
        <p class="recording-warning" role="status">
          {$t.record.quietCall(status.seconds)}
        </p>
      {/if}
      {#if quietMicrophone}
        <p class="recording-warning" role="status">
          {$t.record.quietMicrophone(status.seconds)}
        </p>
      {/if}
    {/if}

    <!-- Before the button, because after it is a meeting. -->
    {#if !status.recording && status.available && (systemBlocked || microphoneBlocked)}
      <div class="recording-permission" role="status">
        {#if systemBlocked}
          <p>
            <strong>{$t.record.callWouldNotRecordTitle}</strong>
            {$t.record.callWouldNotRecordBody}
          </p>
          <button class="secondary-action" onclick={() => onOpenSettings('screen')}>
            {$t.record.openTheSetting}
          </button>
        {/if}
        {#if microphoneBlocked}
          <p>
            <strong>{$t.record.roomWouldNotRecordTitle}</strong>
            {$t.record.roomWouldNotRecordBody}
          </p>
          <button class="secondary-action" onclick={() => onOpenSettings('microphone')}>
            {$t.record.openTheSetting}
          </button>
        {/if}
        <p class="recording-permission-aside">
          {$t.record.grantedInSettings}
        </p>
      </div>
    {/if}

    <!-- The recorder's own words, which it has always written and nothing read.
         Shown verbatim: this is the one thing that can distinguish "a permission
         is missing" from "Core Audio refused for a reason we did not foresee",
         and paraphrasing it would throw away the only detail that separates them. -->
    {#if status.notes.length > 0}
      <div class="recording-permission" role="status">
        <p><strong>{$t.record.recorderNotesTitle}</strong></p>
        <ul class="recorder-notes">
          {#each status.notes as note (note)}
            <li>{note}</li>
          {/each}
        </ul>
      </div>
    {/if}

    {#if status.stoppedUnexpectedly}
      <p class="setting-error" role="alert">
        {$t.record.stoppedOnItsOwn}
      </p>
    {/if}
    {#if error}<p class="setting-error" role="alert">{error}</p>{/if}

    <div class="recording-actions">
      {#if status.recording}
        <button class="primary-action" disabled={working} onclick={stop}>
          {working ? $t.record.finishing : $t.record.stopRecording}
        </button>
      {:else if status.available}
        <button class="primary-action" disabled={working} onclick={start}>
          {working ? $t.record.starting : $t.record.startRecording}
        </button>
      {:else}
        <p>{$t.record.noRecorder}</p>
      {/if}
      <button
        class="text-action"
        onclick={() => onNavigate({ name: 'meeting', meetingId: meeting.id })}
      >
        {$t.record.backToMeeting}
      </button>
    </div>
  </section>
</main>
