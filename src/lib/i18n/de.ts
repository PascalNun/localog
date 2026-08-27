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
  unsupportedSchema: (version: number) =>
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
};

export const de: Strings = {
  failures,

  settings: {
    interfaceLanguage: 'Sprache der Oberfläche',
    interfaceLanguageDetail:
      'In welcher Sprache LocaLog selbst geschrieben ist. Unabhängig von der Sprache der einzelnen Besprechung.',
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
