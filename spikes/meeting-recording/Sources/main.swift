// The macOS recorder: system audio and the microphone, on separate tracks.
//
// Implements the contract in the README. Nothing about this file should be
// visible to the application, which spawns a recorder and watches two files grow.
//
// System audio comes from a Core Audio process tap, which asks for audio capture
// rather than screen recording. The tap is wrapped in a private aggregate device,
// because a tap is not something an IO proc can be attached to directly.

import AVFoundation
import CoreAudio
import Darwin
import Foundation

// MARK: - Writing audio that survives being killed

/// A WAV file written as the meeting happens.
///
/// The lengths in a WAV header are only known when the recording stops, and a
/// recorder that learns them only then is a recorder that loses a ninety-minute
/// meeting to a crash at minute eighty-nine. So the header is rewritten every
/// second over the file that is already on disk: a recording killed outright is
/// valid and playable, short by at most that second.
final class GrowingWave {
    private let handle: FileHandle
    private var frames: UInt32 = 0
    private var declared: UInt32 = 0
    private let rate: UInt32
    private let queue = DispatchQueue(label: "wave")

    init(path: String, sampleRate: UInt32) throws {
        FileManager.default.createFile(atPath: path, contents: nil)
        guard let handle = FileHandle(forWritingAtPath: path) else {
            throw RecorderError.cannotWrite(path)
        }
        self.handle = handle
        self.rate = sampleRate
        try handle.write(contentsOf: GrowingWave.header(frames: 0, rate: sampleRate))
    }

    /// Mono 16-bit, which is what speech needs and what the transcription runtime
    /// wants anyway. Stereo would double the file to record a conference call that
    /// was mixed to one channel before it reached us.
    private static func header(frames: UInt32, rate: UInt32) -> Data {
        let bytes = frames * 2
        var data = Data()
        func put<T: FixedWidthInteger>(_ value: T) {
            withUnsafeBytes(of: value.littleEndian) { data.append(contentsOf: $0) }
        }
        data.append(contentsOf: Array("RIFF".utf8))
        put(UInt32(36) &+ bytes)
        data.append(contentsOf: Array("WAVEfmt ".utf8))
        put(UInt32(16))
        put(UInt16(1))  // uncompressed PCM
        put(UInt16(1))  // one channel
        put(rate)
        put(rate &* 2)  // bytes a second
        put(UInt16(2))  // bytes a frame
        put(UInt16(16))  // bits a sample
        data.append(contentsOf: Array("data".utf8))
        put(bytes)
        return data
    }

    func append(_ samples: [Int16]) {
        queue.sync {
            var copy = samples
            let data = copy.withUnsafeBufferPointer { Data(buffer: $0) }
            try? handle.seekToEnd()
            try? handle.write(contentsOf: data)
            frames &+= UInt32(samples.count)
        }
    }

    /// Rewrite the header to cover everything written so far.
    func checkpoint() {
        queue.sync {
            guard frames != declared else { return }
            let position = (try? handle.offset()) ?? 0
            try? handle.seek(toOffset: 0)
            try? handle.write(contentsOf: GrowingWave.header(frames: frames, rate: rate))
            try? handle.seek(toOffset: position)
            declared = frames
        }
    }

    var seconds: Double { Double(frames) / Double(rate) }

    func finish() {
        checkpoint()
        try? handle.close()
    }
}

enum RecorderError: Error {
    case cannotWrite(String)
    case coreAudio(String, OSStatus)
    case noMicrophone
}

// MARK: - Turning what Core Audio hands us into mono 16-bit

/// Peak level of a block, so the interface can show that sound is arriving.
/// A recording that is silently capturing nothing is the failure people discover
/// afterwards, which is the worst time to discover it.
func peak(_ samples: [Int16]) -> Float {
    var highest: Int16 = 0
    for sample in samples where abs(Int32(sample)) > abs(Int32(highest)) {
        highest = sample
    }
    return abs(Float(highest)) / 32768.0
}

