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
  providerNeededForCorrections:
    'Starten Sie Ihre vorhandene Ollama-Installation, bevor Sie diese Schreibweisen prüfen.',
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
  protocolWouldNotFit: (sizes: string) => {
    const [expected, ceiling] = sizes.split('/').map((n) => Number(n).toLocaleString('de-DE'));
    return `Diese Besprechung ist so lang, dass ein Protokoll davon — etwa ${expected} Zeichen — nicht in eine Antwort passt; darin ist Platz für ungefähr ${ceiling}. Es wurde nichts versucht, denn das ist eine Rechnung und kein misslungener Durchlauf: ein erneuter Versuch scheiterte genauso. Wählen Sie einen knapperen Protokollstil, oder teilen Sie die Aufnahme.`;
  },
  generationConfigUnreadable:
    'Dieser Vorgang wurde von einer früheren Version von LocaLog vorbereitet und kann nicht gelesen werden. Es wurde nichts übernommen, Ihr Transkript ist unverändert. Starten Sie die Erzeugung erneut.',
  ollamaUnchecked: 'Ollama wurde noch nicht geprüft.',
  responseUnusable:
    'Das lokale Modell hat eine Antwort geliefert, die LocaLog nicht als Protokoll verwenden kann. Es wurde nichts übernommen, Ihr Transkript ist unverändert. Ein erneuter Versuch führt oft zum Ziel, da ein Modell jedes Mal anders antwortet.',
  generationRuntimeReady: 'Bereit, auf diesem Gerät Protokolle zu schreiben.',
  generationModelNotDownloaded: 'Es wurde noch kein Modell zum Schreiben von Protokollen geladen.',
  generationRuntimeMissing:
    'Es ist keine Laufzeit zum Schreiben von Protokollen installiert. LocaLog bringt eine mit; dieser Build findet sie nicht.',
  generationRuntimeNoPort:
    'Für die lokale Laufzeit war kein freier Port zu bekommen. Schließen Sie einige Programme und versuchen Sie es erneut.',
  generationRuntimeNotStarted: 'Die lokale Laufzeit konnte nicht gestartet werden.',
  generationRuntimeNeverReady:
    'Die lokale Laufzeit startete, war aber nie bereit. Ein sehr großes Modell auf einer langsamen Festplatte braucht Zeit; passiert es wieder, wählen Sie ein kleineres.',
  generationRuntimeStopped:
    'Die lokale Laufzeit hat sich beim Starten beendet. Die Modelldatei ist womöglich unvollständig — entfernen Sie sie und laden Sie sie erneut.',
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

  // Siehe die Anmerkung in en.ts: Sätze, die die Rust-Seite noch selbst schrieb.
  settingInvalid: 'Diese Laufzeiteinstellung kann nicht gespeichert werden.',
  meetingTitleRequiredToRecord:
    'Geben Sie der Besprechung einen Titel. Es gibt keine Datei, aus der einer übernommen werden könnte.',
  importSourceGone:
    'Wählen Sie die ursprüngliche Datei erneut, bevor Sie diesen Import wiederholen.',
  termProjectRequired: 'Wählen Sie das Projekt, zu dem dieser Begriff gehört.',
  termAlreadyPresent: 'Dieser Begriff steht hier bereits.',
  sourceRecordingRequired: 'Wählen Sie die Quellaufnahme erneut.',
  managedPathInvalid: 'Der Pfad zu dieser gespeicherten Datei ist ungültig.',
  documentChecksumFailed:
    'Ein gespeichertes Dokument hat seine lokale Integritätsprüfung nicht bestanden.',
  transcriptOutputInvalid:
    'Die Transkription hat etwas erzeugt, das LocaLog nicht als Transkript lesen kann.',
  speakerCountOutOfRange: 'Die erwartete Anzahl der Sprecher muss zwischen 2 und 64 liegen.',
  sourceNotCommitted: 'Übernehmen Sie die Quelle der Besprechung, bevor Sie sie transkribieren.',
  providerNeededForGeneration:
    'Starten Sie Ihre vorhandene Ollama-Installation, bevor Sie ein Protokoll erzeugen.',
  exportDestinationInvalid: 'Wählen Sie ein gültiges Exportziel.',
  exportFileExists:
    'Wählen Sie einen neuen Dateinamen. Eine vorhandene Datei wird nie ungefragt überschrieben.',
  exportFolderMissing: 'Der gewählte Exportordner ist nicht verfügbar.',
  processingBusy:
    'Es läuft bereits eine andere lokale Aufgabe. Warten Sie sie ab oder brechen Sie sie zuerst ab.',
  ffmpegMissingForRecording:
    'FFmpeg wird gebraucht, um eine Aufnahme abzuschließen, und wurde nicht gefunden.',

  // Die Ollama-Zeile in den Einstellungen. Siehe die Anmerkung in en.ts.
  ollamaNotRunning: (detail: string) =>
    `Starten Sie Ihre vorhandene Ollama-Installation und aktualisieren Sie dann.${detail ? ` ${detail}` : ''}`,
  ollamaModelsUnreadable: (detail: string) =>
    `Ollama läuft, hat aber nicht mitgeteilt, welche Modelle installiert sind.${detail ? ` ${detail}` : ''}`,
  ollamaReadyNoModel:
    'Ollama ist bereit. Wählen Sie ein installiertes Modell, um Protokolle zu erzeugen.',
  ollamaModelReady: 'Das gewählte lokale Modell ist bereit.',
  ollamaSelectedModelMissing:
    'Das gewählte Modell ist nicht installiert. Wählen Sie ein anderes, bereits installiertes Modell.',
};

