import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { FakeWorkflowBridge } from './fakeBridge';

describe('FakeWorkflowBridge', () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it('keeps stable meeting lifecycle separate from a running job', async () => {
    const bridge = new FakeWorkflowBridge({ tickMs: 10, progressStep: 50 });
    await bridge.startTranscription('meeting-kickoff');

    let snapshot = await bridge.getSnapshot();
    expect(snapshot.meetings.find(({ id }) => id === 'meeting-kickoff')?.lifecycle).toBe(
      'source_ready',
    );
    expect(snapshot.activeJob?.state).toBe('queued');

    await vi.advanceTimersByTimeAsync(20);
    snapshot = await bridge.getSnapshot();
    expect(snapshot.meetings.find(({ id }) => id === 'meeting-kickoff')?.lifecycle).toBe(
      'transcript_ready',
    );
    expect(snapshot.activeJob?.outcome).toBe('succeeded');
  });

  it('cancels work without deleting the latest stable state', async () => {
    const bridge = new FakeWorkflowBridge({ tickMs: 10, progressStep: 20 });
    await bridge.startTranscription('meeting-kickoff');
    await vi.advanceTimersByTimeAsync(10);
    await bridge.cancelActiveJob();
    await vi.advanceTimersByTimeAsync(10);

    const snapshot = await bridge.getSnapshot();
    expect(snapshot.meetings.find(({ id }) => id === 'meeting-kickoff')?.lifecycle).toBe(
      'source_ready',
    );
    expect(snapshot.activeJob?.outcome).toBe('cancelled');
  });

  it('supports a synthetic failure and safe retry', async () => {
    const bridge = new FakeWorkflowBridge({ tickMs: 10, progressStep: 50 });
    await bridge.setNextJobOutcome('failure');
    await bridge.startTranscription('meeting-kickoff');
    await vi.advanceTimersByTimeAsync(10);
    expect((await bridge.getSnapshot()).activeJob?.state).toBe('failed');

    await bridge.retryActiveJob();
    await vi.advanceTimersByTimeAsync(20);
    const snapshot = await bridge.getSnapshot();
    expect(snapshot.activeJob?.outcome).toBe('succeeded');
    expect(snapshot.transcripts['meeting-kickoff']).toHaveLength(4);
  });

  it('returns reviewed protocols to draft when their content changes', async () => {
    const bridge = new FakeWorkflowBridge();
    await bridge.markReviewed('meeting-envelope-options');
    await bridge.updateProtocol('meeting-envelope-options', '# Revised protocol');
    const snapshot = await bridge.getSnapshot();
    expect(snapshot.meetings.find(({ id }) => id === 'meeting-envelope-options')?.lifecycle).toBe(
      'protocol_draft',
    );
  });
});