/// Interleaved or planar float channels, summed to one and clipped, because a
/// conference call is speech and speech does not need two of everything.
func monoSixteenBit(_ list: UnsafePointer<AudioBufferList>, channels: Int) -> [Int16] {
    let buffers = UnsafeMutableAudioBufferListPointer(
        UnsafeMutablePointer(mutating: list))
    guard let first = buffers.first, first.mData != nil else { return [] }
    let frames = Int(first.mDataByteSize) / MemoryLayout<Float>.size
        / max(1, Int(first.mNumberChannels))
    var out = [Int16](repeating: 0, count: frames)
    var counted = 0
    for buffer in buffers {
        guard let raw = buffer.mData else { continue }
        let stride = Int(buffer.mNumberChannels)
        let floats = raw.assumingMemoryBound(to: Float.self)
        let available = Int(buffer.mDataByteSize) / MemoryLayout<Float>.size
        for frame in 0..<frames {
            var sum: Float = 0
            for channel in 0..<stride {
                let at = frame * stride + channel
                if at < available { sum += floats[at] }
            }
            let scaled = Float(out[frame]) + sum * 32767.0 / Float(max(1, stride))
            out[frame] = Int16(max(-32768, min(32767, scaled)))
        }
        counted += 1
        if counted >= channels { break }
    }
    return out
}

// MARK: - System audio

/// The UID of the device the machine is currently playing through.
///
/// The aggregate needs one as its main sub-device. A tap on its own carries no
/// clock, and an aggregate without a clock runs but delivers silence - which is
/// indistinguishable from a meeting where nobody spoke.
func defaultOutputUID() throws -> String {
    var address = AudioObjectPropertyAddress(
        mSelector: kAudioHardwarePropertyDefaultOutputDevice,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMain)
    var device = AudioObjectID(kAudioObjectUnknown)
    var size = UInt32(MemoryLayout<AudioObjectID>.size)
    var status = AudioObjectGetPropertyData(
        AudioObjectID(kAudioObjectSystemObject), &address, 0, nil, &size, &device)
    guard status == noErr else {
        throw RecorderError.coreAudio("finding the output device", status)
    }
    address.mSelector = kAudioDevicePropertyDeviceUID
    var uid: CFString = "" as CFString
    size = UInt32(MemoryLayout<CFString>.size)
    status = AudioObjectGetPropertyData(device, &address, 0, nil, &size, &uid)
    guard status == noErr else {
        throw RecorderError.coreAudio("naming the output device", status)
    }
    return uid as String
}

/// A process tap over everything the machine is playing, wrapped in a private
/// aggregate device so an IO proc has something to attach to.
final class SystemAudio {
    private var tap = AudioObjectID(kAudioObjectUnknown)
    private var aggregate = AudioObjectID(kAudioObjectUnknown)
    private var proc: AudioDeviceIOProcID?
    private let wave: GrowingWave
    private var level: Float = 0
    private let lock = NSLock()

    init(wave: GrowingWave) throws {
        self.wave = wave

        let description = CATapDescription(stereoGlobalTapButExcludeProcesses: [])
        description.uuid = UUID()
        // Leave what the user is listening to audible. Muting the tap would record
        // the call while removing it from the room, which is not a trade anybody
        // asked for.
        description.muteBehavior = .unmuted
        description.name = "LocaLog meeting recorder"
        description.isPrivate = true

        var status = AudioHardwareCreateProcessTap(description, &tap)
        guard status == noErr else {
            throw RecorderError.coreAudio("creating the system-audio tap", status)
        }

        let uid = UUID().uuidString
        let settings: [String: Any] = [
            kAudioAggregateDeviceNameKey: "LocaLog meeting recorder",
            kAudioAggregateDeviceUIDKey: uid,
            // The output device supplies the clock the tap does not have.
            kAudioAggregateDeviceMainSubDeviceKey: try defaultOutputUID(),
            // Private, so this never appears in the user's sound settings.
            kAudioAggregateDeviceIsPrivateKey: true,
            kAudioAggregateDeviceIsStackedKey: false,
            kAudioAggregateDeviceTapAutoStartKey: true,
            kAudioAggregateDeviceSubDeviceListKey: [[String: Any]](),
            kAudioAggregateDeviceTapListKey: [
                [
                    kAudioSubTapUIDKey: description.uuid.uuidString,
                    kAudioSubTapDriftCompensationKey: true,
                ]
            ],
        ]
        status = AudioHardwareCreateAggregateDevice(settings as CFDictionary, &aggregate)
        guard status == noErr else {
            throw RecorderError.coreAudio("creating the recording device", status)
        }
    }

