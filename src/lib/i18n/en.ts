/**
 * Every word the application says, in English.
 *
 * This file is the source of truth for *what strings exist*. Every other
 * language is typed against it, so a key that is missing or misspelled in a
 * translation is a compile error rather than an English word appearing in the
 * middle of a German sentence at run time.
 *
 * ## Why a plain object and not a library
 *
 * An i18n library brings a parser, a plural engine, a message format and a
 * dependency, to solve a problem this application does not have: the strings are
 * known at build time, there are no user-supplied templates, and the only thing
 * genuinely needed is "the same key, in another language, checked". A typed
 * object does that, and a value that needs a number in the middle of it is a
 * function — which TypeScript checks the arguments of, which no message-format
 * string can.
 *
 * ## Two rules for anything added here
 *
 * **Write the sentence, not the fragment.** A key holding "Could not" and
 * another holding "save the file" cannot be translated: languages put the parts
 * in different orders, and some need to know the gender of the noun to write the
 * verb. Every value here is a whole thing somebody reads.
 *
 * **Say what it is for, where it is not obvious.** A translator — including a
 * later reader of this file — needs to know that `preparing` is a button that
 * becomes a label, or that `silentSystem` is shown *during* a meeting and has to
 * be read at a glance.
 */

/** Text that comes back from the Rust side as a code rather than a sentence. */
const failures = {
  // Storage: something referred to is not there any more.
  missingProject: 'The selected project no longer exists.',
  missingMeeting: 'The selected meeting no longer exists.',
  missingJob: 'The import job is no longer available.',
  importBusy: 'Another recording is already being imported. Finish or cancel it first.',
  unsupportedSchema: (version: string) =>
    `This LocaLog data was created by a newer, unsupported version (${version}).`,
  storageUnavailable: 'LocaLog could not access its local workspace storage.',

  // Protocol styles.
  styleMissing: 'That style no longer exists.',
  styleNameRequired: 'Give the style a name.',
  styleNotSaved: 'The style could not be saved.',
  styleUnavailable: 'The selected protocol style is unavailable.',
  styleUsedByMeeting: 'A meeting is using this style. Change those meetings first.',
  styleUsedByProject: 'A project uses this style by default. Change that first.',

  // Appearance presets. These said "template" until the concept was renamed, and
  // three of them still did afterwards — which is the sort of thing a dictionary
  // makes visible and scattered string literals do not.
  presetNameRequired: 'Give the preset a name.',
  presetNotSaved: 'The preset could not be saved.',
  presetBuiltInUndeletable: 'A preset that shipped with LocaLog cannot be removed.',

  // Transcripts.
  transcriptInvalid: 'The saved transcript is invalid.',
  transcriptSegmentMissing: 'The transcript segment no longer exists.',
  transcriptTextRequired: 'Enter valid transcript text.',
  transcriptNeedsSegment: 'A transcript needs at least one segment.',
  transcriptSpeakerRequired: 'Enter a valid speaker label.',
  transcriptNotSaved: 'The transcript could not be saved.',
  transcriptNotCommitted: 'The transcript could not be committed.',
  spellingRequired: 'Enter a valid spelling.',

  // Protocols.
  protocolTextRequired: 'Enter valid protocol text.',
  protocolRevisionMissing: 'The selected protocol revision no longer exists.',
  protocolNeededBeforeExport: 'Generate a protocol before exporting it.',
  protocolNeededBeforeSetAside: 'Generate a protocol before setting a section aside.',
  sectionNotSetAside: 'The section could not be set aside.',
  reviewBeforeGeneration: 'Review the transcript before generation.',
  vocabularyUnresolved: 'The vocabulary could not be resolved.',

  // Rewriting a passage, which needs the provider running.
  selectionRequired: 'Select some text to change.',
  selectionTooLong:
    'That is too much text to change at once. Select a section rather than the document.',
  passageNotRewritten: 'That passage could not be rewritten.',
  openingNotRead: "The meeting's opening could not be read.",
  providerNeededForPassage: 'Start your existing Ollama installation before changing a passage.',
  providerNeededForOpening:
    'Start your existing Ollama installation before reading the introductions.',
  providerModelRequired: 'Choose an installed Ollama model in Settings → Protocol generation.',

  // Validation and storage failures found while writing the codes down.
  styleNotMigrated: 'A style could not be migrated.',
  termMissing: 'That term no longer exists.',
  exportFormatInvalid: 'Choose a valid export format.',
  meetingDateInvalid: 'Choose a valid meeting date.',
  scopeInvalid: 'Choose a valid scope.',
  sourceFileInvalid: 'Choose a valid source file.',
  workspaceViewInvalid: 'Choose a valid workspace view.',
  recordingUnreadable: 'That recording could not be read.',
  appearanceNotSaved: 'The appearance could not be stored.',
  furnitureNotSaved: 'The header and footer could not be stored.',
  documentOperationFailed: 'The local document operation could not finish.',
  providerConfigNotSaved: 'The protocol provider configuration could not be saved.',
  runtimeConfigNotSaved: 'The transcription runtime configuration could not be saved.',
  recorderNotStarted: 'The recorder could not be started.',
  tracksNotCombined: "The recording's tracks could not be combined.",
  protocolInvalid: 'The saved protocol is invalid.',
  protocolNotUtf8: 'The saved protocol is not valid UTF-8.',
  editsNotRecorded: 'Those edits cannot be recorded.',

  // Failures the command layer itself reports.
  recordingAlreadyRunning: 'A meeting is already being recorded.',
  presetUnknown: 'Choose a known transcription quality.',
  providerModelNotInstalled: 'Choose a model that is already installed in Ollama.',
  diariserPathInvalid: 'Choose an existing speaker separation program.',
  whisperPathInvalid: 'Choose an existing whisper.cpp executable.',
  nothingRecording: 'Nothing is being recorded.',
  revealOnlyOnMac: 'Opening the folder is only wired up on macOS. The path above is correct.',
  privacySettingsOnlyOnMac: 'Opening the privacy settings is only wired up on macOS.',
  providerNeededForModel: 'Start your existing Ollama installation before selecting a model.',
  settingsNotOpened: 'System Settings could not be opened.',
  presetMissing: 'That export template is no longer available.',
  downloadStopped: 'The download stopped unexpectedly.',
  coordinatorUnavailable: 'The import coordinator is unavailable.',
  taskStopped: 'The local cancellation task stopped unexpectedly.',
  recorderPermissionsUnknown: 'The recorder could not be asked about permissions.',
  recorderStateUnknown: 'The recorder is in an unknown state. Restart LocaLog.',
  recordingNotFinished: 'The recording could not be finished.',
  replacementNotPrepared: 'The replacement could not be prepared.',
  workspaceNotOpened: 'The workspace folder could not be opened.',
  settingsPaneUnknown: 'There is no such settings pane.',
  meetingBusy: 'This meeting is still being worked on. Cancel that first.',
  printDialogUnavailable: 'This window could not open the print dialog.',

  // Backing up and restoring.
  backupNameUnsafe: 'That backup name cannot be used as a folder name.',
  notABackup: 'That folder is not a LocaLog backup: it has no manifest.json.',
  backupPathOutside: (path: string) =>
    `This backup lists a file outside its own folder (${path}), so it was not restored.`,
  backupFormatUnknown: (format: string) =>
    `This backup was written in format ${format}, which this version of LocaLog does not know how to read. A newer LocaLog will.`,
  backupDamaged: (what: string) =>
    `This backup is incomplete or damaged (${what}), so nothing was changed. Your current work is untouched.`,
  backupNameTaken: (name: string) => `There is already something called “${name}” in that folder.`,
  backupIoFailed: (what: string) => `The backup could not be written or read: ${what}`,
  backupDatabaseFailed: (what: string) => `The database could not be copied: ${what}`,

  // The pipeline: audio, models, the local provider, the recorder.
  embeddingsUnrecognisable: 'The speaker pass did not write recognisable embeddings.',
  embeddingsNoDimensions: 'The embeddings describe no dimensions.',
  embeddingsTruncated: 'The embeddings are shorter than they claim to be.',
  probeInvalid: 'The media probe returned invalid metadata.',
  cachePathInvalid: 'The normalized cache path is invalid.',
  normalizerNoOutput: 'The media normalizer did not produce an audio file.',
  speakerPassNoAudio: 'There is nothing for the speaker pass to listen to.',
  speakerPassTooMuchAudio: 'The speaker pass planned more audio than can be held.',
  recordingEmpty: 'The recording was stored as an empty file.',
  editsLeaveNothing: 'These edits would leave no recording at all.',
  workingAudioUnreadable: 'The working audio is not a readable WAV file.',
  workingAudioNotWav: 'The working audio is not a WAV file.',
  workingAudioSilent: 'The working audio has no audio in it.',
  workingAudioFormatUnreadable: 'The working audio has an unreadable format.',
  workingAudioNoFormat: 'The working audio describes no format.',
  condensedAudioTooLarge: 'The condensed audio is too large to write.',
  combinedPathInvalid: 'The combined recording path is invalid.',
  modelUnknown: 'That transcription model is not recognised.',
  downloadCancelled: 'The download was cancelled.',
  downloadCorrupt: 'The download was incomplete or corrupt and was discarded.',
  ollamaModelGone:
    'The selected Ollama model is no longer installed. Choose another model and retry.',
  ollamaModelChanged:
    'The selected Ollama model changed after this job was queued. Retry to resolve it again.',
  ollamaRuntimeChanged:
    'The Ollama runtime changed after this job was queued. Retry to resolve it again.',
  responseTooLarge: 'The local model response exceeded the safe limit and was not committed.',
  responseIncomplete: 'The local model stopped before returning a complete protocol.',
  recorderMissing: 'No recorder is installed. LocaLog ships one; this build cannot find it.',
  recorderSilentAboutPermissions: 'The recorder did not say what it is allowed to do.',
  recorderCannotReportPermissions: 'This recorder cannot report what it is allowed to do.',
  runtimePathsMustBeAbsolute: 'Choose absolute paths for the whisper.cpp executable and model.',
  whisperExecutableMissing: 'The selected whisper.cpp executable was not found.',
  whisperModelMissing: 'The selected whisper.cpp model was not found.',
  embeddingsVersion: (version: string) =>
    `These embeddings are version ${version}, which this build does not read.`,
  recordingTooSmall: (what: string) =>
    `The stored recording is too small for its length (${what}).`,
  workingAudioFormatWrong: (what: string) =>
    `The speaker pass needs 16 kHz mono 16-bit audio, and this is ${what}.`,
  notEnoughSpace: (what: string) => `Not enough space for this model (${what}).`,
};

