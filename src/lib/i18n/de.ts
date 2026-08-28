/**
 * Every word the application says, in German.
 *
 * Typed against English, so this file cannot be missing a key or inventing one.
 *
 * ## Decisions taken once, here, so the whole application reads as one voice
 *
 * **Sie, not du.** This is written for professional offices keeping minutes of
 * formal meetings. Du would be wrong in a Protokoll and wrong in the application
 * that writes one.
 *
 * **Protokoll, not "Protokoll (Transkript)" or any gloss.** The product exists
 * because *Protokoll* is a real institutional practice in German-speaking
 * offices, with an expected shape — participants, decisions, tasks with owners.
 * The English "protocol" is the translation; this is the original.
 *
 * **Besprechung for a meeting.** Meeting is common in German offices and would
 * be understood, but Besprechung is the word that belongs beside Protokoll.
 * Sitzung is reserved for a formal body — a board, a council — and would
 * overstate what most of these are.
 *
 * **Aufnahme for a recording, Spur for a track.** Both are the ordinary words a
 * German-speaking person would use about audio, not calques of the English.
 *
 * Where English uses a dash for an aside, German uses one too; where English
 * uses a semicolon, German more often takes a full stop, and this file does.
 */

import type { Strings } from './en';

const failures = {
  missingProject: 'Das gewählte Projekt existiert nicht mehr.',
  missingMeeting: 'Die gewählte Besprechung existiert nicht mehr.',
  missingJob: 'Der Importvorgang ist nicht mehr verfügbar.',
  importBusy:
    'Es wird bereits eine Aufnahme importiert. Schließen Sie diesen Vorgang ab oder brechen Sie ihn ab.',
  unsupportedSchema: (version: string) =>
    `Diese LocaLog-Daten stammen aus einer neueren, nicht unterstützten Version (${version}).`,
  storageUnavailable: 'LocaLog konnte nicht auf seinen lokalen Arbeitsbereich zugreifen.',

  styleMissing: 'Dieser Stil existiert nicht mehr.',
  styleNameRequired: 'Geben Sie dem Stil einen Namen.',
  styleNotSaved: 'Der Stil konnte nicht gespeichert werden.',
  styleUnavailable: 'Der gewählte Protokollstil ist nicht verfügbar.',
  styleUsedByMeeting: 'Eine Besprechung verwendet diesen Stil. Ändern Sie diese zuerst.',
  styleUsedByProject: 'Ein Projekt verwendet diesen Stil als Standard. Ändern Sie das zuerst.',

  presetNameRequired: 'Geben Sie der Voreinstellung einen Namen.',
  presetNotSaved: 'Die Voreinstellung konnte nicht gespeichert werden.',
  presetBuiltInUndeletable:
    'Eine mit LocaLog ausgelieferte Voreinstellung kann nicht entfernt werden.',

  transcriptInvalid: 'Das gespeicherte Transkript ist ungültig.',
  transcriptSegmentMissing: 'Dieser Transkriptabschnitt existiert nicht mehr.',
  transcriptTextRequired: 'Geben Sie gültigen Transkripttext ein.',
  transcriptNeedsSegment: 'Ein Transkript braucht mindestens einen Abschnitt.',
  transcriptSpeakerRequired: 'Geben Sie eine gültige Sprecherbezeichnung ein.',
  transcriptNotSaved: 'Das Transkript konnte nicht gespeichert werden.',
  transcriptNotCommitted: 'Das Transkript konnte nicht festgeschrieben werden.',
  spellingRequired: 'Geben Sie eine gültige Schreibweise ein.',

  protocolTextRequired: 'Geben Sie gültigen Protokolltext ein.',
  protocolRevisionMissing: 'Die gewählte Protokollfassung existiert nicht mehr.',
  protocolNeededBeforeExport: 'Erzeugen Sie ein Protokoll, bevor Sie es exportieren.',
  protocolNeededBeforeSetAside:
    'Erzeugen Sie ein Protokoll, bevor Sie einen Abschnitt zurücklegen.',
  sectionNotSetAside: 'Der Abschnitt konnte nicht zurückgelegt werden.',
  reviewBeforeGeneration: 'Prüfen Sie das Transkript vor der Erzeugung.',
  vocabularyUnresolved: 'Die Begriffe konnten nicht aufgelöst werden.',

  selectionRequired: 'Markieren Sie den Text, der geändert werden soll.',
  selectionTooLong:
    'Das ist zu viel Text auf einmal. Markieren Sie einen Abschnitt statt des ganzen Dokuments.',
  passageNotRewritten: 'Diese Passage konnte nicht umformuliert werden.',
  openingNotRead: 'Der Beginn der Besprechung konnte nicht gelesen werden.',
  providerNeededForPassage:
    'Starten Sie Ihre vorhandene Ollama-Installation, bevor Sie eine Passage ändern.',
  providerNeededForOpening:
    'Starten Sie Ihre vorhandene Ollama-Installation, bevor Sie die Vorstellungsrunde lesen.',
  providerModelRequired:
    'Wählen Sie in den Einstellungen unter „Protokollerzeugung“ ein installiertes Ollama-Modell.',

  // Validierung und Speicherung.
  styleNotMigrated: 'Ein Stil konnte nicht übernommen werden.',
  termMissing: 'Dieser Begriff existiert nicht mehr.',
  exportFormatInvalid: 'Wählen Sie ein gültiges Exportformat.',
  meetingDateInvalid: 'Wählen Sie ein gültiges Datum für die Besprechung.',
  scopeInvalid: 'Wählen Sie einen gültigen Geltungsbereich.',
  sourceFileInvalid: 'Wählen Sie eine gültige Quelldatei.',
  workspaceViewInvalid: 'Wählen Sie eine gültige Ansicht.',
  recordingUnreadable: 'Diese Aufnahme konnte nicht gelesen werden.',
  appearanceNotSaved: 'Das Erscheinungsbild konnte nicht gespeichert werden.',
  furnitureNotSaved: 'Kopf- und Fußzeile konnten nicht gespeichert werden.',
  documentOperationFailed: 'Der Vorgang am Dokument konnte nicht abgeschlossen werden.',
  providerConfigNotSaved:
    'Die Einstellungen zur Protokollerzeugung konnten nicht gespeichert werden.',
  runtimeConfigNotSaved: 'Die Einstellungen zur Transkription konnten nicht gespeichert werden.',
  recorderNotStarted: 'Die Aufnahme konnte nicht gestartet werden.',
  tracksNotCombined: 'Die Spuren der Aufnahme konnten nicht zusammengeführt werden.',
  protocolInvalid: 'Das gespeicherte Protokoll ist ungültig.',
  protocolNotUtf8: 'Das gespeicherte Protokoll ist kein gültiges UTF-8.',
  editsNotRecorded: 'Diese Änderungen konnten nicht festgehalten werden.',

  // Fehler aus der Befehlsschicht.
  recordingAlreadyRunning: 'Es wird bereits eine Besprechung aufgenommen.',
  presetUnknown: 'Wählen Sie eine bekannte Transkriptionsqualität.',
  providerModelNotInstalled: 'Wählen Sie ein Modell, das in Ollama bereits installiert ist.',
  diariserPathInvalid: 'Wählen Sie ein vorhandenes Programm zur Sprechertrennung.',
  whisperPathInvalid: 'Wählen Sie eine vorhandene whisper.cpp-Anwendung.',
  nothingRecording: 'Es wird gerade nichts aufgenommen.',
  revealOnlyOnMac:
    'Das Öffnen des Ordners ist bisher nur unter macOS eingerichtet. Der Pfad oben stimmt.',
  privacySettingsOnlyOnMac:
    'Das Öffnen der Datenschutzeinstellungen ist bisher nur unter macOS eingerichtet.',
  providerNeededForModel:
    'Starten Sie Ihre vorhandene Ollama-Installation, bevor Sie ein Modell wählen.',
  settingsNotOpened: 'Die Systemeinstellungen konnten nicht geöffnet werden.',
  presetMissing: 'Diese Voreinstellung ist nicht mehr verfügbar.',
  downloadStopped: 'Der Download wurde unerwartet beendet.',
  coordinatorUnavailable: 'LocaLog konnte den Vorgang nicht starten. Starten Sie LocaLog neu.',
  taskStopped: 'Ein interner Vorgang wurde unerwartet beendet.',
  recorderPermissionsUnknown: 'Die Aufnahme konnte nicht nach ihren Berechtigungen gefragt werden.',
  recorderStateUnknown: 'Die Aufnahme ist in einem unbekannten Zustand. Starten Sie LocaLog neu.',
  recordingNotFinished: 'Die Aufnahme konnte nicht abgeschlossen werden.',
  replacementNotPrepared: 'Die Ersetzung konnte nicht vorbereitet werden.',
  workspaceNotOpened: 'Der Ordner konnte nicht geöffnet werden.',
  settingsPaneUnknown: 'Diesen Einstellungsbereich gibt es nicht.',
  meetingBusy: 'An dieser Besprechung wird noch gearbeitet. Brechen Sie das zuerst ab.',
  printDialogUnavailable: 'Dieses Fenster konnte den Druckdialog nicht öffnen.',

  // Sichern und Zurückspielen.
  backupNameUnsafe: 'Dieser Name kann nicht als Ordnername verwendet werden.',
  notABackup: 'Dieser Ordner ist keine LocaLog-Sicherung: Es fehlt die Datei manifest.json.',
  backupPathOutside: (path: string) =>
    `Diese Sicherung nennt eine Datei außerhalb ihres eigenen Ordners (${path}) und wurde deshalb nicht zurückgespielt.`,
  backupFormatUnknown: (format: string) =>
    `Diese Sicherung wurde im Format ${format} geschrieben, das diese LocaLog-Version nicht lesen kann. Eine neuere kann es.`,
  backupDamaged: (what: string) =>
    `Diese Sicherung ist unvollständig oder beschädigt (${what}). Es wurde nichts geändert, Ihre laufende Arbeit ist unberührt.`,
  backupNameTaken: (name: string) => `In diesem Ordner gibt es bereits etwas namens „${name}“.`,
  backupIoFailed: (what: string) =>
    `Die Sicherung konnte nicht geschrieben oder gelesen werden: ${what}`,
  backupDatabaseFailed: (what: string) => `Die Datenbank konnte nicht kopiert werden: ${what}`,

  // Validation the storage layer does, and failures that carry the reason.
  categoryRequired: 'Wählen Sie eine Kategorie.',
  meetingLanguageRequired: 'Wählen Sie eine Sprache für die Besprechung.',
  meetingLanguageInvalid: 'Wählen Sie eine gültige Sprache für die Besprechung.',
  meetingInvalid: 'Wählen Sie eine gültige Besprechung.',
  projectInvalid: 'Wählen Sie ein gültiges Projekt.',
  styleInvalid: 'Wählen Sie einen gültigen Protokollstil.',
  sourceRecordingInvalid: 'Wählen Sie eine gültige Quellaufnahme.',
  meetingTitleRequired: 'Geben Sie einen Titel für die Besprechung ein.',
  projectNameRequired: 'Geben Sie einen Projektnamen ein.',
  termRequired: 'Geben Sie einen Begriff ein.',
  meetingTitleTooLong: 'Der Titel der Besprechung ist zu lang.',
  speakerPassCannotRead: (what: string) =>
    `Die Sprechererkennung konnte die Arbeitsdatei nicht lesen: ${what}`,
  speakerPassCannotWrite: (what: string) =>
    `Die Sprechererkennung konnte ihr Audio nicht schreiben: ${what}`,
  recordingNotStored: (what: string) => `Die Aufnahme konnte nicht gespeichert werden: ${what}`,
  recordingNotRead: (what: string) => `Die Aufnahme konnte nicht gelesen werden: ${what}`,
  modelNotDownloaded: (what: string) => `Das Modell konnte nicht geladen werden: ${what}`,
  modelNotSaved: (what: string) => `Das Modell konnte nicht gespeichert werden: ${what}`,
  ollamaRequestFailed: (what: string) =>
    `Ollama konnte die lokale Anfrage nicht abschließen: ${what}`,
  recorderStartFailed: (what: string) => `Die Aufnahme konnte nicht gestartet werden: ${what}`,

  // Die Verarbeitung: Audio, Modelle, lokales Modell, Aufnahme.
  embeddingsUnrecognisable: 'Die Sprechererkennung hat keine lesbaren Stimmprofile geschrieben.',
  embeddingsNoDimensions: 'Die Stimmprofile beschreiben keine Dimensionen.',
  embeddingsTruncated: 'Die Stimmprofile sind kürzer, als sie angeben.',
  probeInvalid: 'Die Medienprüfung hat ungültige Angaben zurückgegeben.',
  cachePathInvalid: 'Der Pfad zur aufbereiteten Audiodatei ist ungültig.',
  normalizerNoOutput: 'Die Audioaufbereitung hat keine Datei erzeugt.',
  speakerPassNoAudio: 'Für die Sprechererkennung ist nichts zu hören.',
  speakerPassTooMuchAudio:
    'Die Sprechererkennung hat mehr Audio geplant, als gehalten werden kann.',
  recordingEmpty: 'Die Aufnahme wurde als leere Datei gespeichert.',
  editsLeaveNothing: 'Nach diesen Änderungen bliebe von der Aufnahme nichts übrig.',
  workingAudioUnreadable: 'Die Arbeitsdatei ist keine lesbare WAV-Datei.',
  workingAudioNotWav: 'Die Arbeitsdatei ist keine WAV-Datei.',
  workingAudioSilent: 'Die Arbeitsdatei enthält kein Audio.',
  workingAudioFormatUnreadable: 'Die Arbeitsdatei hat ein unlesbares Format.',
  workingAudioNoFormat: 'Die Arbeitsdatei beschreibt kein Format.',
  condensedAudioTooLarge: 'Das zusammengefasste Audio ist zu groß zum Schreiben.',
  combinedPathInvalid: 'Der Pfad zur zusammengeführten Aufnahme ist ungültig.',
  modelUnknown: 'Dieses Transkriptionsmodell ist nicht bekannt.',
  downloadCancelled: 'Der Download wurde abgebrochen.',
  downloadCorrupt: 'Der Download war unvollständig oder beschädigt und wurde verworfen.',
  ollamaModelGone:
    'Das gewählte Ollama-Modell ist nicht mehr installiert. Wählen Sie ein anderes und versuchen Sie es erneut.',
  ollamaModelChanged:
    'Das gewählte Ollama-Modell hat sich geändert, nachdem dieser Vorgang eingereiht wurde. Versuchen Sie es erneut.',
  ollamaRuntimeChanged:
    'Die Ollama-Laufzeit hat sich geändert, nachdem dieser Vorgang eingereiht wurde. Versuchen Sie es erneut.',
  responseTooLarge:
    'Die Antwort des lokalen Modells hat die sichere Grenze überschritten und wurde nicht übernommen.',
  responseIncomplete:
    'Das lokale Modell hat abgebrochen, bevor ein vollständiges Protokoll vorlag.',
  recorderMissing:
    'Es ist keine Aufnahmekomponente installiert. LocaLog liefert eine mit, dieser Build findet sie nicht.',
  recorderSilentAboutPermissions: 'Die Aufnahmekomponente hat nicht mitgeteilt, was sie darf.',
  recorderCannotReportPermissions: 'Diese Aufnahmekomponente kann nicht mitteilen, was sie darf.',
  runtimePathsMustBeAbsolute:
    'Wählen Sie absolute Pfade für die whisper.cpp-Anwendung und das Modell.',
  whisperExecutableMissing: 'Die gewählte whisper.cpp-Anwendung wurde nicht gefunden.',
  whisperModelMissing: 'Das gewählte whisper.cpp-Modell wurde nicht gefunden.',
  embeddingsVersion: (version: string) =>
    `Diese Stimmprofile haben Version ${version}, die dieser Build nicht liest.`,
  recordingTooSmall: (what: string) =>
    `Die gespeicherte Aufnahme ist zu klein für ihre Länge (${what}).`,
  workingAudioFormatWrong: (what: string) =>
    `Die Sprechererkennung braucht 16 kHz, mono, 16 Bit — vorliegend ist ${what}.`,
  notEnoughSpace: (what: string) => `Nicht genug Platz für dieses Modell (${what}).`,
};