    func start() throws {
        let channels = try tapChannels()
        FileHandle.standardError.write(
            Data("tap \(tap), aggregate \(aggregate), \(channels) channel(s)\n".utf8))
        var described = false
        let status = AudioDeviceCreateIOProcIDWithBlock(&proc, aggregate, nil) {
            [weak self] _, input, _, _, _ in
            guard let self else { return }
            if !described {
                described = true
                let list = UnsafeMutableAudioBufferListPointer(
                    UnsafeMutablePointer(mutating: input))
                var note = "first callback: \(list.count) buffer(s)"
                for buffer in list {
                    note += " [\(buffer.mNumberChannels)ch \(buffer.mDataByteSize)B]"
                }
                FileHandle.standardError.write(Data((note + "\n").utf8))
            }
            let samples = monoSixteenBit(input, channels: channels)
            guard !samples.isEmpty else { return }
            self.wave.append(samples)
            let loudest = peak(samples)
            self.lock.lock()
            self.level = max(self.level, loudest)
            self.lock.unlock()
        }
        guard status == noErr, let proc else {
            throw RecorderError.coreAudio("attaching to the recording device", status)
        }
        let started = AudioDeviceStart(aggregate, proc)
        guard started == noErr else {
            throw RecorderError.coreAudio("starting system audio", started)
        }
    }

    private func tapChannels() throws -> Int {
        var address = AudioObjectPropertyAddress(
            mSelector: kAudioTapPropertyFormat,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain)
        var format = AudioStreamBasicDescription()
        var size = UInt32(MemoryLayout<AudioStreamBasicDescription>.size)
        let status = AudioObjectGetPropertyData(tap, &address, 0, nil, &size, &format)
        guard status == noErr else {
            throw RecorderError.coreAudio("reading the tap's format", status)
        }
        return Int(format.mChannelsPerFrame)
    }

    /// The loudest thing since this was last asked, then reset.
    func takeLevel() -> Float {
        lock.lock()
        defer {
            level = 0
            lock.unlock()
        }
        return level
    }

    func stop() {
        if let proc {
            AudioDeviceStop(aggregate, proc)
            AudioDeviceDestroyIOProcID(aggregate, proc)
        }
        if aggregate != kAudioObjectUnknown { AudioHardwareDestroyAggregateDevice(aggregate) }
        if tap != kAudioObjectUnknown { AudioHardwareDestroyProcessTap(tap) }
    }
}

// MARK: - Microphone

final class Microphone {
    private let engine = AVAudioEngine()
    private let wave: GrowingWave
    private var level: Float = 0
    private let lock = NSLock()
    private let rate: Double

    init(wave: GrowingWave, sampleRate: Double) {
        self.wave = wave
        self.rate = sampleRate
    }

    func start() throws {
        let input = engine.inputNode
        let native = input.outputFormat(forBus: 0)
        guard native.sampleRate > 0 else { throw RecorderError.noMicrophone }
        guard
            let wanted = AVAudioFormat(
                commonFormat: .pcmFormatFloat32, sampleRate: rate, channels: 1,
                interleaved: false),
            let converter = AVAudioConverter(from: native, to: wanted)
        else { throw RecorderError.noMicrophone }

        input.installTap(onBus: 0, bufferSize: 4096, format: native) { [weak self] buffer, _ in
            guard let self else { return }
            let capacity = AVAudioFrameCount(
                Double(buffer.frameLength) * self.rate / native.sampleRate + 1024)
            guard let converted = AVAudioPCMBuffer(pcmFormat: wanted, frameCapacity: capacity)
            else { return }
            var supplied = false
            var error: NSError?
            converter.convert(to: converted, error: &error) { _, status in
                if supplied {
                    status.pointee = .noDataNow
                    return nil
                }
                supplied = true
                status.pointee = .haveData
                return buffer
            }
            guard error == nil, let channel = converted.floatChannelData?[0] else { return }
            var samples = [Int16](repeating: 0, count: Int(converted.frameLength))
            for index in 0..<Int(converted.frameLength) {
                samples[index] = Int16(max(-32768, min(32767, channel[index] * 32767.0)))
            }
            self.wave.append(samples)
            let loudest = peak(samples)
            self.lock.lock()
            self.level = max(self.level, loudest)
            self.lock.unlock()
        }
        try engine.start()
    }