export const en = {
  failures,

  /** The screen somebody sees before there is anything to open. */
  /** The settings screen. */
  settings: {
    interfaceLanguage: 'Interface language',
    interfaceLanguageDetail:
      'What LocaLog itself is written in. Separate from the language of each meeting.',
  },

  start: {
    eyebrow: 'Local AI for private meeting protocols',
    title: 'Start a meeting',
    lead: 'Import an audio or video file. Review every step before it becomes a protocol.',
    importTitle: 'Import recording',
    importDetail: 'Choose a project, then keep every artifact in context',
    recordTitle: 'Record a meeting',
    recordDetail: 'Capture the room and the call on this device, on separate tracks',
    promiseTitle: 'Your meeting work stays on this device.',
    promiseDetail: 'No LocaLog account, cloud service, or telemetry.',

    /** Shown only on an installation that has no transcription model yet. */
    setupTitle: 'One download before the first transcription',
    /** `quality` is a preset name — Fast, Balanced, Accurate — and `size` reads "141 MB". */
    setupBody: (quality: string, size: string) =>
      `LocaLog transcribes on this device, so the model has to be on it. ${quality} quality is ${size}, downloaded once. You can import a recording first — this is needed when transcription starts, not before.`,
    setupDownload: (size: string) => `Download it now (${size})`,
    setupCancel: 'Cancel',
    setupAside: 'Other qualities, and speaker separation, are in Settings.',
  },
};

/**
 * The shape every other language must have.
 *
 * Derived from English rather than declared separately, so adding a key here is
 * the only thing needed to require it everywhere.
 */
export type Strings = typeof en;
