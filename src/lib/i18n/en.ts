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

  // Validation the storage layer does, and failures that carry the reason.
  categoryRequired: 'Choose a category.',
  meetingLanguageRequired: 'Choose a meeting language.',
  meetingLanguageInvalid: 'Choose a valid meeting language.',
  meetingInvalid: 'Choose a valid meeting.',
  projectInvalid: 'Choose a valid project.',
  styleInvalid: 'Choose a valid protocol style.',
  sourceRecordingInvalid: 'Choose a valid source recording.',
  meetingTitleRequired: 'Enter a meeting title.',
  projectNameRequired: 'Enter a project name.',
  termRequired: 'Enter a term.',
  meetingTitleTooLong: 'The meeting title is too long.',
  speakerPassCannotRead: (what: string) =>
    `The speaker pass could not read the working audio: ${what}`,
  speakerPassCannotWrite: (what: string) => `The speaker pass could not write its audio: ${what}`,
  recordingNotStored: (what: string) => `The recording could not be stored: ${what}`,
  recordingNotRead: (what: string) => `The recording could not be read: ${what}`,
  modelNotDownloaded: (what: string) => `The model could not be downloaded: ${what}`,
  modelNotSaved: (what: string) => `The model could not be saved: ${what}`,
  ollamaRequestFailed: (what: string) => `Ollama could not complete the local request: ${what}`,
  recorderStartFailed: (what: string) => `The recorder could not be started: ${what}`,

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
  /** `expected/ceiling`, both in characters, straight from the arithmetic. */
  protocolWouldNotFit: (sizes: string) => {
    const [expected, ceiling] = sizes.split('/').map((n) => Number(n).toLocaleString('en-GB'));
    return `This meeting is long enough that a protocol of it — roughly ${expected} characters — would not fit in one answer, which holds about ${ceiling}. Nothing was attempted, because this is arithmetic rather than a bad run and retrying would fail the same way. Choose a terser protocol style, or split the recording.`;
  },
  generationConfigUnreadable:
    'This job was prepared by an earlier version of LocaLog and cannot be read. Nothing was committed and your transcript is unchanged. Start the generation again.',
  responseUnusable:
    'The local model returned an answer LocaLog could not use as a protocol. Nothing was committed and your transcript is unchanged. Retrying often succeeds, because a model answers differently each time.',
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
  /**
   * What `Intl` should be told, for dates and numbers. On the dictionary rather
   * than derived from the language id, because `en` alone gives American date
   * order and this product's English is British.
   */
  locale: 'en-GB',

  failures,

  /** The screen somebody sees before there is anything to open. */
  /** The settings screen. */
  settings: {
    catalogueNote:
      'The catalogue is intentionally curated. LocaLog does not silently download models or present an arbitrary model marketplace. New entries become selectable only after their runtime, licence, memory use and German/English quality have been checked.',
    managedCopiesNote:
      'LocaLog keeps managed copies of imported recordings, prepared audio, transcripts, protocols and downloaded models in its application-data folder. Exports are written only to the location you choose.',
    discoveredRuntime: (path: string) => `Discovered runtime: ${path}`,
    runtimeVersion: (version: string) => `Runtime version: ${version}`,
    evaluatedIn: (languages: string) => `Evaluated in ${languages}`,
    evaluationPending: 'Quality evaluation still pending',
    otherModelNote:
      'This is for people who already know which local model they want to try. It is not evaluated or recommended by LocaLog, and it remains subject to the same local runtime and memory limits.',
    qualityLead:
      'Choose the quality you want. LocaLog downloads what it needs the first time and keeps it on this device.',
    speakerDiscovery:
      'LocaLog discovers the speaker runtime automatically from its bundled resources or the system path. The runtime is optional and never blocks transcription.',
    noSpeakerRuntime: 'No compatible speaker runtime was found on this machine yet.',
    readinessNote:
      'Readiness includes a bounded launch check, so an incompatible or broken executable is not presented as available.',
    restoreSummary: (name: string, projects: number, meetings: number, version: string) =>
      `${name} holds ${projects} projects and ${meetings} meetings, backed up from LocaLog ${version}.`,
    restoreWarning:
      'Restoring replaces the projects and meetings in this workspace with those. Nothing is deleted — what is here is kept in a folder beside it — but LocaLog will be showing the restored work, and you will need to quit and open it again.',
    interfaceLanguage: 'Interface language',
    interfaceLanguageDetail:
      'What LocaLog itself is written in. Separate from the language of each meeting.',
    application: 'Application',
    title: 'Settings',
    lead: 'Professional defaults first. Runtime details stay progressively disclosed.',
    sectionsLabel: 'Settings sections',
    sectionGeneral: 'General',
    sectionModels: 'Models',
    sectionTranscription: 'Transcription',
    sectionStorage: 'Storage',
    sectionAppearance: 'Appearance',
    sectionAdvanced: 'Advanced',
    defaultExport: 'Default export',
    defaultExportDetail:
      'Which format the protocol editor offers first. The others stay one click away.',
    defaultExportLabel: 'Default export format',
    formatPdf: 'PDF',
    formatWord: 'Word',
    formatMarkdown: 'Markdown',
    formatPlainText: 'Plain text',
    defaultForProtocols: 'Default for protocols',
    chooseOnce: 'Choose once, then keep working',
    modelLead:
      'LocaLog uses this model for local protocol drafts until you change it. The normal workflow does not ask you to choose a model for every meeting.',
    recommendedForMachine: 'Recommended for this machine',
    notInstalledYet: 'Not installed yet',
    baseline: 'Baseline',
    european: 'European',
    checkInstalled: 'Check installed models',
    curatedModels: 'Curated protocol models',
    useAnotherModel: 'Use another installed model',
    installedModel: 'Installed model',
    chooseInstalledModel: 'Choose an installed model',
    useInstalledModel: 'Use installed model',
    conservativeBaseline: 'Using the conservative 8 GB baseline',
    transcriptionQuality: 'Transcription quality',
    cancel: 'Cancel',
    ready: 'Ready',
    remove: 'Remove',
    advancedDetails: 'Advanced details',
    modelsStoredNote:
      'Models are stored in LocaLog’s application data folder and verified before use.',
    whisperExecutable: 'whisper-cli executable',
    chooseFile: 'Choose file',
    whisperNote: 'Choose the command-line transcription binary, not whisper-server.',
    saveRuntime: 'Save runtime',
    detected: (version: string) => `Detected: ${version}`,
    chooseWhisper: 'Choose whisper-cli executable',
    speakerDifferentiation: 'Speaker differentiation',
    speakerLead:
      'Optional voice-turn separation labels who spoke when. It never blocks a transcript, and labels remain editable during review.',
    runtimeUnavailable: 'Runtime not available in this installation',
    optional: 'Optional',
    checkReadiness: 'Check readiness',
    downloadingSpeakerModels: 'Downloading speaker separation models',
    speakerRuntimeMissing:
      'The models are prepared, but this installation has no compatible speaker runtime.',
    whereWorkIsKept: 'Where your work is kept',
    workspaceNote:
      'LocaLog manages this folder so that paths inside it stay valid, but it is yours and you can look in it whenever you like.',
    showInFinder: 'Show in Finder',
    backup: 'Backup',
    backupLead:
      'Everything stays on this device, which also means it leaves with the device. A backup is an ordinary folder you can put on a drive or wherever you keep things safe.',
    backUpNow: 'Back up now',
    working: 'Working…',
    backupContents:
      'Holds every project, meeting, transcript and protocol, and the recordings themselves. Two things are left out on purpose, because neither is your work and both are rebuilt when they are needed: downloaded models, and the prepared copy of each recording. Measured on a real workspace, that prepared audio alone was three quarters of the backup.',
    restore: 'Restore',
    restoreLead:
      'Puts a backup back. It is checked in full first, and what is here now is moved aside rather than deleted.',
    chooseBackup: 'Choose a backup…',
    chooseBackupTitle: 'Choose a LocaLog backup',
    whereToKeepBackup: 'Where to keep the backup',
    replaceWorkspace: 'Replace this workspace',
    restoring: 'Restoring…',
    archived: 'Archived',
    archivedLead:
      'Projects and meetings put out of the way. Nothing was deleted: every meeting, transcript and protocol under them is still here, and still in every backup.',
    show: 'Show',
    hide: 'Hide',
    nothingArchived: 'Nothing has been archived.',
    project: 'Project',
    meeting: 'Meeting',
    bringBack: 'Bring back',
    theme: 'Theme',
    themeFollowing: (theme: string) => `Following this Mac, which is set to ${theme}.`,
    themeSetHere: 'Set here, whatever this Mac is set to.',
    nextFakeJob: 'Next fake job',
    nextFakeJobDetail: 'Development-only control for reviewing failure and retry states.',
    completeNormally: 'Complete normally',
    failOnce: 'Fail once, then allow retry',
    syntheticNote: 'This affects only the in-memory synthetic runtime.',
  },

  /** The sidebar, which is on screen whatever else is. */
  project: {
    deleteWarning:
      'Deleting a meeting removes its recording, its transcript and every protocol revision, from this device. It cannot be undone.',
    eyebrow: 'Project',
    archiveProject: 'Archive project',
    newMeeting: 'New meeting',
    meetings: 'Meetings',
    newestFirst: 'Newest first',
    columnDate: 'Date',
    columnMeeting: 'Meeting',
    columnDuration: 'Duration',
    columnStatus: 'Status',
    archive: 'Archive',
    delete: 'Delete',
    keep: 'Keep',
    noMeetings: 'No meetings yet',
    noMeetingsDetail: 'Import the first recording to begin this project’s meeting record.',
    importRecording: 'Import recording',
  },

  lifecycle: {
    draft: 'Draft',
    sourceReady: 'Ready to transcribe',
    transcriptReady: 'Transcript ready',
    protocolDraft: 'Protocol draft',
    reviewed: 'Reviewed',
    archived: 'Archived',
  },

  sections: {
    noHeadings: 'This protocol has no headings yet, so there is nothing to list.',
    setAside: 'Set aside',
    addSection: 'Add section',
    dragHint: 'Drag, or use the arrow keys',
    setThisAside: 'Set this section aside',
    putThisBack: 'Put this section back',
    moveSection: (title: string) => `Move ${title}. Use the arrow keys.`,
    setAsideNamed: (title: string) => `Set aside ${title}`,
    putBackNamed: (title: string) => `Put back ${title}`,
    setAsideNote:
      'A section set aside leaves the document, so what you read is still exactly what is exported. It is kept here and can be put back.',
  },

  stages: {
    label: 'Meeting stages',
    source: 'Source',
    transcript: 'Transcript',
    protocol: 'Protocol',
  },

  progress: {
    speakerPassNote:
      'This pass reads the full recording to compare voice turns. Long recordings can take a few minutes; you can cancel safely at any time.',
    latestRetained: 'Latest stable work retained',
    originalUnchanged: ' · external original unchanged',
    retry: 'Retry',
    importing: 'Importing recording',
    transcribing: 'Transcribing',
    generating: 'Generating protocol',
    separatingSpeakers: 'Separating speakers',
    working: 'Working…',
    duplicateNote:
      'The same content is already stored in LocaLog. Nothing has been merged or discarded.',
    cancelImport: 'Cancel import',
    importAnotherCopy: 'Import another copy',
    chooseSourceAgain: 'Choose source again',
    continueImport: 'Continue import',
    transcribeAgain: 'Start transcription again',
    generateAgain: 'Start generation again',
  },

  newProject: {
    afterCreated:
      'A protocol style, and the names and terms this work uses, can be set for the project after it is created. The names are worth a minute: they are what transcription cannot guess.',
    eyebrow: 'Projects',
    title: 'New project',
    lead: 'Create the professional context that meetings and sources belong to.',
    defaults: 'Project defaults',
    name: 'Project name',
    namePlaceholder: 'e.g. Community hall study',
    description: 'Description',
    descriptionOptional: 'optional',
    descriptionPlaceholder: 'A concise internal description',
    defaultLanguage: 'Default meeting language',
    defaultLanguageDetail: 'Independent from the application interface language.',
    cancel: 'Cancel',
  },

  appearance: {
    appliesToProject: (project: string) =>
      `Applies to every protocol in ${project}, so a firm's documents look alike. It changes how the protocol is set, never what it says — that is the style above.`,
    bodySize: 'Body size',
    headingScale: 'Heading scale',
    lineSpacing: 'Line spacing',
    pageWidth: 'Page width',
  },

  record: {
    recordingNow: 'Recording',
    recordThisMeeting: 'Record this meeting',
    lead: 'The room and the call are captured on separate tracks, on this device. Whether the people in the meeting have agreed to be recorded is yours to settle, not something LocaLog can know.',
    notRecording: 'Not recording',
    microphone: 'Microphone',
    theCall: 'The call',
    trackRecording: 'Recording',
    trackSilent: 'Silent so far',
    trackListening: 'Listening…',
    stopRecording: 'Stop recording',
    finishing: 'Finishing…',
    startRecording: 'Start recording',
    starting: 'Starting…',
    backToMeeting: 'Back to the meeting',
    noRecorder: 'This build has no recorder. Import a file instead.',
    openTheSetting: 'Open the setting',
    grantedInSettings: 'Granted in System Settings, and picked up here as soon as you come back.',
    callWouldNotRecordTitle: 'The call would not be recorded.',
    callWouldNotRecordBody:
      'macOS has not granted LocaLog Screen & System Audio Recording, and without it a recording of the call is silence rather than an error — so this is worth granting now rather than discovering afterwards. The microphone in the room would still be captured.',
    roomWouldNotRecordTitle: 'The room would not be recorded.',
    roomWouldNotRecordBody:
      'LocaLog has been refused the microphone. The call would still be captured if the setting above allows it.',
    recorderNotesTitle: 'The recorder could not do everything it was asked.',
    stoppedOnItsOwn:
      'The recorder stopped on its own. Whatever it captured up to that point has been kept.',
    quietCall: (seconds: number) =>
      `Nothing has arrived from the call in ${seconds} seconds. macOS gives an application silence rather than an error when it has not been granted Screen & System Audio Recording, so this is worth checking now rather than after the meeting.`,
    quietMicrophone: (seconds: number) =>
      `Nothing has arrived from the microphone in ${seconds} seconds. Check that the right input is selected and that nothing else is holding it.`,
  },

  meeting: {
    importInterrupted:
      'LocaLog was closed before the managed copy was committed. The meeting remains in Draft and the import can be retried safely.',
    importCancelled:
      'The managed copy was cancelled. The meeting remains in Draft and the external file was not modified.',
    importFailed:
      'The managed copy could not be committed. The meeting remains in Draft and the external file was not modified.',
    importRunning:
      'LocaLog is copying this source into private managed storage. It will become ready only after the copy has been validated and committed.',
    sourceStored: 'is safely stored with this meeting. The external original was not modified.',
    sourceSynthetic:
      'is assigned to this synthetic browser meeting. No real media file was copied.',
    syntheticFixture: 'Synthetic fixture',
    eyebrow: 'Meeting',
    titleLabel: 'Meeting title',
    editTitle: 'Edit meeting title',
    languageLabel: 'Meeting language',
    changeLanguage: 'Change meeting language',
    save: 'Save',
    saveLanguage: 'Save language',
    cancel: 'Cancel',
    recordingEyebrow: 'Recording',
    nothingRecorded: 'Nothing recorded yet',
    recordLead:
      'The room and the call will be captured on separate tracks, on this device. You can stop whenever the meeting ends.',
    recordThisMeeting: 'Record this meeting',
    sourceImport: 'Source import',
    originalUnchanged: 'Your original remains unchanged',
    sourceReady: 'Source ready',
    readyToTranscribe: 'Ready to transcribe',
    managedSource: 'Managed source',
    language: 'Language',
    languageHint: 'Meeting setting · change above before transcribing',
    preset: 'Preset',
    globalDefault: 'Global default',
    notSelected: 'Not selected',
    peopleSpeaking: 'People speaking',
    doNotSeparate: 'Do not separate speakers',
    separateAndCount: 'Separate them, and work out how many',
    prepareSpeakers: 'Prepare speaker separation',
    prepareSpeakersDetail:
      'LocaLog needs two verified local model files before it can add provisional speaker labels. Your recording stays on this device.',
    preparing: (percent: number) => `Preparing ${percent}%`,
    prepare: 'Prepare',
    prepareWithSize: (size: string) => `Prepare (${size})`,
    speakerRuntimeMissing:
      'The speaker runtime is not available in this installation. Transcription can continue, but the transcript will use editable generic speaker labels.',
    reviewAndTrim: 'Review and trim the recording first',
    trimDetail:
      '— cut the wait before the meeting starts and anything it does not need. Your recording is never changed.',
    gettingReady: 'Getting ready to transcribe…',
    useJobControls: 'Use the job controls above',
    prepareSpeakersFirst: 'Prepare speaker separation first',
    transcribe: 'Transcribe',
    transcriptionFailedToStart: 'Transcription could not be started. Please try again.',
    transcriptReady: 'Transcript ready',
    reviewBeforeGeneration: 'Review before generation',
    transcriptReadyDetail:
      'The timestamped transcript is ready for corrections and manual speaker mapping.',
    reviewTranscript: 'Review transcript',
    protocolAvailable: 'Protocol available',
    continueInEditor: 'Continue in the document editor',
    protocolDetail: 'The transcript remains available alongside the current protocol revision.',
    openProtocol: 'Open protocol',
  },

  newMeeting: {
    noPerMeetingOverrides:
      'Per-meeting overrides and choosing names & terms per meeting are not available yet.',
    chosenOnceNote:
      'Transcription quality and the model that writes the protocol are chosen once, in Settings, and reused for every meeting.',
    titleRecording: 'Recording',
    titleImport: 'Structured import',
    heading: 'New meeting',
    leadRecording: 'Name the meeting and choose its project. Recording starts on the next screen.',
    leadImport: 'Choose the recording, confirm the details, and LocaLog takes it from there.',
    context: 'Context',
    chooseProject: 'Choose a project',
    project: 'Project',
    newProject: 'New project',
    noInbox:
      'Every source belongs to a meeting, and every meeting belongs to a project. There is no inbox.',
    source: 'Source',
    importRecording: 'Import recording',
    originalStays: 'Your original stays where it is',
    readyToCopy: 'Ready to copy after you confirm this meeting',
    letGoToImport: 'Let go to import',
    originalStaysShort: 'The original stays where it is.',
    dropHere: 'Drop a recording here, or click to choose one',
    dropDetail:
      'MP3, M4A, WAV, MP4, MOV and others. The original remains untouched — LocaLog copies it into its own storage.',
    readyToAssign: 'Ready to assign to this meeting',
    chooseFile: 'Choose an audio or video file',
    previewNote: 'The browser preview demonstrates the workflow without storing the file.',
    useDemoRecording: 'Use the synthetic demo recording',
    essentials: 'Essentials',
    meetingInformation: 'Meeting information',
    title: 'Title',
    titlePlaceholder: 'Derived from the file if left empty',
    date: 'Date',
    language: 'Meeting language',
    protocolStyle: 'Protocol style',
    projectDefault: 'Project default',
    qualityNote: 'Transcription quality is chosen once in Settings and applies to every meeting.',
    advanced: 'Advanced processing options',
    cancel: 'Cancel',
    createAndRecord: 'Create meeting and record',
    createAndImport: 'Create meeting and import',
  },

  recordingReview: {
    lead: 'Cut what the meeting does not need before it is transcribed. Your recording is never changed — everything here can be undone.',
    noPreparedAudio:
      'This meeting has no prepared audio to review. It becomes available once the import has been committed.',
    dragToSelect:
      'Drag across the recording to select a stretch, or use the arrow keys and hold shift.',
    selectedRange: (from: string, to: string) => `Selected ${from} to ${to}.`,
    eyebrow: 'Recording',
    heading: 'Review recording',
    noAudio: 'No working audio yet',
    waveformLabel: 'The recording. Move with the arrow keys, hold shift to select.',
    startHere: 'Start here',
    removeSelection: 'Remove selection',
    endHere: 'End here',
    edits: 'Edits',
    nothingRemoved: 'Nothing removed. The whole recording will be transcribed.',
    undo: 'Undo',
    putEverythingBack: 'Put everything back',
    untouchedNote: 'The recording itself is untouched. These are instructions for what to use.',
    undoStartTrim: 'Undo the start trim',
    undoEndTrim: 'Undo the end trim',
    putStretchBack: 'Put this stretch back',
    next: 'Next',
    continueToTranscription: 'Continue to transcription',
    backToMeeting: 'Back to the meeting',
  },

  transcript: {
    /** Beside a meeting's date, before its length has been worked out. */
    durationPending: 'Duration pending',
    introducedThemselves: (count: number) => `${count} introduced themselves`,
    noNamesYet: (project: string) => `No names yet for ${project}`,
    speltAsHeard:
      'Spelt as the transcriber heard them. Correct any that are wrong — they will be fixed here and remembered for this project.',
    openingNote:
      'Meetings usually open with people saying who they are. Reading that gives this project its names, which is what transcription cannot guess.',
    foundInPlaces: (count: number) =>
      `Found in ${count} ${count === 1 ? 'place' : 'places'}. Untick any that should stay as they are.`,
    noneMisheardEveryTime: (count: number) =>
      `No word was misheard every time it came up. ${count} passages are still flagged as unclear for other reasons.`,
    nothingFlaggedNote:
      'Nothing was flagged as unclear. A transcript made before this was recorded also shows nothing here, so an older one may be worth reading rather than trusting.',
    workingAudioLater: 'Working audio becomes available once this meeting has been transcribed.',
    recordingEndsNote:
      'If the meeting carried on past this, the recording did not capture it and the protocol will not contain it.',
    heading: 'Transcript review',
    exportTranscript: 'Export transcript…',
    exportLabel: 'Export this transcript',
    asMarkdown: 'As Markdown',
    asPlainText: 'As plain text',
    reviewDetails: 'Review details',
    sourceContext: 'Meeting source context',
    seekAudio: 'Seek audio',
    follow: 'Follow',
    followLabel: 'Scroll the transcript to the segment being played',
    searchTranscript: 'Search transcript',
    editableTranscript: 'Editable transcript',
    removeLine: 'Remove this line from the transcript',
    nothingFlagged: 'Nothing flagged as unclear',
    show: 'Show',
    showing: 'Showing',
    onePassage: '1 unclear passage',
    manyPassages: (count: number) => `${count} unclear passages`,
    speakerHint: 'Speaker labels are a starting point—rename them to the people who spoke.',
    generateProtocol: 'Generate protocol',
    review: 'Review',
    detailsLabel: 'Transcript review details',
    closeInspector: 'Close inspector',
    speakers: 'Speakers',
    whereRecordingStops: 'Where the recording stops',
    transcriptionInput: 'Transcription input',
    language: 'Language',
    meetingLanguage: 'Meeting language',
    saveLanguage: 'Save language',
    cancel: 'Cancel',
    changeLanguage: 'Change language',
    rerunNote:
      'Use this after changing the language or transcription settings. The new run is recorded as a separate revision.',
    rerun: 'Rerun transcription',
    rerunPreparing: 'Preparing a new transcript…',
    rerunConfirm: (language: string) =>
      `Rerun transcription in ${language}? The current transcript will stay until the new result is committed, then this working transcript will be replaced.`,
    whoIsHere: 'Who is in this meeting',
    close: 'Close',
    aboutAMinute: 'About a minute. Nothing else can run meanwhile.',
    unsureNames: 'Names the transcriber was unsure of',
    whatShouldItSay: 'What should it say?',
    rememberForProject: 'Remember for this project, so the next meeting spells it correctly',
    areAnyNames: 'Are any of these names? Correcting one repairs this transcript and remembers it.',
    nothingToCheck: 'Nothing to check',
    correctSpelling: 'Correct spelling',
    checkWording: 'Check wording',
    protocolStyle: 'Protocol style',
    audioUnplayable: 'This meeting’s working audio could not be played.',
    speakersResolved:
      'Speaker turns were resolved locally. Labels are provisional—rename them only when you know the participant.',
    speakersFailed:
      'Speaker separation did not produce usable turns for this run. The transcript is intact and uses neutral labels; you can continue with manual labels.',
    speakersUnavailable:
      'Speaker separation was not available for this run. The transcript is intact and uses a neutral label; you can still rename it manually.',
    speakersUnknown:
      'This older transcript does not record whether speaker separation ran. Its neutral labels are not evidence that there was only one speaker.',
  },

  library: {
    instructionsGiven:
      'These are the instructions the model is given, in the order it is given them',
    asShipped: ', exactly as this style shipped',
    invariantsNote:
      'These are not part of this style and cannot be edited here — they are not stored with a style at all. They are added to every protocol as it is written, because a document that reports a decision nobody made is not a differently-styled protocol but a wrong one.',
    whichTermsHelp:
      'Names, firms and abbreviations help most. Ordinary professional terminology is usually transcribed correctly without being listed.',
    termsLeadLong:
      'Add the names, firms and abbreviations this work uses so they are transcribed correctly. On a real eighty-minute meeting this took the project’s own name from never spelled correctly to always.',
    eyebrow: 'Library',
    protocolStyles: 'Protocol styles',
    namesAndTerms: 'Names & terms',
    stylesLead:
      'What a protocol says, and in what order. Not how it is set — that is the appearance, and it lives in the protocol editor beside the document it describes.',
    termsLead:
      'The names transcription cannot guess: your project, the firms, the people. Measured on a real meeting, these are worth more than any other setting here.',
    addTerm: 'Add term',
    saveTerm: 'Save term',
    stylesUnreadable: 'Styles cannot be read here.',
    length: 'Length',
    name: 'Name',
    description: 'Description',
    whatItAsksFor: 'What this style asks for',
    addInstruction: 'Add an instruction',
    removeInstruction: 'Remove this instruction',
    checkedOnProtocol: 'Checked on the finished protocol',
    alwaysEveryStyle: 'Always, in every style',
    saveStyle: 'Save style',
    cancel: 'Cancel',
    delete: 'Delete',
    editThisStyle: 'Edit this style',
    duplicate: 'Duplicate',
    duplicateToEdit: 'Duplicate to edit',
    shippedStyleNote:
      'A style that shipped stays as it is, so a protocol written last year can be written the same way again. Copy it to make your own.',
    ownershipAutomatic: 'Ownership is automatic.',
    termsScopeNote: 'A project’s names and terms apply to its meetings without repeated selection.',
    term: 'Term',
    spellingAsShown: 'Spelling as it should appear',
    category: 'Category',
    appliesTo: 'Applies to',
    everyProject: 'Every project',
    unknownProject: 'Unknown project',
    noTerms: 'No names or terms yet',
    deleteThisTerm: 'Delete this term?',
    densityFull: 'Full prose',
    densityPlain: 'Plain statements',
    densityLine: 'A line per point',
    densityFullMeaning: 'Full prose. A reader who was absent can follow the discussion.',
    densityPlainMeaning: 'Plain statements. What was said, without the retelling.',
    densityLineMeaning: 'A line per point. The record, and nothing around it.',
    categoryPerson: 'Person',
    categoryOrganisation: 'Organisation',
    categoryProject: 'Project',
    categoryAbbreviation: 'Abbreviation',
    categoryTechnicalTerm: 'Technical term',
    categoryOther: 'Other',
  },

  furniture: {
    header: 'Header',
    footer: 'Footer',
    left: 'Left',
    centre: 'Centre',
    right: 'Right',
    insert: 'Insert…',
    lineHint:
      'Type the line as it should read, and put a value into it where you want one — “Seite ”, the page number, “ von 12”. A value is one object: it selects and deletes whole.',
    appliesTo: (project: string) =>
      `Applies to every protocol in ${project}. It repeats on the printed page and is not part of the document you are editing.`,
  },

  shell: {
    breadcrumbMeeting: 'Meeting',
    breadcrumbRecording: 'Recording',
    breadcrumbReview: 'Review',
    skipToWorkspace: 'Skip to workspace',
    workspace: 'Workspace',
    workspaceFailed: 'Workspace could not be opened',
    workspaceFailedDetail: 'Your existing files have not been changed.',
    tryAgain: 'Try again',
    preparingWorkspace: 'Preparing local workspace…',
    openNavigation: 'Open navigation',
  },

  protocol: {
    notFound: 'Not found',
    matchCount: (count: number) => `${count} ${count === 1 ? 'match' : 'matches'}`,
    replacedCount: (count: number) => ` · replaced ${count}`,
    changesNotYetMade: (count: number) =>
      `${count} ${count === 1 ? 'change' : 'changes'}, not yet made`,
    compoundNote:
      'A capitalised name is looked for inside compounds as well, which is where a plain replace misses it. Read them, then keep them or leave them.',
    andMore: (count: number) => `and ${count} more, all of the same two forms.`,
    passageGoesAlone:
      'The passage goes to your local model on its own. Numbers, names and dates are to come back unchanged — check them, and undo if they did not.',
    nothingChangedYet:
      'Nothing has been changed yet. Read it, then keep it or leave it — a local model rewrites well and is not to be taken on trust.',
    secondPassNote:
      'Asked of your own model, and it is wrong in both directions: it misses changes and it queries wording that is fine. Worth a look, not a verdict.',
    pageEdgesNote:
      'Where the pages would end, measured the way the print stylesheet sets them: a heading or a table moves down whole rather than splitting, prose does not. The printer settles the last line or two, so treat this as within a line rather than exact.',
    transcriptSourceNote:
      'Written from the reviewed transcript of this meeting. Nothing records which passage produced which sentence, so what follows looks for the words rather than claiming to know — a paraphrase will find nothing, which is the honest answer.',
    noWordsTogether:
      'None of these words appear together in the transcript. That usually means the draft has put it in its own words, which it is entitled to do — the recording is the place to check it.',
    revisionNote:
      'Typing is kept as working edits and does not make a revision. A revision is made when a draft is generated, when you ask for one, when you mark a protocol reviewed, and when an older one is restored — so this list stays short enough to read.',
    nothingRewrites:
      'Nothing here rewrites your text for you. The draft is yours to edit, and every revision is kept.',
    figuresKept: (kept: number, stated: number) => `${kept} of ${stated} figures kept`,
    figuresNote: (stated: number, kept: number) =>
      `The meeting stated ${stated} figures and this draft repeats ${kept} of them. How many belong here is a matter of the style you chose, so this is something to look at rather than a score.`,
    figuresInvented: (count: number) =>
      count === 1
        ? 'One figure appears here that the meeting did not state'
        : `${count} figures appear here that the meeting did not state`,
    confirmAgainstRecording: '. Worth confirming against the recording.',
    tasksUnowned: (count: number) =>
      count === 1
        ? 'One task here has nobody against it'
        : `${count} tasks here have nobody against them`,
    unownedNote:
      '. The draft leaves an owner out rather than guessing at one, so this may be exactly what the meeting decided — and it is far cheaper to put a name to it now than at the next meeting.',
    editor: 'Protocol editor',
    markdownBacked: 'Markdown backed',
    /**
     * What LocaLog writes into a protocol when the document has to admit something
     * about itself. Printed in the document, so they follow the interface's language
     * the way the page header does, and they are whole sentences because a language
     * that orders the parts differently cannot be served by assembling fragments.
     */
    noteMissingTableHeading: 'No table of next steps',
    noteMissingTableBody:
      'This protocol was written three times and none of them ended with a table of agreed tasks and their owners. Any actions the meeting agreed are described in the sections above but are not collected here.',
    noteGapsHeading: 'Not covered by this protocol',
    noteOneGap:
      'One stretch of the recording could not be read, and nothing above describes it. The recording itself is complete and it can still be listened to.',
    noteSeveralGaps:
      'Several stretches of the recording could not be read, and nothing above describes them. The recording itself is complete and these stretches can still be listened to.',
    /** Printed in the page header, as the kind of document this is. */
    documentType: 'Protocol',
    statusDraft: 'Draft',
    statusReviewed: 'Reviewed',
    statusChanged: 'Changed since review',
    fieldProjectName: 'Project name',
    fieldMeetingTitle: 'Meeting title',
    fieldMeetingDate: 'Meeting date',
    fieldDocumentType: 'Document type',
    fieldProtocolStatus: 'Status',
    fieldPageNumber: 'Page number',
    fieldPageOfCount: 'Page n of m',
    fieldText: 'Custom text',
    editorTools: 'Editor tools',
    find: 'Find',
    findInProtocol: 'Find in protocol',
    replaceWith: 'Replace with',
    makeChanges: 'Make these changes',
    leaveIt: 'Leave it',
    zoomOut: 'Zoom out',
    zoomIn: 'Zoom in',
    insertTable: 'Insert table',
    insertDivider: 'Insert divider',
    documentMenu: 'Document menu',
    clearFormatting: 'Clear formatting',
    table: 'Table',
    blockType: 'Block type',
    addColumnLeft: 'Add a column to the left',
    addColumnRight: 'Add a column to the right',
    deleteColumn: 'Delete this column',
    addRowAbove: 'Add a row above',
    addRowBelow: 'Add a row below',
    deleteRow: 'Delete this row',
    formatting: 'Formatting',
    bold: 'Bold',
    italic: 'Italic',
    bulletedList: 'Bulleted list',
    numberedList: 'Numbered list',
    quotation: 'Quotation',
    askModel: 'Ask the model to say this differently',
    customInstruction: 'Custom instruction…',
    whatShouldChange: 'What should change?',
    proposedChange: 'Proposed change',
    proposedReplacement: 'Proposed replacement',
    proposedRewrite: 'Proposed rewrite',
    unchanged: 'The model returned the passage unchanged.',
    factsMoved: 'A second pass thinks these facts moved',
    noFactMoved: 'A second pass found no fact moved. It misses things.',
    useThis: 'Use this',
    improveClarity: 'Improve clarity',
    improveClarityInstruction: 'Make this clearer to read.',
    makeFormal: 'Make more formal',
    makeFormalInstruction:
      'Make the register more formal, as a professional minute would be written.',
    makePlainer: 'Make plainer',
    makePlainerInstruction: 'Make the wording plainer and more direct, without losing precision.',
    shorten: 'Shorten',
    shortenInstruction: 'Say this in fewer words.',
    rewriteUnavailable: 'Rewriting is not available here.',
    replaceUnavailable: 'Replacing a name is not available here.',
    nameNotFound: 'That name is not in this protocol.',
    protocolMarkdown: 'Protocol Markdown',
    protocolLabel: 'Protocol',
    protocolDetails: 'Protocol details',
    documentDetails: 'Document details',
    closeInspector: 'Close inspector',
    tabDocument: 'Document',
    tabTranscript: 'Transcript',
    tabHistory: 'History',
    status: 'Status',
    createRevision: 'Create revision',
    markReviewed: 'Mark reviewed',
    style: 'Style',
    sections: 'Sections',
    newSection: 'New section',
    appearance: 'Appearance',
    editAppearance: 'Edit appearance',
    headerFooter: 'Header & footer',
    editHeaderFooter: 'Edit header & footer',
    nothingRepeated: 'Nothing repeated on the page',
    presets: 'Presets',
    useOrSavePreset: 'Use or save a preset',
    noneSaved: 'None saved yet',
    savedCount: (count: number) => `${count} saved`,
    use: 'Use',
    remove: 'Remove',
    nameThisPreset: 'Name this preset',
    nameForPreset: 'Name for this preset',
    save: 'Save',
    cancel: 'Cancel',
    saveAsPreset: 'Save this appearance and header as a preset',
    export: 'Export',
    exportPdf: 'Export PDF',
    exportWord: 'Export Word',
    exportMarkdown: 'Export Markdown',
    exportPlainText: 'Export plain text',
    exportNote:
      'The PDF is printed from the document you are reading, set the way this project sets its protocols — choose “Save as PDF” in the print dialog.',
    source: 'Source',
    findSelectedPassage: 'Find the selected passage',
    lookingFor: 'Looking for:',
    openReviewedTranscript: 'Open reviewed transcript',
    whatToCheck: 'What to check',
    revisions: 'Revisions',
    current: 'Current',
    restore: 'Restore',
  },

  sidebar: {
    projects: 'Projects',
    newProject: 'New project',
    createProject: 'Create project',
    library: 'Library',
    protocolStyles: 'Protocol styles',
    namesAndTerms: 'Names & terms',
    settings: 'Settings',
    recording: 'Recording',
    primaryNavigation: 'Primary navigation',
    closeNavigation: 'Close navigation',
    openNavigation: 'Open navigation',
    themeFollowingSystem: 'Following the system theme. Switch to always light.',
    themeAlwaysLight: 'Always light. Switch to always dark.',
    themeAlwaysDark: 'Always dark. Switch to following the system.',
    themeFollowingShort: 'Following the system',
    resizeSidebar: 'Resize sidebar. Use arrow keys to adjust or Enter to reset.',
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