    func takeLevel() -> Float {
        lock.lock()
        defer {
            level = 0
            lock.unlock()
        }
        return level
    }

    func stop() {
        engine.inputNode.removeTap(onBus: 0)
        engine.stop()
    }
}

// MARK: - Running

let arguments = CommandLine.arguments
func option(_ name: String) -> String? {
    guard let at = arguments.firstIndex(of: name), at + 1 < arguments.count else { return nil }
    return arguments[at + 1]
}

guard let systemPath = option("--system"), let microphonePath = option("--microphone") else {
    FileHandle.standardError.write(
        Data("usage: record-meeting --system <path.wav> --microphone <path.wav>\n".utf8))
    exit(2)
}
// Long enough for speech to be worth keeping, and the rate the microphone and the
// tap both prefer, so nothing is resampled twice.
let sampleRate: UInt32 = 48_000

func fail(_ message: String) -> Never {
    FileHandle.standardError.write(Data((message + "\n").utf8))
    exit(1)
}

let systemWave: GrowingWave
let microphoneWave: GrowingWave
do {
    systemWave = try GrowingWave(path: systemPath, sampleRate: sampleRate)
    microphoneWave = try GrowingWave(path: microphonePath, sampleRate: sampleRate)
} catch {
    fail("Could not open the recording files: \(error)")
}

var systemAudio: SystemAudio?
var systemNote = ""
do {
    let audio = try SystemAudio(wave: systemWave)
    try audio.start()
    systemAudio = audio
} catch RecorderError.coreAudio(let what, let status) {
    // A refused tap is a permission answer, not a crash. The microphone alone is
    // still a recording, and losing the room to a failure of the call audio would
    // be the worse outcome.
    systemNote = "system audio unavailable while \(what) (status \(status))"
} catch {
    systemNote = "system audio unavailable: \(error)"
}

let microphone = Microphone(wave: microphoneWave, sampleRate: Double(sampleRate))
var microphoneNote = ""
do {
    try microphone.start()
} catch {
    microphoneNote = "microphone unavailable: \(error)"
}

if !systemNote.isEmpty { FileHandle.standardError.write(Data((systemNote + "\n").utf8)) }
if !microphoneNote.isEmpty { FileHandle.standardError.write(Data((microphoneNote + "\n").utf8)) }
if systemAudio == nil && !microphoneNote.isEmpty {
    fail("Neither source could be recorded.")
}

let running = DispatchSemaphore(value: 0)
for signal in [SIGINT, SIGTERM] {
    let source = DispatchSource.makeSignalSource(signal: signal, queue: .main)
    source.setEventHandler { running.signal() }
    source.resume()
    Darwin.signal(signal, SIG_IGN)
}

// One line a second: the levels a display is drawn from, and the checkpoint that
// makes the files survive being killed between them.
let ticker = DispatchSource.makeTimerSource(queue: .global())
ticker.schedule(deadline: .now() + 1, repeating: 1)
ticker.setEventHandler {
    systemWave.checkpoint()
    microphoneWave.checkpoint()
    let line = """
        {"seconds":\(Int(microphoneWave.seconds.rounded())),\
        "systemPeak":\(String(format: "%.3f", systemAudio?.takeLevel() ?? 0)),\
        "microphonePeak":\(String(format: "%.3f", microphone.takeLevel()))}
        """
    FileHandle.standardOutput.write(Data((line + "\n").utf8))
}
ticker.resume()

running.wait()
ticker.cancel()
microphone.stop()
systemAudio?.stop()
microphoneWave.finish()
systemWave.finish()
FileHandle.standardError.write(
    Data("stopped after \(Int(microphoneWave.seconds)) s\n".utf8))
