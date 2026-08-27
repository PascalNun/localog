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
