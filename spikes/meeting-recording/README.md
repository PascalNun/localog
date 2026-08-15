# Recording a meeting

This study asks what it costs to record a meeting from inside the application,
rather than importing a file somebody else recorded.

The goal is one button. Click record, and the microphone in the room and the audio
from the conference call are both captured, on separate tracks, and the recording
survives whatever the next ninety minutes do to the machine.

## The contract comes first, because the platforms differ and the product does not

Capturing system audio is a different problem on every operating system, and the
temptation is to build the one in front of us and discover later that its shape has
leaked through the whole application. So the boundary is defined before any of it:

A **recorder** is a supervised subprocess, like the transcription and diarisation
runtimes already are. It is given two paths and it writes two files.

```
record-meeting --system <path.wav> --microphone <path.wav>
```

- Both files are 48 kHz mono 16-bit PCM WAV, written continuously.
- One JSON line per second on stdout: `{"seconds":N,"systemPeak":F,"microphonePeak":F}`,
  which is what a level display and a running duration are drawn from.
- `SIGINT` or `SIGTERM` stops cleanly and finalises both files.
- `SIGKILL`, a panic or a power cut leaves both files valid and playable, missing at
  most the last second.

Nothing above mentions an operating system. The application spawns a recorder,
watches two files grow, and never learns what a process tap or a monitor source is.

## What each platform needs

|         | System audio                       | Microphone    | Cost                                            |
| ------- | ---------------------------------- | ------------- | ----------------------------------------------- |
| Linux   | PipeWire/PulseAudio monitor source | Pulse or ALSA | ffmpeg already does both — likely no new binary |
| Windows | WASAPI loopback                    | WASAPI        | unresearched                                    |
| macOS   | Core Audio process tap             | AVFoundation  | a native component; this study                  |

macOS is the expensive one and the first audience's machine, so it is what this
study builds. Linux looks close to free: ffmpeg carries the `pulse` demuxer as
standard, the application already ships ffmpeg, and a recorder there may be a thin
wrapper that emits the same lines. That is a claim from the format list rather than
a measurement, and it needs testing on an actual Linux machine before anybody
relies on it.

The macOS route is `AudioHardwareCreateProcessTap`, available since macOS 14.2.

It was chosen on the belief that it asks for audio capture rather than screen
recording, which would be the honest permission for an audio product to request.
**That belief was wrong.** macOS gates system-audio capture behind the same
_Screen & System Audio Recording_ permission either way, so a tap has no advantage
over ScreenCaptureKit on that count — and ScreenCaptureKit is better documented and
reaches back to macOS 13.

## What was measured

On an M1 Pro running macOS 26.3, Swift 6.2.

### The microphone works

Peak level up to 0.918 with sound present in all 235 seconds of a four-minute run.
Converted to 48 kHz mono from whatever the device offers, so a headset that arrives
at 16 kHz and a desk microphone at 96 kHz produce the same track.

### Surviving a kill works, and was tested by accident

A run was killed outright rather than stopped. Both files were valid and playable
afterwards. The headers declared 235.168 s and 234.900 s while the files held
235.285 s and 234.985 s, so the recording lost 0.117 s of _declared_ length and no
audio at all — within the one-second checkpoint, as intended. Being killed is the
normal case this has to survive, and it does.

### System audio needs a permission, and says nothing when it does not have one

This is the finding that matters, and it is a permission rather than a bug.

`AudioHardwareCreateProcessTap` returns no error. The tap and its private aggregate
device are both created. The IO proc is called at the right rate with correctly
shaped buffers — one buffer, two channels, 512 frames. **Every sample is zero**, for
every second of every run, while audio is demonstrably playing. There is no TCC
denial in the system log, no complaint from `coreaudiod`, and no permission dialog.

`CGPreflightScreenCaptureAccess()` answers it: **false**. macOS gates system-audio
capture behind _Screen & System Audio Recording_, and an application that has not
been granted it is handed silence rather than refused.

Several things were tried first and none of them was the cause: a main sub-device
to give the aggregate a clock, `NSAudioCaptureUsageDescription` linked into the
binary's `__info_plist`, ad-hoc code signing, and running from inside a signed
`.app` bundle. Worth recording, so nobody spends the afternoon on them again.

The recorder now asks before it creates anything and refuses with that explanation
rather than producing a file of nothing.

**This is the requirement the study exists to have found.** A recorder that silently
captures nothing is the one failure this product cannot ship, and macOS will do
exactly that by default. Whatever gets built has to establish that sound is arriving
before the meeting starts — which is the same live level display the interface
already wants, promoted from a nicety to a correctness requirement.

### Drift between the tracks is real and unexplained

Over 235 seconds the two tracks ended 0.300 s apart. A single run cannot separate a
fixed offset at startup, which is harmless and correctable once, from an
accumulating clock difference, which is neither — at that rate it would be about six
seconds across the reference meeting, and every transcript timestamp would inherit
it. Two runs of different lengths would tell them apart. This needs answering before
recording is built, not after.

## What this changes about the estimate

macOS system audio was called the expensive platform on the strength of reading an
API header. That was right, and for a better reason than the one given: the API is
approachable and its permission model is not.

The remaining macOS work is a permission flow rather than an audio problem — asking
for it, explaining why a product that only wants sound must ask to record the
screen, and handling a refusal without pretending to record. Since the permission is
the same either way, ScreenCaptureKit is now the better-supported route rather than
the compromise it looked like.

The microphone and the crash-survival mechanism are done and are the reusable parts.

## Storage

Recording at 48 kHz mono 16-bit costs 5.76 MB a minute per track, so two tracks over
a ninety-minute meeting is about 1 GB. The same recording as 32 kbps Opus is about
29 MB. Raw is the working format; Opus is the stored one, and the owner has chosen
it. Whether to encode as the meeting runs or afterwards is a crash-safety question:
a truncated Ogg Opus stream stays playable up to the cut, so encoding live is viable
and avoids writing a gigabyte in order to delete it.
