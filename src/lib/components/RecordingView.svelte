<script lang="ts">
  import type { AppRoute, MeetingSummary, RecordingStatus } from '../workflow/types';

  export let meeting: MeetingSummary;
  export let onNavigate: (route: AppRoute) => void;
  /// Read once for the whole application and handed down, rather than polled again
  /// here. Two pollers asking the same question at different rates is two answers,
  /// and the sidebar and this screen would disagree by up to a second.
  export let status: RecordingStatus;
  export let onStart: (meetingId: string) => Promise<void>;
  export let onStop: () => Promise<void>;

  let error = '';
  let working = false;

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
      error = cause instanceof Error ? cause.message : String(cause);
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
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      working = false;
    }
  }

  function clock(seconds: number) {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const rest = seconds % 60;
    const pad = (value: number) => String(value).padStart(2, '0');
    return hours > 0 ? `${hours}:${pad(minutes)}:${pad(rest)}` : `${minutes}:${pad(rest)}`;
  }

  $: quietSystem =
    status.recording && !systemHeard && status.seconds >= SILENCE_IS_SUSPICIOUS_AFTER;
  $: quietMicrophone =
    status.recording && !microphoneHeard && status.seconds >= SILENCE_IS_SUSPICIOUS_AFTER;
</script>

<main class="workspace" id="main-content">
  <header class="workspace-header">
    <div>
      <p class="breadcrumb">{meeting.title} <span>›</span> Recording</p>
      <h1 tabindex="-1">{status.recording ? 'Recording' : 'Record this meeting'}</h1>
      <p class="page-lead">
        The room and the call are captured on separate tracks, on this device. Whether the people in
        the meeting have agreed to be recorded is yours to settle, not something LocaLog can know.
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
        {status.recording ? clock(status.seconds) : 'Not recording'}
      </p>
    </div>

    {#if status.recording}
      <dl class="recording-tracks">
        <div>
          <dt>Microphone</dt>
          <dd class:is-quiet={quietMicrophone}>
            {microphoneHeard ? 'Recording' : quietMicrophone ? 'Silent so far' : 'Listening…'}
          </dd>
        </div>
        <div>
          <dt>The call</dt>
          <dd class:is-quiet={quietSystem}>
            {systemHeard ? 'Recording' : quietSystem ? 'Silent so far' : 'Listening…'}
          </dd>
        </div>
      </dl>

      {#if quietSystem}
        <p class="recording-warning" role="status">
          Nothing has arrived from the call in {status.seconds} seconds. macOS gives an application silence
          rather than an error when it has not been granted
          <strong>Screen &amp; System Audio Recording</strong>, so this is worth checking now rather
          than after the meeting.
        </p>
      {/if}
      {#if quietMicrophone}
        <p class="recording-warning" role="status">
          Nothing has arrived from the microphone in {status.seconds} seconds. Check that the right input
          is selected and that nothing else is holding it.
        </p>
      {/if}
    {/if}

    {#if status.stoppedUnexpectedly}
      <p class="setting-error" role="alert">
        The recorder stopped on its own. Whatever it captured up to that point has been kept.
      </p>
    {/if}
    {#if error}<p class="setting-error" role="alert">{error}</p>{/if}

    <div class="recording-actions">
      {#if status.recording}
        <button class="primary-action" disabled={working} onclick={stop}>
          {working ? 'Finishing…' : 'Stop recording'}
        </button>
      {:else if status.available}
        <button class="primary-action" disabled={working} onclick={start}>
          {working ? 'Starting…' : 'Start recording'}
        </button>
      {:else}
        <p>This build has no recorder. Import a file instead.</p>
      {/if}
      <button
        class="text-action"
        onclick={() => onNavigate({ name: 'meeting', meetingId: meeting.id })}
      >
        Back to the meeting
      </button>
    </div>
  </section>
</main>