export const de: Strings = {
  locale: 'de-DE',

  failures,

  /** Siehe die Anmerkung in en.ts: nach dem gespeicherten Wert geschlüsselt. */
  meetingLanguages: {
    English: 'Englisch',
    German: 'Deutsch',
    French: 'Französisch',
    Spanish: 'Spanisch',
    Italian: 'Italienisch',
    Dutch: 'Niederländisch',
    Portuguese: 'Portugiesisch',
    Polish: 'Polnisch',
    Danish: 'Dänisch',
    Swedish: 'Schwedisch',
    Norwegian: 'Norwegisch',
    Finnish: 'Finnisch',
    Czech: 'Tschechisch',
    Turkish: 'Türkisch',
    Japanese: 'Japanisch',
    Korean: 'Koreanisch',
    Chinese: 'Chinesisch',
    Arabic: 'Arabisch',
    Ukrainian: 'Ukrainisch',
  },
  dialog: {
    detectFromRecording: 'Aus der Aufnahme erkennen',
    chooseRecording: 'Aufnahme einer Besprechung wählen',
    audioAndVideo: 'Audio und Video',
    plainText: 'Nur Text',
    exportTitle: (title: string) => `${title} exportieren`,
  },

  settings: {
    memoryReported: (gb: number) => `${gb} GB Arbeitsspeicher erkannt`,
    themeAutomatic: 'Automatisch',
    themeLight: 'Hell',
    themeDark: 'Dunkel',
    modelSelected: 'Gewählt',
    useThisModel: 'Dieses Modell verwenden',
    useModel: 'Modell verwenden',
    catalogueNote:
      'Der Katalog ist bewusst ausgewählt. LocaLog lädt keine Modelle im Stillen herunter und zeigt keinen beliebigen Modellmarkt. Neue Einträge werden erst wählbar, wenn Laufzeit, Lizenz, Speicherbedarf und Qualität auf Deutsch und Englisch geprüft sind.',
    managedCopiesNote:
      'LocaLog hält verwaltete Kopien importierter Aufnahmen, aufbereiteter Audiodateien, Transkripte, Protokolle und geladener Modelle in seinem Anwendungsdatenordner. Exporte werden nur dorthin geschrieben, wo Sie es wählen.',
    discoveredRuntime: (path: string) => `Gefundene Laufzeit: ${path}`,
    runtimeVersion: (version: string) => `Version der Laufzeit: ${version}`,
    evaluatedIn: (languages: string) => `Geprüft in ${languages}`,
    evaluationPending: 'Qualitätsprüfung steht noch aus',
    otherModelNote:
      'Das ist für Menschen, die bereits wissen, welches lokale Modell sie ausprobieren wollen. Es wird von LocaLog nicht geprüft und nicht empfohlen, und es unterliegt denselben Grenzen von Laufzeit und Arbeitsspeicher.',
    qualityLead:
      'Wählen Sie die gewünschte Qualität. LocaLog lädt beim ersten Mal, was es braucht, und behält es auf diesem Gerät.',
    speakerDiscovery:
      'LocaLog findet die Sprechererkennung selbst — in den mitgelieferten Ressourcen oder im Systempfad. Sie ist optional und blockiert nie die Transkription.',
    noSpeakerRuntime: 'Auf diesem Gerät wurde noch keine passende Sprechererkennung gefunden.',
    readinessNote:
      'Zur Bereitschaftsprüfung gehört ein begrenzter Startversuch, damit ein unpassendes oder defektes Programm nicht als verfügbar gilt.',
    restoreSummary: (name: string, projects: number, meetings: number, version: string) =>
      `${name} enthält ${projects} Projekte und ${meetings} Besprechungen, gesichert aus LocaLog ${version}.`,
    restoreWarning:
      'Beim Zurückspielen werden die Projekte und Besprechungen in diesem Arbeitsbereich durch jene ersetzt. Nichts wird gelöscht — was hier liegt, bleibt in einem Ordner daneben erhalten —, aber LocaLog zeigt danach die zurückgespielte Arbeit, und Sie müssen es beenden und neu öffnen.',
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
    downloadModel: (size: string) => `Laden (${size})`,
    prepareSpeakerSeparation: 'Sprechertrennung vorbereiten',
    restoredBackup: (projects: number, meetings: number, previous: string) =>
      `${projects} Projekte und ${meetings} Besprechungen wiederhergestellt. Was hier war, wurde nach ${previous} verschoben und nicht gelöscht. Beenden Sie LocaLog und öffnen Sie es erneut, um mit dem wiederhergestellten Arbeitsbereich zu arbeiten.`,
    /** Siehe die Anmerkung in en.ts. */
    transcriptionPreset: {
      fast: { name: 'Schnell', detail: 'Schnelle Entwürfe, am sparsamsten mit dem Speicher' },
      balanced: { name: 'Ausgewogen', detail: 'Für den Besprechungsalltag' },
      accurate: { name: 'Genau', detail: 'Beste Qualität, am langsamsten' },
    },
    downloadingPreset: (name: string) => `${name} wird geladen`,
    /** Siehe die Anmerkung in en.ts. */
    modelDescription: {
      'gemma4-12b':
        'Das genaueste und beständigste der gemessenen Modelle: Über drei Läufe hat es 27 bis 31 der 35 Zahlen einer Besprechung erhalten, wo das nächstbeste auf nur 6 kam. Langsamer — etwa vierzehn Minuten für eine achtzigminütige Besprechung.',
      'ministral-8b':
        'An einer deutschen Besprechung mit drei Einstellungen gemessen und bei einer davon ein brauchbares Protokoll geschrieben: die anderen ergaben einen zweizeiligen Rumpf und ein JSON-Dokument, wo Markdown verlangt war. Bleibt als europäischer Kandidat, noch keine Alternative zur Grundlage.',
      'qwen3.5-4b':
        'Das schnellste gemessene Modell, etwa fünf Minuten für eine achtzigminütige Besprechung, und die Wahl, wenn der Speicher knapp ist. Die Tabelle der nächsten Schritte, die der formale Stil verlangt, hat es nie erzeugt.',
      'ministral-3b': 'Der erste europäische Kandidat für den schwächsten unterstützten Mac.',
      'granite4.1-8b':
        'An einer deutschen Besprechung mit drei Einstellungen gemessen und bei identischer Eingabe 22, 19 und 6 der 35 genannten Zahlen erhalten. Ein Lauf, der fünf Sechstel des Gesagten verliert, ist kein Werkzeug, um eine Aufzeichnung zu erstellen, und wird deshalb nicht empfohlen.',
      'llama-8b': 'Ein späterer Vergleichsplatz für eine geprüfte Llama-Veröffentlichung.',
    },
    modelOrigin: {
      international: 'Internationales offenes Modell',
      european: 'Europäisches Modell',
    },
    modelLicence: {
      apache2: 'Apache 2.0',
      gemma: 'Gemma-Nutzungsbedingungen',
      modelSpecific: 'Modellabhängig',
    },
    modelLanguage: {
      de: 'Deutsch',
      en: 'Englisch',
      ja: 'Japanisch',
      more: 'viele weitere',
    },
    modelStatus: {
      installed: 'Installiert',
      notInstalled: 'Nicht installiert',
      plannedCandidate: 'Geplanter Kandidat',
    },
    modelSizeInstalled: (gb: string) => `etwa ${gb} GB installiert`,
    modelSizeSmall: 'kleines Modell für das Gerät',
    modelSizeLarger: 'größeres lokales Modell',
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
    whisperExecutablePlaceholder: '/pfad/zu/whisper-cli',
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
    deleteMeeting: (title: string) => `${title} löschen`,
    deleteWarning:
      'Eine Besprechung zu löschen entfernt ihre Aufnahme, ihr Transkript und jede Protokollfassung von diesem Gerät. Das lässt sich nicht rückgängig machen.',
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

  jobErrors: {
    interrupted: {
      title: 'Die Übernahme wurde unterbrochen',
      detail:
        'LocaLog wurde beendet, bevor die verwaltete Kopie festgeschrieben war. Das externe Original ist unverändert, und Sie können es gefahrlos erneut versuchen.',
    },
    permission_denied: {
      title: 'LocaLog konnte die Aufnahme nicht lesen oder ablegen',
      detail:
        'Prüfen Sie den Zugriff auf die gewählte Datei und auf den lokalen Datenordner von LocaLog, und versuchen Sie es erneut. Das externe Original wurde nicht verändert.',
    },
    insufficient_space: {
      title: 'Es ist nicht genug lokaler Speicher vorhanden',
      detail:
        'Geben Sie Speicher frei und versuchen Sie es erneut. Es wurde keine unvollständige Aufnahme als fertig ausgegeben.',
    },
    source_missing: {
      title: 'Die gewählte Aufnahme ist nicht mehr verfügbar',
      detail:
        'Legen Sie die Datei wieder an ihren Ort, oder importieren Sie die Besprechung neu. Die Besprechung bleibt sicher im Entwurf.',
    },
    source_reselection_required: {
      title: 'Wählen Sie die Aufnahme erneut',
      detail:
        'Diese Besprechung stammt aus einem früheren Entwicklungsstand, der den Ort der Quelldatei nicht festgehalten hat. Wählen Sie die Aufnahme erneut, um fortzufahren; die Besprechung ist erhalten.',
    },
    unsupported_media: {
      title: 'Dieses Medienformat wird noch nicht unterstützt',
      detail:
        'Wählen Sie eine gängige Audio- oder Videodatei. Das externe Original wurde nicht verändert.',
    },
    empty_source: {
      title: 'Die gewählte Aufnahme ist leer',
      detail:
        'Wählen Sie eine Aufnahme, die Audio- oder Videodaten enthält. Die leere externe Datei wurde nicht verändert.',
    },
    synthetic_failure: {
      title: 'Der Entwicklungsadapter hat wie gewünscht angehalten',
      detail:
        'Der ausgelöste Fehler trat auf, bevor eine Fassung festgeschrieben war. Ihre Quelle und Ihr letzter stabiler Stand sind sicher, und Sie können es erneut versuchen.',
    },
    invalid_adapter_output: {
      title: 'Die lokale Ausgabe konnte nicht geprüft werden',
      detail:
        'LocaLog hat das unvollständige Ergebnis nicht übernommen. Ihre letzte stabile Quelle und die Dokumentfassungen bleiben sicher.',
    },
    runtime_missing: {
      title: 'Wählen Sie eine lokale Transkriptionslaufzeit',
      detail:
        'Wählen Sie in den Einstellungen unter Transkription eine installierte whisper.cpp-Anwendung. LocaLog lädt keine Laufzeiten herunter.',
    },
    model_missing: {
      title: 'Wählen Sie ein lokales Transkriptionsmodell',
      detail:
        'Wählen Sie in den Einstellungen unter Transkription ein bereits vorhandenes whisper.cpp-Modell. Es wurde nichts heruntergeladen oder geändert.',
    },
    runtime_changed: {
      title: 'Die Transkriptionslaufzeit hat sich geändert',
      detail:
        'Der eingereihte Vorgang wurde nicht ausgeführt, weil die whisper.cpp-Anwendung nicht mehr zur festgehaltenen Laufzeit passt. Versuchen Sie es erneut, um die aktuelle Laufzeit zu übernehmen.',
    },
    model_changed: {
      title: 'Das Transkriptionsmodell hat sich geändert',
      detail:
        'Der eingereihte Vorgang wurde nicht ausgeführt, weil das Modell nicht mehr zur festgehaltenen Prüfsumme passt. Versuchen Sie es erneut, um das aktuelle Modell zu übernehmen.',
    },
    media_probe_failed: {
      title: 'Die Aufnahme konnte nicht untersucht werden',
      detail:
        'Prüfen Sie, ob FFprobe installiert und die übernommene Quelle noch lesbar ist. Das Original bleibt unverändert.',
    },
    normalization_failed: {
      title: 'Die Aufnahme konnte nicht vorbereitet werden',
      detail:
        'Prüfen Sie, ob FFmpeg installiert ist, und versuchen Sie es erneut. Die aufbereitete Fassung lässt sich neu erzeugen, das Original bleibt unverändert.',
    },
    transcription_failed: {
      title: 'Die lokale Transkription konnte nicht abschließen',
      detail:
        'Die whisper.cpp-Laufzeit hat aufgehört, bevor eine Transkriptfassung festgeschrieben war. Prüfen Sie ihr Modell und versuchen Sie es erneut.',
    },
    transcription_timeout: {
      title: 'Die lokale Transkription hat zu lange gedauert',
      detail:
        'Der überwachte Transkriptionsvorgang wurde beendet, bevor eine Fassung festgeschrieben war. Prüfen Sie Aufnahme und Laufzeit, und versuchen Sie es erneut.',
    },
    provider_model_missing: {
      title: 'Das gewählte lokale Modell ist nicht verfügbar',
      detail:
        'Das gewählte Ollama-Modell ist nicht mehr installiert. Wählen Sie in den Einstellungen unter Protokollerzeugung ein installiertes Modell und versuchen Sie es erneut.',
    },
    provider_model_changed: {
      title: 'Das lokale Modell hat sich geändert',
      detail:
        'Die Prüfsumme des Modells hat sich geändert, nachdem dieser Vorgang eingereiht wurde. Versuchen Sie es erneut, um das aktuelle Modell zu übernehmen.',
    },
    provider_runtime_changed: {
      title: 'Der lokale Anbieter hat sich geändert',
      detail:
        'Die Version der Ollama-Laufzeit hat sich geändert, nachdem dieser Vorgang eingereiht wurde. Versuchen Sie es erneut, um die aktuelle Laufzeit zu übernehmen.',
    },
    provider_unavailable: {
      title: 'Die lokale Protokollerzeugung konnte keine Verbindung herstellen',
      detail:
        'Starten Sie Ihre vorhandene Ollama-Installation und versuchen Sie es erneut. LocaLog startet und lädt keine Laufzeiten.',
    },
    provider_invalid_output: {
      title: 'Die Ausgabe des lokalen Modells konnte nicht geprüft werden',
      detail:
        'LocaLog hat das unvollständige oder fehlerhafte Protokoll nicht übernommen. Ihr Transkript bleibt sicher, und Sie können es erneut versuchen.',
    },
    provider_incomplete_output: {
      title: 'Die Ausgabe des lokalen Modells konnte nicht geprüft werden',
      detail:
        'LocaLog hat das unvollständige oder fehlerhafte Protokoll nicht übernommen. Ihr Transkript bleibt sicher, und Sie können es erneut versuchen.',
    },
    provider_response_too_large: {
      title: 'Die Antwort des lokalen Modells war zu groß',
      detail:
        'Die Antwort hat die sichere Grenze von LocaLog überschritten und wurde nicht übernommen. Versuchen Sie es mit einem kürzeren Transkript oder einem anderen lokalen Modell.',
    },
    invalid_transcript_output: {
      title: 'Die Transkriptionsausgabe konnte nicht geprüft werden',
      detail:
        'LocaLog hat die Ausgabe der Laufzeit nicht übernommen, weil sie unvollständig oder fehlerhaft war. Ihre Quelle bleibt sicher.',
    },
    processing_failed: {
      title: 'Die lokale Verarbeitung konnte nicht abschließen',
      detail:
        'Es wurde kein unvollständiges Transkript und kein unvollständiges Protokoll als fertig ausgegeben. Ihr letzter stabiler Stand bleibt verfügbar, und Sie können es erneut versuchen.',
    },
    unknown: {
      title: 'Die Übernahme konnte nicht abschließen',
      detail:
        'Die Besprechung bleibt im Entwurf, und das externe Original wurde nicht verändert. Sie können es gefahrlos erneut versuchen.',
    },
  },

  jobStages: {
    transcriptSaved: 'Transkript gespeichert',
    protocolSaved: 'Protokoll gespeichert',
    importComplete: 'Übernahme abgeschlossen — Original unverändert',
    processingCancelled: 'Die lokale Verarbeitung wurde abgebrochen — der stabile Stand bleibt',
    processingInterrupted: 'Die lokale Verarbeitung wurde unterbrochen — der stabile Stand bleibt',
    processingFailed: 'Die lokale Verarbeitung konnte nicht abschließen — der stabile Stand bleibt',

    ready_to_import: 'Bereit, die Aufnahme zu übernehmen',
    copying: 'Die Aufnahme wird übernommen',
    stoppingSafely: 'Wird sicher beendet',
    temporary_complete: 'Fast geschafft',
    finalizing: 'Die Aufnahme wird sicher abgelegt',
    duplicate_confirmation: 'Diese Aufnahme liegt möglicherweise schon hier',
    completed: 'Die Aufnahme ist da',
    cancelled: 'Übernahme abgebrochen — Original unverändert',
    interrupted: 'Übernahme unterbrochen — Original unverändert',
    failed: 'Übernahme konnte nicht abschließen — Original unverändert',
    probing_media: 'Die Aufnahme wird angesehen',
    normalizing_audio: 'Das Audio wird vorbereitet',
    output_staged: 'Wird sicher gespeichert',

    transcription_queued: 'Bereit zum Transkribieren',
    checking_source: 'Die Aufnahme wird geprüft',
    loading_transcription_model: 'Das Modell wird geladen',
    transcribing_audio: 'Wird transkribiert',
    separating_speakers: 'Die Sprecher werden unterschieden',
    validating_transcript: 'Das Transkript wird gespeichert',
    preparing_fake_transcriber: 'Wird vorbereitet',
    transcribing_synthetic_segments: 'Transkriptabschnitte werden erzeugt',

    generation_queued: 'Bereit, das Protokoll zu schreiben',
    checking_transcript: 'Das Transkript wird geprüft',
    resolving_protocol_inputs: 'Stil und Begriffe werden zusammengestellt',
    condensing_transcript: 'Die Besprechung wird durchgelesen',
    generating_protocol: 'Der Protokollentwurf wird geschrieben',
    validating_protocol: 'Das Protokoll wird gespeichert',
    reading_introductions: 'Es wird gelesen, wer sich vorgestellt hat',

    protocol_would_not_fit: 'Diese Besprechung ist länger, als ein Durchgang fassen kann',
    segments_no_subject_claimed: 'Ein Teil der Besprechung fiel unter kein Thema',
    sections_over_their_length: 'Einige Abschnitte sind länger geraten als vorgesehen',

    finding_subjects: (detail: string) =>
      detail
        ? `Es wird gesucht, worum es ging — Passage ${detail}`
        : 'Es wird gesucht, worum es ging',
    writing_section: (detail: string) =>
      detail
        ? `${detail} wird geschrieben`
        : 'Das Protokoll wird Abschnitt für Abschnitt geschrieben',
    joining_subjects: (detail: string) =>
      detail
        ? `Zusammengehörige Themen werden verbunden — ${detail} gefunden`
        : 'Zusammengehörige Themen werden verbunden',
    joined_subjects: (detail: string) =>
      detail ? `Themen verbunden — ${detail}` : 'Themen verbunden',
    joining_failed: (detail: string) =>
      detail
        ? `Die Themen konnten nicht verbunden werden — ${detail}`
        : 'Die Themen konnten nicht verbunden werden',

    working: 'Läuft',
  },

  stages: {
    label: 'Schritte der Besprechung',
    source: 'Quelle',
    transcript: 'Transkript',
    protocol: 'Protokoll',
  },

  progress: {
    needsAttention: 'Braucht Aufmerksamkeit',
    backgroundWork: 'Arbeit im Hintergrund',
    cancellingSafely: 'Wird sicher abgebrochen …',
    cancel: 'Abbrechen',
    speakerPassNote:
      'Dieser Durchgang liest die ganze Aufnahme, um Sprecherwechsel zu vergleichen. Bei langen Aufnahmen kann das einige Minuten dauern. Sie können jederzeit gefahrlos abbrechen.',
    latestRetained: 'Der letzte stabile Stand bleibt erhalten',
    originalUnchanged: ' · externes Original unverändert',
    retry: 'Erneut versuchen',
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
    namesHeading: 'Namen & Begriffe',
    namesLead:
      'Eine Transkription kann keinen Namen erraten, den sie nie gehört hat. Diese Angaben jetzt zu machen ist die nützlichste Minute, die Sie in dieses Projekt stecken können: Ein falsch gehörter Name steht in jedem Protokoll zu dieser Aufnahme genauso, und kein späterer Schritt holt ihn zurück.',
    namesPeople: 'Personen',
    namesPeopleHint: 'Alle, die voraussichtlich im Raum sind oder genannt werden.',
    namesOrganisations: 'Firmen und Auftraggeber',
    namesOrganisationsHint: 'Der Auftraggeber, die weiteren Fachplaner, die Lieferanten.',
    namesProject: 'Dieses Projekt',
    namesProjectHint: 'Wie das Projekt, das Grundstück oder das Gebäude heißt.',
    namesTerms: 'Weitere Begriffe, die richtig geschrieben werden sollen',
    namesTermsHint: 'Fachwörter dieser Arbeit, die eine allgemeine Transkription nicht kennt.',
    namesNote:
      'Mit Kommas trennen. Alles freiwillig, und nichts davon ist endgültig: Sie können jederzeit unter „Namen & Begriffe“ ergänzen und korrigieren, und eine Korrektur beim Durchsehen eines Transkripts wird hier ebenfalls übernommen.',
    creating: 'Wird angelegt …',
    createAndContinue: 'Anlegen und fortfahren',
    afterCreated:
      'Ein Protokollstil sowie die Namen und Begriffe dieser Arbeit lassen sich für das Projekt festlegen, sobald es angelegt ist. Die Namen sind eine Minute wert: Sie sind das, was eine Transkription nicht erraten kann.',
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
    font: 'Schrift',
    appliesToProject: (project: string) =>
      `Gilt für jedes Protokoll in ${project}, damit die Dokumente eines Hauses gleich aussehen. Es ändert, wie das Protokoll gesetzt ist, nie was darin steht — das ist der Stil darüber.`,
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
    browserPreview: 'Browser-Vorschau',
    speakersEstimateNote:
      'LocaLog fasst die gehörten Stimmen zusammen und zählt sie. Eine Schätzung — Sie können sie durch eine Zahl ersetzen, wenn sie nicht stimmt.',
    speakersCountNote:
      'Ihre beste Schätzung genügt — es ist die Zahl der Stimmen, nach denen LocaLog sucht. Zu viele können eine Person aufteilen, zu wenige zwei Personen zusammenlegen.',
    speakersTogetherNote: 'Das Transkript behält eine einzige Sprecherbezeichnung.',
    importInterrupted:
      'LocaLog wurde geschlossen, bevor die verwaltete Kopie übernommen war. Die Besprechung bleibt ein Entwurf, und der Import kann gefahrlos wiederholt werden.',
    importCancelled:
      'Die verwaltete Kopie wurde abgebrochen. Die Besprechung bleibt ein Entwurf, und die externe Datei wurde nicht verändert.',
    importFailed:
      'Die verwaltete Kopie konnte nicht übernommen werden. Die Besprechung bleibt ein Entwurf, und die externe Datei wurde nicht verändert.',
    importRunning:
      'LocaLog kopiert diese Quelle in den eigenen verwalteten Speicher. Sie ist erst bereit, wenn die Kopie geprüft und übernommen wurde.',
    sourceStored:
      'ist sicher bei dieser Besprechung abgelegt. Das externe Original wurde nicht verändert.',
    sourceSynthetic:
      'ist dieser synthetischen Browser-Besprechung zugeordnet. Es wurde keine echte Mediendatei kopiert.',
    syntheticFixture: 'Synthetische Vorlage',
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
    meetingOverride: 'Abweichend für diese Besprechung',
    preparing: 'Wird vorbereitet …',
    bringingRecordingIn: 'Aufnahme wird übernommen …',
    noPerMeetingOverrides:
      'Abweichungen je Besprechung und die Wahl von Namen & Begriffen je Besprechung gibt es noch nicht.',
    chosenOnceNote:
      'Die Transkriptionsqualität und das Modell, das das Protokoll schreibt, werden einmal in den Einstellungen gewählt und für jede Besprechung verwendet.',
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
    lead: 'Schneiden Sie weg, was die Besprechung nicht braucht, bevor sie transkribiert wird. Ihre Aufnahme wird dabei nie verändert — alles hier lässt sich rückgängig machen.',
    noPreparedAudio:
      'Für diese Besprechung gibt es noch kein aufbereitetes Audio zum Prüfen. Es steht bereit, sobald der Import übernommen wurde.',
    dragToSelect:
      'Ziehen Sie über die Aufnahme, um einen Abschnitt zu wählen, oder benutzen Sie die Pfeiltasten mit gedrückter Umschalttaste.',
    selectedRange: (from: string, to: string) => `Gewählt: ${from} bis ${to}.`,
    eyebrow: 'Aufnahme',
    heading: 'Aufnahme prüfen',
    noAudio: 'Noch keine Arbeitsdatei',
    waveformLabel: 'Die Aufnahme. Mit den Pfeiltasten bewegen, mit Umschalt auswählen.',
    keptOf: (kept: string, whole: string) => `${kept} von ${whole} behalten`,
    startsAt: (time: string) => `Beginnt bei ${time}`,
    endsAt: (time: string) => `Endet bei ${time}`,
    removedSpan: (from: string, to: string) => `${from} bis ${to} entfernt`,
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
    heardAs: (heard: string) => `Verstanden als „${heard}“`,
    askAboutTheRest: 'Den Rest prüfen lassen',
    askingAboutTheRest: 'Die Sätze werden gelesen …',
    askAboutTheRestNote:
      'Einige Wörter werden jedes Mal anders verhört, deshalb findet sie keine Korrektur einer Schreibweise. Hier wird jedes davon in seinem eigenen Satz gelesen und ein Name aus der Liste dieses Projekts vorgeschlagen — etwas anderes kann nicht vorgeschlagen werden, und geändert wird nichts, bevor Sie es sagen.',
    proposedNothing: 'Es wurde nichts weiter erkannt.',
    proposedNothingNote:
      'Das ist die übliche und eine gute Antwort: Vorgeschlagen werden darf nur ein Name, den dieses Projekt bereits führt — also bleibt es lieber still, als einen zu erfinden.',
    proposalsHeading: (count: number) => (count === 1 ? '1 Vorschlag' : `${count} Vorschläge`),
    proposalSuggests: (heard: string, suggested: string) => `${heard} → ${suggested}`,
    spellingsToCheck: (count: number) =>
      count === 1 ? '1 Schreibweise zum Prüfen' : `${count} Schreibweisen zum Prüfen`,
    questionedByProtocol: 'das Protokoll kennt dieses Wort nicht',
    autosaveFailed:
      'Automatisches Speichern fehlgeschlagen — Ihr zuletzt gespeicherter Stand ist unversehrt',
    correctCount: (count: number) => `${count} korrigieren`,
    audioCouldNotLoad: 'Das Arbeitsaudio dieser Besprechung konnte nicht geladen werden.',
    pauseAudio: 'Wiedergabe anhalten',
    playAudio: 'Wiedergabe starten',
    saving: 'Wird gespeichert …',
    editsSaved: 'Änderungen gespeichert',
    revisionSaved: 'Transkriptfassung gespeichert',
    separationUnavailableHere:
      'Die Sprechertrennung ist in dieser Installation noch nicht verfügbar. Sie können mit selbst vergebenen Bezeichnungen weiterarbeiten.',
    rerunForSeparation:
      'Transkribieren Sie erneut, um ein aktuelles Ergebnis der Sprechertrennung festzuhalten.',
    separationUnavailableForRun:
      'Für diesen Durchlauf war keine Sprechertrennung verfügbar. Sie können mit selbst vergebenen Bezeichnungen weiterarbeiten.',
    nothingChangedYet: 'Noch nichts geändert',
    readingOpening: 'Der Anfang wird gelesen …',
    readWhoIsHere: 'Lesen, wer in dieser Besprechung ist',
    correcting: 'Wird korrigiert …',
    durationPending: 'Dauer wird noch ermittelt',
    introducedThemselves: (count: number) => `${count} haben sich vorgestellt`,
    noNamesYet: (project: string) => `Noch keine Namen für ${project}`,
    speltAsHeard:
      'So geschrieben, wie die Transkription sie gehört hat. Korrigieren Sie, was falsch ist — es wird hier berichtigt und für dieses Projekt gemerkt.',
    openingNote:
      'Besprechungen beginnen meist damit, dass die Anwesenden sagen, wer sie sind. Daraus bekommt dieses Projekt seine Namen — genau das, was eine Transkription nicht erraten kann.',
    foundInPlaces: (count: number) =>
      `An ${count} ${count === 1 ? 'Stelle' : 'Stellen'} gefunden. Haken Sie ab, was so bleiben soll.`,
    noneMisheardEveryTime: (count: number) =>
      `Kein Wort wurde jedes Mal falsch gehört. ${count} Passagen sind aus anderen Gründen weiterhin als unklar markiert.`,
    nothingFlaggedNote:
      'Nichts wurde als unklar markiert. Ein Transkript, das vor dieser Aufzeichnung entstand, zeigt hier ebenfalls nichts — ein älteres liest man daher besser, als dass man ihm vertraut.',
    workingAudioLater:
      'Die Arbeitsdatei steht zur Verfügung, sobald diese Besprechung transkribiert wurde.',
    recordingEndsNote:
      'Wenn die Besprechung darüber hinaus weiterging, hat die Aufnahme das nicht erfasst — und das Protokoll enthält es nicht.',
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
    unsureNames: 'Namen, die einen zweiten Blick verdienen',
    whatShouldItSay: 'Wie soll es heißen?',
    rememberForProject:
      'Für dieses Projekt merken, damit die nächste Besprechung es richtig schreibt',
    areAnyNames: 'Sind das Namen? Eine Korrektur bessert dieses Transkript aus und wird gemerkt.',
    nothingToCheck: 'Nichts zu prüfen',
    correctSpelling: 'Schreibweise korrigieren',
    checkWording: 'Formulierung prüfen',
    checkWords: (words: string) => `${words} prüfen`,
    textAt: (time: string) => `Transkripttext bei ${time}`,
    jumpTo: (time: string) => `Zu ${time} springen`,
    removeLineAt: (time: string) => `Die Zeile bei ${time} entfernen`,
    renameSpeaker: (speaker: string) => `${speaker} umbenennen`,
    nameHeardAs: (heard: string) => `Name, gehört als ${heard}`,
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
    remove: 'Entfernen',
    edit: 'Bearbeiten',
    keep: 'Behalten',
    notInUseSuffix: ' · nicht in Verwendung',
    /** Siehe die Anmerkung in en.ts: nur solange ein Stil nicht umbenannt wurde. */
    shippedStyle: {
      'style-formal': {
        name: 'Förmliches Protokoll',
        description: 'Gegliederte Aufzeichnung von Erörterung, Beschlüssen und Aufgaben.',
      },
      'style-working-note': {
        name: 'Interne Arbeitsnotiz',
        description: 'Knappe Arbeitsaufzeichnung für ein internes Projektteam.',
      },
      'style-decision-log': {
        name: 'Technisches Entscheidungsprotokoll',
        description: 'Betont Alternativen, Randbedingungen und ausdrückliche Entscheidungen.',
      },
    },
    copyOf: (name: string) => `${name} (Kopie)`,
    enterATerm: 'Geben Sie einen Begriff ein.',
    reading: 'Wird gelesen …',
    editTerm: 'Begriff bearbeiten',
    inUse: 'In Verwendung',
    notInUse: 'Nicht in Verwendung',
    instructionsGiven:
      'Das sind die Anweisungen, die das Modell erhält, in der Reihenfolge, in der es sie erhält',
    asShipped: ', genau so, wie dieser Stil ausgeliefert wurde',
    invariantsNote:
      'Diese gehören nicht zu diesem Stil und lassen sich hier nicht ändern — sie werden gar nicht bei einem Stil gespeichert. Sie kommen zu jedem Protokoll hinzu, während es geschrieben wird: Ein Dokument, das eine Entscheidung festhält, die niemand getroffen hat, ist kein anders gesetztes Protokoll, sondern ein falsches.',
    whichTermsHelp:
      'Am meisten helfen Namen, Firmen und Abkürzungen. Übliche Fachbegriffe werden meist auch ohne Eintrag richtig transkribiert.',
    termsLeadLong:
      'Tragen Sie die Namen, Firmen und Abkürzungen dieser Arbeit ein, damit sie richtig transkribiert werden. In einer echten achtzigminütigen Besprechung wurde der Projektname damit von nie richtig geschrieben zu immer.',
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
    insertInto: (where: string) => `Einen Wert in ${where} einfügen`,
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

    notSelected: 'Nicht gewählt',

    jobNeedsDecision: 'Braucht Ihre Entscheidung',
    jobReadyToContinue: 'Bereit zum Fortfahren',
    jobCancelling: 'Wird sicher abgebrochen',

    formatWordDocument: 'Word-Dokument',
    formatPlainText: 'Reiner Text',
    exportSaved: (format: string) => `Export als ${format} gespeichert`,
    exportFailed: (format: string, why: string) => `Export als ${format} fehlgeschlagen: ${why}`,
    exportPrepared: (format: string) => `Export als ${format} vorbereitet`,
    exportNeedsDesktop: (format: string) =>
      `Der Export als ${format} braucht die Desktop-Anwendung.`,

    meetingArchived: 'Besprechung archiviert. Sie liegt in den Einstellungen unter Speicher.',
    projectArchived: 'Projekt archiviert. Es liegt in den Einstellungen unter Speicher.',
    transcriptExported: 'Transkript exportiert',
  },

  protocol: {
    undo: 'Rückgängig',
    redo: 'Wiederherstellen',
    next: 'Weiter',
    blockParagraph: 'Absatz',
    blockHeading1: 'Überschrift 1',
    blockHeading2: 'Überschrift 2',
    blockHeading3: 'Überschrift 3',
    figuresMissingFromRewrite: (count: number) =>
      `${count} Zahlen, die in der Passage standen, fehlen in dieser Neuformulierung`,
    markdownView: 'Markdown-Ansicht',
    documentView: 'Dokumentansicht',
    looking: 'Wird gesucht …',
    replaceAll: 'Alle ersetzen',
    rewrite: 'Neu formulieren',
    rewriting: 'Wird neu formuliert',
    figureMissingFromRewrite:
      'Eine Zahl, die in der Passage stand, fehlt in dieser Neuformulierung',
    reviewedRevisionPreserved:
      'Die geprüfte Fassung bleibt erhalten. Diese Zwischenstände sind nicht geprüft.',
    thisRevisionReviewed: 'Genau diese unveränderliche Fassung wurde als geprüft markiert.',
    generatedStaysEditable: 'Erzeugte Inhalte bleiben prüfbar und bearbeitbar.',
    notFound: 'Nicht gefunden',
    matchCount: (count: number) => `${count} ${count === 1 ? 'Treffer' : 'Treffer'}`,
    replacedCount: (count: number) => ` · ${count} ersetzt`,
    changesNotYetMade: (count: number) =>
      `${count} ${count === 1 ? 'Änderung' : 'Änderungen'}, noch nicht vorgenommen`,
    compoundNote:
      'Ein großgeschriebener Name wird auch innerhalb von Zusammensetzungen gesucht — dort übersieht ihn ein einfaches Ersetzen. Lesen Sie sie, dann übernehmen oder belassen Sie sie.',
    andMore: (count: number) => `und ${count} weitere, alle in denselben beiden Formen.`,
    passageGoesAlone:
      'Die Passage geht allein an Ihr lokales Modell. Zahlen, Namen und Daten sollen unverändert zurückkommen — prüfen Sie das, und machen Sie es rückgängig, wenn nicht.',
    nothingChangedYet:
      'Es wurde noch nichts geändert. Lesen Sie es, dann übernehmen oder belassen Sie es — ein lokales Modell formuliert gut um und ist trotzdem nicht blind zu übernehmen.',
    secondPassNote:
      'Von Ihrem eigenen Modell gefragt — und es irrt in beide Richtungen: Es übersieht Änderungen und beanstandet Formulierungen, die in Ordnung sind. Ein Blick wert, kein Urteil.',
    pageEdgesNote:
      'Wo die Seiten enden würden, gemessen so, wie das Druck-Stylesheet sie setzt: Eine Überschrift oder eine Tabelle rutscht als Ganzes nach unten statt umzubrechen, Fließtext nicht. Die letzten ein, zwei Zeilen entscheidet der Drucker — nehmen Sie das also auf eine Zeile genau, nicht exakt.',
    transcriptSourceNote:
      'Geschrieben aus dem geprüften Transkript dieser Besprechung. Nirgends ist festgehalten, welche Passage welchen Satz hervorgebracht hat — was folgt, sucht daher nach den Worten, statt es zu behaupten. Eine Umschreibung findet nichts, und das ist die ehrliche Antwort.',
    noWordsTogether:
      'Diese Worte kommen im Transkript nirgends zusammen vor. Meist heißt das, der Entwurf hat es in eigene Worte gefasst, was ihm zusteht — nachprüfen lässt es sich an der Aufnahme.',
    revisionNote:
      'Getipptes bleibt als laufende Bearbeitung erhalten und erzeugt keine Fassung. Eine Fassung entsteht, wenn ein Entwurf erzeugt wird, wenn Sie darum bitten, wenn Sie ein Protokoll als geprüft markieren und wenn eine ältere wiederhergestellt wird — so bleibt diese Liste kurz genug zum Lesen.',
    nothingRewrites:
      'Hier schreibt nichts Ihren Text für Sie um. Der Entwurf gehört Ihnen zum Bearbeiten, und jede Fassung bleibt erhalten.',
    figuresKept: (kept: number, stated: number) => `${kept} von ${stated} Zahlen übernommen`,
    figuresNote: (stated: number, kept: number) =>
      `In der Besprechung fielen ${stated} Zahlen, und dieser Entwurf greift ${kept} davon auf. Wie viele hierher gehören, hängt vom gewählten Stil ab — das ist also etwas zum Ansehen, keine Bewertung.`,
    figuresInvented: (count: number) =>
      count === 1
        ? 'Hier steht eine Zahl, die in der Besprechung nicht fiel'
        : `Hier stehen ${count} Zahlen, die in der Besprechung nicht fielen`,
    confirmAgainstRecording: '. Sollte an der Aufnahme geprüft werden.',
    tasksUnowned: (count: number) =>
      count === 1
        ? 'Eine Aufgabe hier hat niemanden zugeordnet'
        : `${count} Aufgaben hier haben niemanden zugeordnet`,
    unownedNote:
      '. Der Entwurf lässt die Zuordnung lieber offen, als sie zu raten — es kann also genau so beschlossen worden sein. Und einen Namen dazuzuschreiben ist jetzt weit billiger als in der nächsten Besprechung.',
    editor: 'Protokolleditor',
    markdownBacked: 'auf Markdown gestützt',
    noteMissingTableHeading: 'Keine Tabelle der nächsten Schritte',
    noteMissingTableBody:
      'Dieses Protokoll wurde dreimal geschrieben, und keiner der Durchläufe endete mit einer Tabelle der vereinbarten Aufgaben und ihrer Verantwortlichen. Was die Besprechung vereinbart hat, steht in den Abschnitten oben, ist hier aber nicht zusammengefasst.',
    noteGapsHeading: 'Nicht in diesem Protokoll enthalten',
    noteOneGap:
      'Ein Abschnitt der Aufnahme konnte nicht ausgewertet werden, und nichts oben beschreibt ihn. Die Aufnahme selbst ist vollständig und kann weiterhin abgehört werden.',
    noteSeveralGaps:
      'Mehrere Abschnitte der Aufnahme konnten nicht ausgewertet werden, und nichts oben beschreibt sie. Die Aufnahme selbst ist vollständig, diese Abschnitte können weiterhin abgehört werden.',
    documentType: 'Protokoll',
    statusDraft: 'Entwurf',
    statusReviewed: 'Geprüft',
    statusChanged: 'Seit der Prüfung geändert',
    fieldProjectName: 'Projektname',
    fieldMeetingTitle: 'Titel der Besprechung',
    fieldMeetingDate: 'Datum der Besprechung',
    fieldDocumentType: 'Art des Dokuments',
    fieldProtocolStatus: 'Stand',
    fieldPageNumber: 'Seitenzahl',
    fieldPageOfCount: 'Seite n von m',
    fieldText: 'Eigener Text',
    showPageBreaks: 'Seitenumbrüche anzeigen',
    hidePageBreaks: 'Seitenumbrüche ausblenden',
    saving: 'Wird gespeichert …',
    autosaveFailed: 'Automatisches Speichern fehlgeschlagen',
    workingEditsSaved: 'Änderungen zwischengespeichert',
    revisionSaved: 'Fassung gespeichert',
    editorTools: 'Werkzeuge',
    find: 'Suchen',
    findInProtocol: 'Im Protokoll suchen',
    replaceWith: 'Ersetzen durch',
    makeChanges: 'Diese Änderungen vornehmen',
    leaveIt: 'Belassen',
    zoomOut: 'Verkleinern',
    zoomIn: 'Vergrößern',
    insertTable: 'Tabelle einfügen',
    insertDivider: 'Trennlinie einfügen',
    documentMenu: 'Dokumentmenü',
    clearFormatting: 'Formatierung entfernen',
    table: 'Tabelle',
    blockType: 'Absatzart',
    addColumnLeft: 'Spalte links einfügen',
    addColumnRight: 'Spalte rechts einfügen',
    deleteColumn: 'Diese Spalte löschen',
    addRowAbove: 'Zeile darüber einfügen',
    addRowBelow: 'Zeile darunter einfügen',
    deleteRow: 'Diese Zeile löschen',
    formatting: 'Formatierung',
    bold: 'Fett',
    italic: 'Kursiv',
    bulletedList: 'Aufzählung',
    numberedList: 'Nummerierte Liste',
    quotation: 'Zitat',
    askModel: 'Das Modell bitten, es anders zu sagen',
    customInstruction: 'Eigene Anweisung …',
    whatShouldChange: 'Was soll sich ändern?',
    proposedChange: 'Vorgeschlagene Änderung',
    proposedReplacement: 'Vorgeschlagene Ersetzung',
    proposedRewrite: 'Vorgeschlagene Umformulierung',
    unchanged: 'Das Modell hat die Passage unverändert zurückgegeben.',
    factsMoved: 'Ein zweiter Durchgang meint, diese Angaben hätten sich verschoben',
    noFactMoved: 'Ein zweiter Durchgang fand keine verschobene Angabe. Er übersieht Dinge.',
    useThis: 'Übernehmen',
    improveClarity: 'Verständlichkeit verbessern',
    improveClarityInstruction: 'Mach das leichter lesbar.',
    makeFormal: 'Förmlicher machen',
    makeFormalInstruction:
      'Formuliere es förmlicher, wie es in einem professionellen Protokoll stünde.',
    makePlainer: 'Einfacher machen',
    makePlainerInstruction:
      'Formuliere es einfacher und direkter, ohne an Genauigkeit zu verlieren.',
    shorten: 'Kürzen',
    shortenInstruction: 'Sag das mit weniger Worten.',
    rewriteUnavailable: 'Umformulieren ist hier nicht verfügbar.',
    replaceUnavailable: 'Einen Namen zu ersetzen ist hier nicht verfügbar.',
    nameNotFound: 'Dieser Name kommt in diesem Protokoll nicht vor.',
    protocolMarkdown: 'Protokoll-Markdown',
    protocolLabel: 'Protokoll',
    protocolDetails: 'Details zum Protokoll',
    documentDetails: 'Details zum Dokument',
    closeInspector: 'Bereich schließen',
    tabDocument: 'Dokument',
    tabTranscript: 'Transkript',
    tabHistory: 'Verlauf',
    status: 'Status',
    createRevision: 'Fassung anlegen',
    lineNumber: (line: number) => `Zeile ${line}`,
    pageNumber: (page: number) => `Seite ${page}`,
    revisionNumber: (ordinal: number) => `Fassung ${ordinal}`,
    markReviewed: 'Als geprüft markieren',
    style: 'Stil',
    sections: 'Abschnitte',
    newSection: 'Neuer Abschnitt',
    appearance: 'Erscheinungsbild',
    editAppearance: 'Erscheinungsbild bearbeiten',
    headerFooter: 'Kopf- und Fußzeile',
    editHeaderFooter: 'Kopf- und Fußzeile bearbeiten',
    nothingRepeated: 'Nichts wiederholt sich auf der Seite',
    presets: 'Voreinstellungen',
    useOrSavePreset: 'Voreinstellung verwenden oder speichern',
    noneSaved: 'Noch keine gespeichert',
    savedCount: (count: number) => `${count} gespeichert`,
    use: 'Verwenden',
    remove: 'Entfernen',
    nameThisPreset: 'Diese Voreinstellung benennen',
    nameForPreset: 'Name für diese Voreinstellung',
    save: 'Speichern',
    cancel: 'Abbrechen',
    saveAsPreset: 'Dieses Erscheinungsbild samt Kopfzeile als Voreinstellung speichern',
    export: 'Export',
    exportPdf: 'Als PDF exportieren',
    exportWord: 'Als Word exportieren',
    exportMarkdown: 'Als Markdown exportieren',
    exportPlainText: 'Als reinen Text exportieren',
    exportNote:
      'Das PDF wird aus dem Dokument gedruckt, das Sie lesen, gesetzt wie dieses Projekt seine Protokolle setzt — wählen Sie im Druckdialog „Als PDF sichern“.',
    source: 'Quelle',
    findSelectedPassage: 'Die markierte Passage suchen',
    lookingFor: 'Gesucht wird:',
    openReviewedTranscript: 'Geprüftes Transkript öffnen',
    whatToCheck: 'Was zu prüfen ist',
    revisions: 'Fassungen',
    current: 'Aktuell',
    restore: 'Wiederherstellen',
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
    sidebarWidth: (width: number) => `${width} Pixel`,
    resizeSidebar: 'Seitenleiste anpassen. Mit den Pfeiltasten ändern, mit Enter zurücksetzen.',
    themeAlwaysLightShort: 'Dauerhaft hell',
    themeAlwaysDarkShort: 'Dauerhaft dunkel',

    importNeedsDecision: 'Der Import braucht Ihre Entscheidung',
    needsAttention: 'Braucht Ihre Aufmerksamkeit',
    importingRecording: 'Aufnahme wird übernommen',
    transcribing: 'Wird transkribiert',
    writingProtocol: 'Protokoll wird geschrieben',
    working: 'Läuft',
    workingEllipsis: 'Läuft …',
    separatingSpeakers: 'Sprecher werden getrennt',
    openMeetingNeedingAttention: 'Die Besprechung öffnen, die Aufmerksamkeit braucht',
    openThisMeeting: 'Diese Besprechung öffnen',
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

    setupProviderTitle: 'Noch eines, bevor das erste Protokoll entsteht',
    setupProviderBody:
      'Transkribieren geht jetzt. Für das Protokoll wird zusätzlich ein Sprachmodell auf diesem Gerät gebraucht, eingerichtet wird es in den Einstellungen. Aufnehmen und Transkribieren können Sie schon vorher.',
    setupProviderAction: 'In den Einstellungen einrichten',
    setupTitle: 'Ein Download vor der ersten Transkription',
    setupBody: (quality: string, size: string) =>
      `LocaLog transkribiert auf diesem Gerät, also muss das Modell darauf sein. Die Qualität „${quality}“ ist ${size} groß und wird einmal geladen. Sie können vorher eine Aufnahme importieren — gebraucht wird es erst, wenn die Transkription beginnt.`,
    setupDownload: (size: string) => `Jetzt laden (${size})`,
    setupCancel: 'Abbrechen',
    setupAside: 'Weitere Qualitäten und die Sprechertrennung finden Sie in den Einstellungen.',
  },
};