export const de: Strings = {
  failures,

  settings: {
    interfaceLanguage: 'Sprache der Oberfläche',
    interfaceLanguageDetail:
      'In welcher Sprache LocaLog selbst geschrieben ist. Unabhängig von der Sprache der einzelnen Besprechung.',
    application: 'Anwendung',
    title: 'Einstellungen',
    lead: 'Zuerst die fachlichen Vorgaben. Technische Details bleiben schrittweise ausgeblendet.',
    sectionsLabel: 'Bereiche der Einstellungen',
    sectionGeneral: 'Allgemein',
    sectionModels: 'Modelle',
    sectionTranscription: 'Transkription',
    sectionStorage: 'Speicher',
    sectionAppearance: 'Erscheinungsbild',
    sectionAdvanced: 'Erweitert',
    defaultExport: 'Standardexport',
    defaultExportDetail:
      'Welches Format der Protokolleditor zuerst anbietet. Die anderen bleiben einen Klick entfernt.',
    defaultExportLabel: 'Standard-Exportformat',
    formatPdf: 'PDF',
    formatWord: 'Word',
    formatMarkdown: 'Markdown',
    formatPlainText: 'Reiner Text',
    defaultForProtocols: 'Vorgabe für Protokolle',
    chooseOnce: 'Einmal wählen, dann weiterarbeiten',
    modelLead:
      'LocaLog verwendet dieses Modell für lokale Protokollentwürfe, bis Sie es ändern. Der normale Ablauf verlangt nicht, für jede Besprechung ein Modell zu wählen.',
    recommendedForMachine: 'Für dieses Gerät empfohlen',
    notInstalledYet: 'Noch nicht installiert',
    baseline: 'Grundlage',
    european: 'Europäisch',
    checkInstalled: 'Installierte Modelle prüfen',
    curatedModels: 'Ausgewählte Protokollmodelle',
    useAnotherModel: 'Ein anderes installiertes Modell verwenden',
    installedModel: 'Installiertes Modell',
    chooseInstalledModel: 'Ein installiertes Modell wählen',
    useInstalledModel: 'Installiertes Modell verwenden',
    conservativeBaseline: 'Es wird die vorsichtige Annahme von 8 GB verwendet',
    transcriptionQuality: 'Transkriptionsqualität',
    cancel: 'Abbrechen',
    ready: 'Bereit',
    remove: 'Entfernen',
    advancedDetails: 'Technische Details',
    modelsStoredNote:
      'Modelle liegen im Anwendungsdatenordner von LocaLog und werden vor der Verwendung geprüft.',
    whisperExecutable: 'whisper-cli-Anwendung',
    chooseFile: 'Datei wählen',
    whisperNote:
      'Wählen Sie das Kommandozeilenprogramm für die Transkription, nicht whisper-server.',
    saveRuntime: 'Laufzeit speichern',
    detected: (version: string) => `Erkannt: ${version}`,
    chooseWhisper: 'whisper-cli-Anwendung wählen',
    speakerDifferentiation: 'Sprecherunterscheidung',
    speakerLead:
      'Die optionale Sprechertrennung hält fest, wer wann gesprochen hat. Sie blockiert nie ein Transkript, und die Bezeichnungen bleiben bei der Prüfung änderbar.',
    runtimeUnavailable: 'In dieser Installation nicht verfügbar',
    optional: 'Optional',
    checkReadiness: 'Bereitschaft prüfen',
    downloadingSpeakerModels: 'Modelle zur Sprechertrennung werden geladen',
    speakerRuntimeMissing:
      'Die Modelle sind vorbereitet, aber diese Installation hat keine passende Sprechererkennung.',
    whereWorkIsKept: 'Wo Ihre Arbeit liegt',
    workspaceNote:
      'LocaLog verwaltet diesen Ordner, damit die Pfade darin gültig bleiben — er gehört aber Ihnen, und Sie können jederzeit hineinsehen.',
    showInFinder: 'Im Finder zeigen',
    backup: 'Sicherung',
    backupLead:
      'Alles bleibt auf diesem Gerät — was auch heißt, dass es mit dem Gerät verschwindet. Eine Sicherung ist ein gewöhnlicher Ordner, den Sie auf ein Laufwerk legen können oder wohin Sie sonst Sicheres legen.',
    backUpNow: 'Jetzt sichern',
    working: 'Wird ausgeführt …',
    backupContents:
      'Enthält jedes Projekt, jede Besprechung, jedes Transkript und Protokoll sowie die Aufnahmen selbst. Zwei Dinge fehlen mit Absicht, weil beide nicht Ihre Arbeit sind und bei Bedarf neu entstehen: geladene Modelle und die aufbereitete Kopie jeder Aufnahme. An einem echten Arbeitsbereich gemessen war allein dieses aufbereitete Audio drei Viertel der Sicherung.',
    restore: 'Zurückspielen',
    restoreLead:
      'Spielt eine Sicherung zurück. Sie wird zuerst vollständig geprüft, und was jetzt hier liegt, wird beiseitegelegt statt gelöscht.',
    chooseBackup: 'Sicherung wählen …',
    chooseBackupTitle: 'Eine LocaLog-Sicherung wählen',
    whereToKeepBackup: 'Wo die Sicherung liegen soll',
    replaceWorkspace: 'Diesen Arbeitsbereich ersetzen',
    restoring: 'Wird zurückgespielt …',
    archived: 'Archiviert',
    archivedLead:
      'Projekte und Besprechungen, die beiseitegelegt wurden. Nichts wurde gelöscht: Jede Besprechung, jedes Transkript und Protokoll darunter ist noch da — und in jeder Sicherung.',
    show: 'Zeigen',
    hide: 'Verbergen',
    nothingArchived: 'Es wurde nichts archiviert.',
    project: 'Projekt',
    meeting: 'Besprechung',
    bringBack: 'Zurückholen',
    theme: 'Erscheinungsbild',
    themeFollowing: (theme: string) => `Folgt diesem Mac, der auf ${theme} steht.`,
    themeSetHere: 'Hier festgelegt, unabhängig von der Einstellung dieses Macs.',
    nextFakeJob: 'Nächster simulierter Vorgang',
    nextFakeJobDetail:
      'Nur für die Entwicklung: zum Prüfen von Fehler- und Wiederholungszuständen.',
    completeNormally: 'Normal abschließen',
    failOnce: 'Einmal fehlschlagen, dann Wiederholung erlauben',
    syntheticNote: 'Das betrifft nur die synthetische Laufzeit im Arbeitsspeicher.',
  },

  project: {
    eyebrow: 'Projekt',
    archiveProject: 'Projekt archivieren',
    newMeeting: 'Neue Besprechung',
    meetings: 'Besprechungen',
    newestFirst: 'Neueste zuerst',
    columnDate: 'Datum',
    columnMeeting: 'Besprechung',
    columnDuration: 'Dauer',
    columnStatus: 'Status',
    archive: 'Archivieren',
    delete: 'Löschen',
    keep: 'Behalten',
    noMeetings: 'Noch keine Besprechungen',
    noMeetingsDetail:
      'Importieren Sie die erste Aufnahme, um mit dem Besprechungsverlauf dieses Projekts zu beginnen.',
    importRecording: 'Aufnahme importieren',
  },

  lifecycle: {
    draft: 'Entwurf',
    sourceReady: 'Bereit zur Transkription',
    transcriptReady: 'Transkript fertig',
    protocolDraft: 'Protokollentwurf',
    reviewed: 'Geprüft',
    archived: 'Archiviert',
  },

  sections: {
    noHeadings: 'Dieses Protokoll hat noch keine Überschriften, es gibt also nichts aufzulisten.',
    setAside: 'Zurücklegen',
    addSection: 'Abschnitt hinzufügen',
    dragHint: 'Ziehen, oder die Pfeiltasten benutzen',
    setThisAside: 'Diesen Abschnitt zurücklegen',
    putThisBack: 'Diesen Abschnitt zurückholen',
    moveSection: (title: string) => `${title} verschieben. Mit den Pfeiltasten.`,
    setAsideNamed: (title: string) => `${title} zurücklegen`,
    putBackNamed: (title: string) => `${title} zurückholen`,
    setAsideNote:
      'Ein zurückgelegter Abschnitt verlässt das Dokument — was Sie lesen, ist also genau das, was exportiert wird. Er bleibt hier erhalten und kann zurückgeholt werden.',
  },

  stages: {
    label: 'Schritte der Besprechung',
    source: 'Quelle',
    transcript: 'Transkript',
    protocol: 'Protokoll',
  },

  progress: {
    importing: 'Aufnahme wird importiert',
    transcribing: 'Transkription läuft',
    generating: 'Protokoll wird erzeugt',
    separatingSpeakers: 'Sprecher werden getrennt',
    working: 'Wird bearbeitet …',
    duplicateNote:
      'Derselbe Inhalt liegt bereits in LocaLog. Es wurde nichts zusammengeführt und nichts verworfen.',
    cancelImport: 'Import abbrechen',
    importAnotherCopy: 'Weitere Kopie importieren',
    chooseSourceAgain: 'Quelle erneut wählen',
    continueImport: 'Import fortsetzen',
    transcribeAgain: 'Transkription erneut starten',
    generateAgain: 'Erzeugung erneut starten',
  },

  newProject: {
    eyebrow: 'Projekte',
    title: 'Neues Projekt',
    lead: 'Legen Sie den fachlichen Zusammenhang an, zu dem Besprechungen und Quellen gehören.',
    defaults: 'Projektvorgaben',
    name: 'Projektname',
    namePlaceholder: 'z. B. Studie Bürgerhaus',
    description: 'Beschreibung',
    descriptionOptional: 'optional',
    descriptionPlaceholder: 'Eine knappe interne Beschreibung',
    defaultLanguage: 'Standardsprache der Besprechung',
    defaultLanguageDetail: 'Unabhängig von der Sprache der Oberfläche.',
    cancel: 'Abbrechen',
  },

  appearance: {
    bodySize: 'Schriftgröße',
    headingScale: 'Überschriften',
    lineSpacing: 'Zeilenabstand',
    pageWidth: 'Satzbreite',
  },

  record: {
    recordingNow: 'Aufnahme läuft',
    recordThisMeeting: 'Diese Besprechung aufnehmen',
    lead: 'Raum und Konferenz werden auf getrennten Spuren aufgenommen, auf diesem Gerät. Ob die Teilnehmenden der Aufnahme zugestimmt haben, klären Sie selbst — das kann LocaLog nicht wissen.',
    notRecording: 'Keine Aufnahme',
    microphone: 'Mikrofon',
    theCall: 'Die Konferenz',
    trackRecording: 'Nimmt auf',
    trackSilent: 'Bisher still',
    trackListening: 'Hört zu …',
    stopRecording: 'Aufnahme beenden',
    finishing: 'Wird abgeschlossen …',
    startRecording: 'Aufnahme starten',
    starting: 'Wird gestartet …',
    backToMeeting: 'Zurück zur Besprechung',
    noRecorder:
      'Dieser Build hat keine Aufnahmekomponente. Importieren Sie stattdessen eine Datei.',
    openTheSetting: 'Einstellung öffnen',
    grantedInSettings:
      'Wird in den Systemeinstellungen erteilt und hier übernommen, sobald Sie zurückkommen.',
    callWouldNotRecordTitle: 'Die Konferenz würde nicht aufgenommen.',
    callWouldNotRecordBody:
      'macOS hat LocaLog die Bildschirm- und Systemtonaufnahme nicht erlaubt. Ohne sie ist eine Aufnahme der Konferenz Stille statt eines Fehlers — das klärt man besser jetzt als hinterher. Das Mikrofon im Raum würde weiterhin aufgenommen.',
    roomWouldNotRecordTitle: 'Der Raum würde nicht aufgenommen.',
    roomWouldNotRecordBody:
      'LocaLog wurde das Mikrofon verweigert. Die Konferenz würde weiterhin aufgenommen, sofern die Einstellung darüber es zulässt.',
    recorderNotesTitle: 'Die Aufnahme konnte nicht alles tun, worum sie gebeten wurde.',
    stoppedOnItsOwn:
      'Die Aufnahme hat von selbst gestoppt. Was bis dahin aufgenommen wurde, ist erhalten.',
    quietCall: (seconds: number) =>
      `Seit ${seconds} Sekunden kommt nichts von der Konferenz an. macOS gibt einer Anwendung ohne die Berechtigung zur Bildschirm- und Systemtonaufnahme Stille statt eines Fehlers — das prüft man besser jetzt als nach der Besprechung.`,
    quietMicrophone: (seconds: number) =>
      `Seit ${seconds} Sekunden kommt nichts vom Mikrofon an. Prüfen Sie, ob der richtige Eingang gewählt ist und ob eine andere Anwendung ihn belegt.`,
  },

  meeting: {
    eyebrow: 'Besprechung',
    titleLabel: 'Titel der Besprechung',
    editTitle: 'Titel bearbeiten',
    languageLabel: 'Sprache der Besprechung',
    changeLanguage: 'Sprache ändern',
    save: 'Speichern',
    saveLanguage: 'Sprache speichern',
    cancel: 'Abbrechen',
    recordingEyebrow: 'Aufnahme',
    nothingRecorded: 'Noch nichts aufgenommen',
    recordLead:
      'Raum und Konferenz werden auf getrennten Spuren aufgenommen, auf diesem Gerät. Sie können jederzeit beenden, wenn die Besprechung vorbei ist.',
    recordThisMeeting: 'Diese Besprechung aufnehmen',
    sourceImport: 'Quelle importieren',
    originalUnchanged: 'Ihr Original bleibt unverändert',
    sourceReady: 'Quelle bereit',
    readyToTranscribe: 'Bereit zur Transkription',
    managedSource: 'Verwaltete Quelle',
    language: 'Sprache',
    languageHint: 'Einstellung der Besprechung · vor der Transkription oben ändern',
    preset: 'Qualität',
    globalDefault: 'Globale Vorgabe',
    notSelected: 'Nicht gewählt',
    peopleSpeaking: 'Sprechende Personen',
    doNotSeparate: 'Sprecher nicht trennen',
    separateAndCount: 'Trennen und Anzahl ermitteln',
    prepareSpeakers: 'Sprechertrennung vorbereiten',
    prepareSpeakersDetail:
      'LocaLog braucht zwei geprüfte lokale Modelldateien, bevor es vorläufige Sprecherbezeichnungen vergeben kann. Ihre Aufnahme bleibt auf diesem Gerät.',
    preparing: (percent: number) => `Wird vorbereitet ${percent} %`,
    prepare: 'Vorbereiten',
    prepareWithSize: (size: string) => `Vorbereiten (${size})`,
    speakerRuntimeMissing:
      'Die Sprechertrennung ist in dieser Installation nicht verfügbar. Die Transkription läuft weiter, das Transkript verwendet dann allgemeine, bearbeitbare Sprecherbezeichnungen.',
    reviewAndTrim: 'Aufnahme zuerst prüfen und kürzen',
    trimDetail:
      '— schneiden Sie die Wartezeit vor dem Beginn und alles Überflüssige weg. Ihre Aufnahme wird dabei nie verändert.',
    gettingReady: 'Transkription wird vorbereitet …',
    useJobControls: 'Nutzen Sie die Steuerung oben',
    prepareSpeakersFirst: 'Zuerst die Sprechertrennung vorbereiten',
    transcribe: 'Transkribieren',
    transcriptionFailedToStart:
      'Die Transkription konnte nicht gestartet werden. Bitte versuchen Sie es erneut.',
    transcriptReady: 'Transkript fertig',
    reviewBeforeGeneration: 'Vor der Erzeugung prüfen',
    transcriptReadyDetail:
      'Das Transkript mit Zeitmarken ist bereit für Korrekturen und die Zuordnung der Sprecher.',
    reviewTranscript: 'Transkript prüfen',
    protocolAvailable: 'Protokoll vorhanden',
    continueInEditor: 'Im Dokumenteditor weiterarbeiten',
    protocolDetail: 'Das Transkript bleibt neben der aktuellen Protokollfassung verfügbar.',
    openProtocol: 'Protokoll öffnen',
  },

  newMeeting: {
    titleRecording: 'Aufnahme',
    titleImport: 'Strukturierter Import',
    heading: 'Neue Besprechung',
    leadRecording:
      'Benennen Sie die Besprechung und wählen Sie ihr Projekt. Die Aufnahme beginnt auf dem nächsten Bildschirm.',
    leadImport: 'Wählen Sie die Aufnahme, bestätigen Sie die Angaben — den Rest übernimmt LocaLog.',
    context: 'Zusammenhang',
    chooseProject: 'Projekt wählen',
    project: 'Projekt',
    newProject: 'Neues Projekt',
    noInbox:
      'Jede Quelle gehört zu einer Besprechung, und jede Besprechung zu einem Projekt. Es gibt keinen Posteingang.',
    source: 'Quelle',
    importRecording: 'Aufnahme importieren',
    originalStays: 'Ihr Original bleibt, wo es ist',
    readyToCopy: 'Wird kopiert, sobald Sie die Besprechung bestätigen',
    letGoToImport: 'Loslassen zum Importieren',
    originalStaysShort: 'Das Original bleibt, wo es ist.',
    dropHere: 'Aufnahme hierher ziehen oder klicken, um eine zu wählen',
    dropDetail:
      'MP3, M4A, WAV, MP4, MOV und weitere. Das Original bleibt unberührt — LocaLog legt eine Kopie in seinem eigenen Speicher an.',
    readyToAssign: 'Bereit, dieser Besprechung zugeordnet zu werden',
    chooseFile: 'Audio- oder Videodatei wählen',
    previewNote: 'Die Browser-Vorschau zeigt den Ablauf, ohne die Datei zu speichern.',
    useDemoRecording: 'Synthetische Beispielaufnahme verwenden',
    essentials: 'Wesentliches',
    meetingInformation: 'Angaben zur Besprechung',
    title: 'Titel',
    titlePlaceholder: 'Wird aus der Datei abgeleitet, wenn leer',
    date: 'Datum',
    language: 'Sprache der Besprechung',
    protocolStyle: 'Protokollstil',
    projectDefault: 'Projektvorgabe',
    qualityNote:
      'Die Transkriptionsqualität wird einmal in den Einstellungen gewählt und gilt für jede Besprechung.',
    advanced: 'Erweiterte Verarbeitungsoptionen',
    cancel: 'Abbrechen',
    createAndRecord: 'Besprechung anlegen und aufnehmen',
    createAndImport: 'Besprechung anlegen und importieren',
  },

  recordingReview: {
    eyebrow: 'Aufnahme',
    heading: 'Aufnahme prüfen',
    noAudio: 'Noch keine Arbeitsdatei',
    waveformLabel: 'Die Aufnahme. Mit den Pfeiltasten bewegen, mit Umschalt auswählen.',
    startHere: 'Hier beginnen',
    removeSelection: 'Auswahl entfernen',
    endHere: 'Hier enden',
    edits: 'Änderungen',
    nothingRemoved: 'Nichts entfernt. Die ganze Aufnahme wird transkribiert.',
    undo: 'Rückgängig',
    putEverythingBack: 'Alles zurückholen',
    untouchedNote:
      'Die Aufnahme selbst bleibt unberührt. Dies sind Angaben dazu, was verwendet wird.',
    undoStartTrim: 'Kürzung am Anfang rückgängig machen',
    undoEndTrim: 'Kürzung am Ende rückgängig machen',
    putStretchBack: 'Diesen Abschnitt zurückholen',
    next: 'Weiter',
    continueToTranscription: 'Weiter zur Transkription',
    backToMeeting: 'Zurück zur Besprechung',
  },

  transcript: {
    heading: 'Transkript prüfen',
    exportTranscript: 'Transkript exportieren …',
    exportLabel: 'Dieses Transkript exportieren',
    asMarkdown: 'Als Markdown',
    asPlainText: 'Als reinen Text',
    reviewDetails: 'Details zur Prüfung',
    sourceContext: 'Quelle der Besprechung',
    seekAudio: 'Im Audio springen',
    follow: 'Folgen',
    followLabel: 'Das Transkript zum laufenden Abschnitt scrollen',
    searchTranscript: 'Transkript durchsuchen',
    editableTranscript: 'Bearbeitbares Transkript',
    removeLine: 'Diese Zeile aus dem Transkript entfernen',
    nothingFlagged: 'Nichts als unklar markiert',
    show: 'Zeigen',
    showing: 'Es werden gezeigt',
    onePassage: '1 unklare Passage',
    manyPassages: (count: number) => `${count} unklare Passagen`,
    speakerHint:
      'Die Sprecherbezeichnungen sind ein Anfang — benennen Sie sie nach den Personen, die gesprochen haben.',
    generateProtocol: 'Protokoll erzeugen',
    review: 'Prüfung',
    detailsLabel: 'Details zur Transkriptprüfung',
    closeInspector: 'Bereich schließen',
    speakers: 'Sprecher',
    whereRecordingStops: 'Wo die Aufnahme endet',
    transcriptionInput: 'Eingabe der Transkription',
    language: 'Sprache',
    meetingLanguage: 'Sprache der Besprechung',
    saveLanguage: 'Sprache speichern',
    cancel: 'Abbrechen',
    changeLanguage: 'Sprache ändern',
    rerunNote:
      'Verwenden Sie das nach einer Änderung der Sprache oder der Transkriptionseinstellungen. Der neue Durchlauf wird als eigene Fassung festgehalten.',
    rerun: 'Transkription erneut ausführen',
    rerunPreparing: 'Neues Transkript wird vorbereitet …',
    rerunConfirm: (language: string) =>
      `Transkription erneut auf ${language} ausführen? Das aktuelle Transkript bleibt erhalten, bis das neue Ergebnis übernommen wird — danach wird dieses Arbeitstranskript ersetzt.`,
    whoIsHere: 'Wer ist in dieser Besprechung',
    close: 'Schließen',
    aboutAMinute: 'Etwa eine Minute. Solange kann nichts anderes laufen.',
    unsureNames: 'Namen, bei denen die Transkription unsicher war',
    whatShouldItSay: 'Wie soll es heißen?',
    rememberForProject:
      'Für dieses Projekt merken, damit die nächste Besprechung es richtig schreibt',
    areAnyNames: 'Sind das Namen? Eine Korrektur bessert dieses Transkript aus und wird gemerkt.',
    nothingToCheck: 'Nichts zu prüfen',
    correctSpelling: 'Schreibweise korrigieren',
    checkWording: 'Formulierung prüfen',
    protocolStyle: 'Protokollstil',
    audioUnplayable: 'Die Arbeitsdatei dieser Besprechung konnte nicht abgespielt werden.',
    speakersResolved:
      'Die Sprecherwechsel wurden lokal bestimmt. Die Bezeichnungen sind vorläufig — benennen Sie sie erst um, wenn Sie die Person kennen.',
    speakersFailed:
      'Die Sprechertrennung hat für diesen Durchlauf keine brauchbaren Wechsel ergeben. Das Transkript ist vollständig und verwendet neutrale Bezeichnungen. Sie können sie von Hand vergeben.',
    speakersUnavailable:
      'Die Sprechertrennung war für diesen Durchlauf nicht verfügbar. Das Transkript ist vollständig und verwendet eine neutrale Bezeichnung, die Sie von Hand ändern können.',
    speakersUnknown:
      'Dieses ältere Transkript hält nicht fest, ob eine Sprechertrennung lief. Die neutralen Bezeichnungen sind kein Beleg dafür, dass nur eine Person gesprochen hat.',
  },

  library: {
    eyebrow: 'Bibliothek',
    protocolStyles: 'Protokollstile',
    namesAndTerms: 'Namen & Begriffe',
    stylesLead:
      'Was ein Protokoll sagt, und in welcher Reihenfolge. Nicht, wie es gesetzt ist — das ist das Erscheinungsbild, und das steht im Protokolleditor neben dem Dokument, das es beschreibt.',
    termsLead:
      'Die Namen, die eine Transkription nicht erraten kann: Ihr Projekt, die Firmen, die Personen. An einer echten Besprechung gemessen sind sie mehr wert als jede andere Einstellung hier.',
    addTerm: 'Begriff hinzufügen',
    saveTerm: 'Begriff speichern',
    stylesUnreadable: 'Stile können hier nicht gelesen werden.',
    length: 'Länge',
    name: 'Name',
    description: 'Beschreibung',
    whatItAsksFor: 'Was dieser Stil verlangt',
    addInstruction: 'Anweisung hinzufügen',
    removeInstruction: 'Diese Anweisung entfernen',
    checkedOnProtocol: 'Wird am fertigen Protokoll geprüft',
    alwaysEveryStyle: 'Immer, in jedem Stil',
    saveStyle: 'Stil speichern',
    cancel: 'Abbrechen',
    delete: 'Löschen',
    editThisStyle: 'Diesen Stil bearbeiten',
    duplicate: 'Duplizieren',
    duplicateToEdit: 'Zum Bearbeiten duplizieren',
    shippedStyleNote:
      'Ein mitgelieferter Stil bleibt, wie er ist, damit ein Protokoll von letztem Jahr wieder genauso geschrieben werden kann. Kopieren Sie ihn, um einen eigenen anzulegen.',
    ownershipAutomatic: 'Die Zuordnung geschieht automatisch.',
    termsScopeNote:
      'Namen und Begriffe eines Projekts gelten für seine Besprechungen, ohne dass sie jedes Mal gewählt werden müssen.',
    term: 'Begriff',
    spellingAsShown: 'Schreibweise, wie sie erscheinen soll',
    category: 'Kategorie',
    appliesTo: 'Gilt für',
    everyProject: 'Alle Projekte',
    unknownProject: 'Unbekanntes Projekt',
    noTerms: 'Noch keine Namen oder Begriffe',
    deleteThisTerm: 'Diesen Begriff löschen?',
    densityFull: 'Ausformuliert',
    densityPlain: 'Knappe Aussagen',
    densityLine: 'Eine Zeile je Punkt',
    densityFullMeaning: 'Ausformuliert. Wer nicht dabei war, kann der Besprechung folgen.',
    densityPlainMeaning: 'Knappe Aussagen. Was gesagt wurde, ohne Nacherzählung.',
    densityLineMeaning: 'Eine Zeile je Punkt. Der Vermerk, und nichts drumherum.',
    categoryPerson: 'Person',
    categoryOrganisation: 'Organisation',
    categoryProject: 'Projekt',
    categoryAbbreviation: 'Abkürzung',
    categoryTechnicalTerm: 'Fachbegriff',
    categoryOther: 'Sonstiges',
  },

  furniture: {
    header: 'Kopfzeile',
    footer: 'Fußzeile',
    left: 'Links',
    centre: 'Mitte',
    right: 'Rechts',
    insert: 'Einfügen …',
    lineHint:
      'Schreiben Sie die Zeile so, wie sie lauten soll, und setzen Sie einen Wert hinein, wo Sie einen möchten — „Seite “, die Seitenzahl, „ von 12“. Ein Wert ist ein Objekt: Er wird als Ganzes markiert und gelöscht.',
    appliesTo: (project: string) =>
      `Gilt für jedes Protokoll in ${project}. Sie wiederholt sich auf der gedruckten Seite und gehört nicht zu dem Dokument, das Sie bearbeiten.`,
  },

  shell: {
    breadcrumbMeeting: 'Besprechung',
    breadcrumbRecording: 'Aufnahme',
    breadcrumbReview: 'Prüfung',
    skipToWorkspace: 'Zum Arbeitsbereich springen',
    workspace: 'Arbeitsbereich',
    workspaceFailed: 'Der Arbeitsbereich konnte nicht geöffnet werden',
    workspaceFailedDetail: 'Ihre vorhandenen Dateien wurden nicht verändert.',
    tryAgain: 'Erneut versuchen',
    preparingWorkspace: 'Lokaler Arbeitsbereich wird vorbereitet …',
    openNavigation: 'Navigation öffnen',
  },

  sidebar: {
    projects: 'Projekte',
    newProject: 'Neues Projekt',
    createProject: 'Projekt anlegen',
    library: 'Bibliothek',
    protocolStyles: 'Protokollstile',
    namesAndTerms: 'Namen & Begriffe',
    settings: 'Einstellungen',
    recording: 'Aufnahme läuft',
    primaryNavigation: 'Hauptnavigation',
    closeNavigation: 'Navigation schließen',
    openNavigation: 'Navigation öffnen',
    themeFollowingSystem: 'Folgt dem System. Umschalten auf immer hell.',
    themeAlwaysLight: 'Immer hell. Umschalten auf immer dunkel.',
    themeAlwaysDark: 'Immer dunkel. Umschalten auf dem System folgen.',
    themeFollowingShort: 'Folgt dem System',
    resizeSidebar: 'Seitenleiste anpassen. Mit den Pfeiltasten ändern, mit Enter zurücksetzen.',
  },

  start: {
    eyebrow: 'Lokale KI für vertrauliche Besprechungsprotokolle',
    title: 'Besprechung beginnen',
    lead: 'Importieren Sie eine Audio- oder Videodatei. Prüfen Sie jeden Schritt, bevor daraus ein Protokoll wird.',
    importTitle: 'Aufnahme importieren',
    importDetail: 'Ein Projekt wählen — danach bleibt alles im Zusammenhang',
    recordTitle: 'Besprechung aufnehmen',
    recordDetail: 'Raum und Konferenz auf diesem Gerät aufnehmen, auf getrennten Spuren',
    promiseTitle: 'Ihre Arbeit bleibt auf diesem Gerät.',
    promiseDetail: 'Kein LocaLog-Konto, kein Cloud-Dienst, keine Telemetrie.',

    setupTitle: 'Ein Download vor der ersten Transkription',
    setupBody: (quality: string, size: string) =>
      `LocaLog transkribiert auf diesem Gerät, also muss das Modell darauf sein. Die Qualität „${quality}“ ist ${size} groß und wird einmal geladen. Sie können vorher eine Aufnahme importieren — gebraucht wird es erst, wenn die Transkription beginnt.`,
    setupDownload: (size: string) => `Jetzt laden (${size})`,
    setupCancel: 'Abbrechen',
    setupAside: 'Weitere Qualitäten und die Sprechertrennung finden Sie in den Einstellungen.',
  },
};
