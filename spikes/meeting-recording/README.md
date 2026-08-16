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

### System audio works — once permitted. It says nothing when it is not

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

**Whether the capture itself works is still unproven, and cannot be proven from a
terminal.** The permission is granted to an application, and this study's binary
runs several processes below one:

```
/bin/zsh
  └ …/Application Support/Claude/claude-code/<version>/claude.app/…/claude
      └ /Applications/Claude.app/Contents/Helpers/disclaimer
          └ /Applications/Claude.app
```

The nested bundle in the middle is separately signed, so a grant to the
application at the bottom does not reach the top, and its path carries a version
number that changes with every update. `CGRequestScreenCaptureAccess()` returns
false immediately without showing a dialog, so macOS will not even attribute the
request.

That distinction matters more than it looks: **not permitted** and **not working**
are indistinguishable from here, both being silence. The tap code has never been
observed to capture anything, and two confident explanations for that silence — a
missing clock source, a missing usage description — were already wrong.

Five ways of asking have now been tried, and all five produce silence:

1. a bare executable run from a terminal;
2. the same with `NSAudioCaptureUsageDescription` linked into `__info_plist`;
3. the same ad-hoc signed with a stable identifier;
4. the same placed inside an `.app` bundle and run directly, which still inherits
   the terminal's identity;
5. the same launched with `open`, so launchd starts it and it is its own
   responsible process rather than a child of a terminal.

The fifth is the interesting one and it changed the result, though not the way
hoped: as its own application it also lost the microphone, which had been working
by inheriting the terminal's grant. Both tracks came back as forty-four bytes of
WAV header and no audio. No permission dialog appeared and `tccd` logged nothing
naming the application.

**The owner then granted the permission, and it captures.** `systemPeak` rises to
0.256 exactly while a sound plays and returns to zero between, and 1.13 MB of real
audio lands in the file. The tap, the aggregate device, the clock source and the
mono conversion were all correct throughout; none of the five attempts failed for
any reason other than not being allowed.

Which is the finding worth keeping. **A tap that is not permitted is
indistinguishable from a tap that is broken**, and three plausible explanations
were investigated and dismissed before the real one — an unauthorised capture
looks exactly like a meeting where nobody spoke. Any recorder built on this must
establish that sound is arriving before a meeting starts rather than after it
ends, which is what the per-source level display in the reference design is
actually for.

The drift between the two tracks over eleven seconds was 0.6 s of start offset,
which is a fixed cost of starting two capture paths and not yet distinguished from
an accumulating clock difference. That still needs a long run to settle.

**This is the requirement the study exists to have found.** A recorder that silently
captures nothing is the one failure this product cannot ship, and macOS will do
exactly that by default. Whatever gets built has to establish that sound is arriving
before the meeting starts — which is the same live level display the interface
already wants, promoted from a nicety to a correctness requirement.

### A recorder that dies without cleaning up breaks the machine's audio

Found by breaking it. Three test runs were ended with `kill -9`, which bypasses the
recorder's own teardown, so their process taps and aggregate devices were never
destroyed. The processes stayed alive holding them, `coreaudiod` went to 43 % CPU,
and every application on the machine lost sound — `afplay` returning
`AudioQueueStart failed`. Killing the orphans was not enough; the daemon had to be
restarted with `sudo killall coreaudiod`.

That is a user-facing failure, not a testing accident. A recorder that crashes
during a meeting — or is force-quit, or killed by the system under memory pressure —
leaves somebody with no sound in any application and nothing to explain why. It is
worse than losing the recording.

So a recorder must not rely on its own exit path to release the tap:

- tear down on every signal it can catch, which this study does for `SIGINT` and
  `SIGTERM` and did not save it from `SIGKILL`;
- find and destroy any tap or aggregate device left behind by a previous run when
  starting, since after `SIGKILL` there is no other opportunity;
- and never be tested with `kill -9` again without checking afterwards.

The private aggregate device is invisible to `system_profiler` and to Sound
settings, which is why nothing showed the cause. `ollama ps` has an equivalent —
what a tool reports about itself is not what the machine is actually holding.

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
