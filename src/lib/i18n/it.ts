/**
 * Every word the application says, in Italian.
 *
 * Typed against English, so this file cannot be missing a key or inventing one.
 *
 * ## Decisions taken once, here, so the whole application reads as one voice
 *
 * **Verbale, never "protocollo".** The decision the whole file rests on. In
 * Italian *protocollo* is the register a document is filed in — *numero di
 * protocollo* — and using it here would name the wrong object entirely. The
 * record a meeting produces is a **verbale**, which is exactly what an Italian
 * architecture or engineering studio circulates after a *riunione*.
 *
 * **Lei, not tu.** Written for offices keeping the formal record of meetings,
 * matching the German *Sie*, the French *vous* and the Spanish *usted*. Imperatives
 * take the third person: *scelga*, *verifichi*, *riprovi*.
 *
 * **Riunione for a meeting, registrazione for a recording, traccia for a track.**
 * The ordinary words, not calques.
 *
 * **Trascrizione** for the machine's reading of the audio. *Sbobinatura* is the
 * word a person would use for the same job done by hand, and would read as
 * folksy here.
 */

import type { Strings } from './en';

const failures = {
  missingProject: 'Il progetto selezionato non esiste più.',
  missingMeeting: 'La riunione selezionata non esiste più.',
  missingJob: 'L’operazione di importazione non è più disponibile.',
  importBusy: 'È già in corso l’importazione di un’altra registrazione. La concluda o la annulli.',
  unsupportedSchema: (version: string) =>
    `Questi dati di LocaLog sono stati creati da una versione più recente e non supportata (${version}).`,
  storageUnavailable: 'LocaLog non è riuscito ad accedere al proprio spazio di lavoro locale.',

  styleMissing: 'Questo stile non esiste più.',
  styleNameRequired: 'Dia un nome allo stile.',
  styleNotSaved: 'Non è stato possibile salvare lo stile.',
  styleUnavailable: 'Lo stile di verbale selezionato non è disponibile.',
  styleUsedByMeeting: 'Una riunione usa questo stile. La modifichi prima.',
  styleUsedByProject: 'Un progetto usa questo stile come predefinito. Lo modifichi prima.',

  presetNameRequired: 'Dia un nome alla preimpostazione.',
  presetNotSaved: 'Non è stato possibile salvare la preimpostazione.',
  presetBuiltInUndeletable: 'Una preimpostazione fornita con LocaLog non può essere eliminata.',

  transcriptInvalid: 'La trascrizione salvata non è valida.',
  transcriptSegmentMissing: 'Questo passaggio della trascrizione non esiste più.',
  transcriptTextRequired: 'Inserisca un testo di trascrizione valido.',
  transcriptNeedsSegment: 'Una trascrizione ha bisogno di almeno un passaggio.',
  transcriptSpeakerRequired: 'Inserisca un nome di interlocutore valido.',
  transcriptNotSaved: 'Non è stato possibile salvare la trascrizione.',
  transcriptNotCommitted: 'Non è stato possibile confermare la trascrizione.',
  spellingRequired: 'Inserisca una grafia valida.',

  protocolTextRequired: 'Inserisca un testo di verbale valido.',
  protocolRevisionMissing: 'La versione del verbale selezionata non esiste più.',
  protocolNeededBeforeExport: 'Generi un verbale prima di esportarlo.',
  protocolNeededBeforeSetAside: 'Generi un verbale prima di mettere da parte una sezione.',
  sectionNotSetAside: 'Non è stato possibile mettere da parte questa sezione.',
  reviewBeforeGeneration: 'Riveda la trascrizione prima della generazione.',
  vocabularyUnresolved: 'Non è stato possibile risolvere i nomi e i termini.',

  selectionRequired: 'Selezioni il testo da modificare.',
  selectionTooLong:
    'È troppo testo da modificare in una volta. Selezioni una sezione anziché il documento.',
  passageNotRewritten: 'Non è stato possibile riformulare questo passaggio.',
  openingNotRead: 'Non è stato possibile leggere l’inizio della riunione.',
  providerNeededForPassage:
    'Avvii la sua installazione di Ollama prima di far riformulare un passaggio.',
  providerNeededForOpening:
    'Avvii la sua installazione di Ollama prima di leggere le presentazioni.',
  providerNeededForCorrections:
    'Avvii la sua installazione di Ollama prima di verificare queste grafie.',
  providerModelRequired:
    'Scelga un modello Ollama installato in Impostazioni → Generazione del verbale.',

  styleNotMigrated: 'Non è stato possibile migrare uno stile.',
  termMissing: 'Questo termine non esiste più.',
  exportFormatInvalid: 'Scelga un formato di esportazione valido.',
  meetingDateInvalid: 'Scelga una data della riunione valida.',
  scopeInvalid: 'Scelga un ambito valido.',
  sourceFileInvalid: 'Scelga un file di origine valido.',
  workspaceViewInvalid: 'Scelga una vista dello spazio di lavoro valida.',
  recordingUnreadable: 'Non è stato possibile leggere questa registrazione.',
  appearanceNotSaved: 'Non è stato possibile salvare l’impaginazione.',
  furnitureNotSaved: 'Non è stato possibile salvare l’intestazione e il piè di pagina.',
  documentOperationFailed: 'L’operazione locale sul documento non è andata a buon fine.',
  providerConfigNotSaved:
    'Non è stato possibile salvare la configurazione del fornitore dei verbali.',
  runtimeConfigNotSaved:
    'Non è stato possibile salvare la configurazione dell’ambiente di trascrizione.',
  recorderNotStarted: 'Non è stato possibile avviare il registratore.',
  tracksNotCombined: 'Non è stato possibile unire le tracce della registrazione.',
  protocolInvalid: 'Il verbale salvato non è valido.',
  protocolNotUtf8: 'Il verbale salvato non è in UTF-8 valido.',
  editsNotRecorded: 'Queste modifiche non possono essere registrate.',

  recordingAlreadyRunning: 'È già in corso la registrazione di una riunione.',
  presetUnknown: 'Scelga una qualità di trascrizione conosciuta.',
  providerModelNotInstalled: 'Scelga un modello già installato in Ollama.',
  diariserPathInvalid: 'Scelga un programma di separazione degli interlocutori esistente.',
  whisperPathInvalid: 'Scelga un eseguibile whisper.cpp esistente.',
  nothingRecording: 'Non è in corso alcuna registrazione.',
  revealOnlyOnMac:
    'L’apertura della cartella è prevista solo su macOS. Il percorso qui sopra è corretto.',
  privacySettingsOnlyOnMac: 'L’apertura delle impostazioni sulla privacy è prevista solo su macOS.',
  providerNeededForModel: 'Avvii la sua installazione di Ollama prima di scegliere un modello.',
  settingsNotOpened: 'Non è stato possibile aprire Impostazioni di Sistema.',
  presetMissing: 'Questo modello di esportazione non è più disponibile.',
  downloadStopped: 'Lo scaricamento si è interrotto.',
  coordinatorUnavailable: 'Il coordinatore delle importazioni non è disponibile.',
  taskStopped: 'L’operazione locale di annullamento si è interrotta.',
  recorderPermissionsUnknown:
    'Non è stato possibile chiedere al registratore quali autorizzazioni abbia.',
  recorderStateUnknown: 'Il registratore è in uno stato sconosciuto. Riavvii LocaLog.',
  recordingNotFinished: 'Non è stato possibile concludere la registrazione.',
  replacementNotPrepared: 'Non è stato possibile preparare la sostituzione.',
  workspaceNotOpened: 'Non è stato possibile aprire la cartella dello spazio di lavoro.',
  settingsPaneUnknown: 'Questo pannello delle impostazioni non esiste.',
  meetingBusy: 'Questa riunione è ancora in lavorazione. Annulli prima quell’operazione.',
  printDialogUnavailable: 'Questa finestra non è riuscita ad aprire la finestra di stampa.',

  backupNameUnsafe: 'Questo nome di copia non può essere usato come nome di cartella.',
  notABackup: 'Questa cartella non è una copia di LocaLog: non ha un manifest.json.',
  backupPathOutside: (path: string) =>
    `Questa copia indica un file fuori dalla propria cartella (${path}), quindi non è stata ripristinata.`,
  backupFormatUnknown: (format: string) =>
    `Questa copia è stata scritta nel formato ${format}, che questa versione di LocaLog non sa leggere. Una versione più recente lo saprà.`,
  backupDamaged: (what: string) =>
    `Questa copia è incompleta o danneggiata (${what}), quindi non è stato modificato nulla. Il suo lavoro attuale è intatto.`,
  backupNameTaken: (name: string) => `In quella cartella c’è già qualcosa chiamato «${name}».`,
  backupIoFailed: (what: string) => `Non è stato possibile scrivere né leggere la copia: ${what}`,
  backupDatabaseFailed: (what: string) => `Non è stato possibile copiare il database: ${what}`,

  categoryRequired: 'Scelga una categoria.',
  meetingLanguageRequired: 'Scelga una lingua della riunione.',
  meetingLanguageInvalid: 'Scelga una lingua della riunione valida.',
  meetingInvalid: 'Scelga una riunione valida.',
  projectInvalid: 'Scelga un progetto valido.',
  styleInvalid: 'Scelga uno stile di verbale valido.',
  sourceRecordingInvalid: 'Scelga una registrazione di origine valida.',
  meetingTitleRequired: 'Inserisca un titolo per la riunione.',
  projectNameRequired: 'Inserisca un nome per il progetto.',
  termRequired: 'Inserisca un termine.',
  meetingTitleTooLong: 'Il titolo della riunione è troppo lungo.',
  speakerPassCannotRead: (what: string) =>
    `La passata sugli interlocutori non è riuscita a leggere l’audio di lavoro: ${what}`,
  speakerPassCannotWrite: (what: string) =>
    `La passata sugli interlocutori non è riuscita a scrivere il proprio audio: ${what}`,
  recordingNotStored: (what: string) =>
    `Non è stato possibile archiviare la registrazione: ${what}`,
  recordingNotRead: (what: string) => `Non è stato possibile leggere la registrazione: ${what}`,
  modelNotDownloaded: (what: string) => `Non è stato possibile scaricare il modello: ${what}`,
  modelNotSaved: (what: string) => `Non è stato possibile salvare il modello: ${what}`,
  ollamaRequestFailed: (what: string) =>
    `Ollama non è riuscito a completare la richiesta locale: ${what}`,
  recorderStartFailed: (what: string) => `Non è stato possibile avviare il registratore: ${what}`,

  embeddingsUnrecognisable:
    'La passata sugli interlocutori non ha prodotto impronte vocali riconoscibili.',
  embeddingsNoDimensions: 'Queste impronte vocali non descrivono alcuna dimensione.',
  embeddingsTruncated: 'Queste impronte vocali sono più corte di quanto dichiarino.',
  probeInvalid: 'L’analisi del file ha restituito metadati non validi.',
  cachePathInvalid: 'Il percorso della cache normalizzata non è valido.',
  normalizerNoOutput: 'La preparazione del file non ha prodotto alcun audio.',
  speakerPassNoAudio: 'La passata sugli interlocutori non ha nulla da ascoltare.',
  speakerPassTooMuchAudio:
    'La passata sugli interlocutori ha previsto più audio di quanto sia possibile trattare.',
  recordingEmpty: 'La registrazione è stata archiviata come file vuoto.',
  editsLeaveNothing: 'Questi tagli non lascerebbero alcuna registrazione.',
  workingAudioUnreadable: 'L’audio di lavoro non è un file WAV leggibile.',
  workingAudioNotWav: 'L’audio di lavoro non è un file WAV.',
  workingAudioSilent: 'L’audio di lavoro non contiene suono.',
  workingAudioFormatUnreadable: 'L’audio di lavoro ha un formato illeggibile.',
  workingAudioNoFormat: 'L’audio di lavoro non descrive alcun formato.',
  condensedAudioTooLarge: 'L’audio condensato è troppo grande da scrivere.',
  combinedPathInvalid: 'Il percorso della registrazione unita non è valido.',
  modelUnknown: 'Questo modello di trascrizione non è riconosciuto.',
  downloadCancelled: 'Lo scaricamento è stato annullato.',
  downloadCorrupt: 'Lo scaricamento era incompleto o danneggiato ed è stato scartato.',
  ollamaModelGone:
    'Il modello Ollama selezionato non è più installato. Ne scelga un altro e riprovi.',
  ollamaModelChanged:
    'Il modello Ollama selezionato è cambiato dopo che questa operazione era in coda. Riprovi per risolverlo di nuovo.',
  ollamaRuntimeChanged:
    'L’ambiente Ollama è cambiato dopo che questa operazione era in coda. Riprovi per risolverlo di nuovo.',
  responseTooLarge:
    'La risposta del modello locale ha superato il limite di sicurezza e non è stata salvata.',
  responseIncomplete: 'Il modello locale si è fermato prima di restituire un verbale completo.',
  protocolWouldNotFit: (sizes: string) => {
    const [expected, ceiling] = sizes.split('/').map((n) => Number(n).toLocaleString('it-IT'));
    return `Questa riunione è abbastanza lunga da far sì che un suo verbale — circa ${expected} caratteri — non stia in un’unica risposta, che ne contiene circa ${ceiling}. Non è stato tentato nulla: è una questione di aritmetica e non di una risposta venuta male, e riprovare fallirebbe allo stesso modo. Scelga uno stile più sintetico, o divida la registrazione.`;
  },
  generationConfigUnreadable:
    'Questa operazione è stata preparata da una versione precedente di LocaLog e non può essere letta. Non è stato salvato nulla e la sua trascrizione è invariata. Avvii di nuovo la generazione.',
  ollamaUnchecked: 'Ollama non è ancora stato verificato.',
  responseUnusable:
    'Il modello locale ha restituito una risposta che LocaLog non può usare come verbale. Non è stato salvato nulla e la sua trascrizione è invariata. Riprovare spesso funziona, perché un modello risponde ogni volta in modo diverso.',
  recorderMissing:
    'Non è installato alcun registratore. LocaLog ne fornisce uno; questa build non lo trova.',
  recorderSilentAboutPermissions: 'Il registratore non ha detto che cosa gli è consentito fare.',
  recorderCannotReportPermissions:
    'Questo registratore non sa dire che cosa gli è consentito fare.',
  runtimePathsMustBeAbsolute: 'Scelga percorsi assoluti per l’eseguibile e il modello whisper.cpp.',
  whisperExecutableMissing: 'L’eseguibile whisper.cpp selezionato non è stato trovato.',
  whisperModelMissing: 'Il modello whisper.cpp selezionato non è stato trovato.',
  embeddingsVersion: (version: string) =>
    `Queste impronte vocali sono della versione ${version}, che questa build non legge.`,
  recordingTooSmall: (what: string) =>
    `La registrazione archiviata è troppo piccola per la sua durata (${what}).`,
  workingAudioFormatWrong: (what: string) =>
    `La passata sugli interlocutori ha bisogno di audio 16 kHz mono a 16 bit, e questo è ${what}.`,
  notEnoughSpace: (what: string) => `Spazio insufficiente per questo modello (${what}).`,

  // Si veda la nota in en.ts: frasi che la parte Rust scriveva ancora da sé.
  settingInvalid: 'Questa impostazione di esecuzione non può essere salvata.',
  meetingTitleRequiredToRecord: 'Dia un titolo alla riunione. Non c’è alcun file da cui ricavarlo.',
  importSourceGone: 'Scelga di nuovo il file originale prima di ritentare questa importazione.',
  termProjectRequired: 'Scelga il progetto a cui appartiene questo termine.',
  termAlreadyPresent: 'Questo termine è già presente qui.',
  sourceRecordingRequired: 'Scelga di nuovo la registrazione di partenza.',
  managedPathInvalid: 'Il percorso di questo file salvato non è valido.',
  documentChecksumFailed: 'Un documento salvato non ha superato il controllo locale di integrità.',
  transcriptOutputInvalid:
    'La trascrizione ha prodotto qualcosa che LocaLog non può leggere come trascrizione.',
  speakerCountOutOfRange: 'Il numero previsto di interlocutori deve essere compreso tra 2 e 64.',
  sourceNotCommitted: 'Confermi la fonte della riunione prima di trascriverla.',
  providerNeededForGeneration: 'Avvii la sua installazione di Ollama prima di generare un verbale.',
  exportDestinationInvalid: 'Scelga una destinazione di esportazione valida.',
  exportFileExists:
    'Scelga un nuovo nome di file. Un file esistente non viene mai sovrascritto senza chiedere.',
  exportFolderMissing: 'La cartella di esportazione scelta non è disponibile.',
  processingBusy:
    'È già in corso un’altra attività locale. Attenda che finisca, oppure la annulli prima.',
  ffmpegMissingForRecording: 'FFmpeg serve per completare una registrazione e non è stato trovato.',

  // La riga di Ollama nelle impostazioni. Si veda la nota in en.ts.
  ollamaNotRunning: (detail: string) =>
    `Avvii la sua installazione di Ollama, poi aggiorni.${detail ? ` ${detail}` : ''}`,
  ollamaModelsUnreadable: (detail: string) =>
    `Ollama è in funzione ma non ha indicato quali modelli sono installati.${detail ? ` ${detail}` : ''}`,
  ollamaReadyNoModel: 'Ollama è pronto. Scelga un modello installato per generare verbali.',
  ollamaModelReady: 'Il modello locale scelto è pronto.',
  ollamaSelectedModelMissing:
    'Il modello scelto non è installato. Ne scelga un altro già installato.',
};

export const it: Strings = {
  locale: 'it-IT',

  failures,

  /** Si veda la nota in en.ts: la chiave è il valore salvato. */
  meetingLanguages: {
    English: 'Inglese',
    German: 'Tedesco',
    French: 'Francese',
    Spanish: 'Spagnolo',
    Italian: 'Italiano',
    Dutch: 'Olandese',
    Portuguese: 'Portoghese',
    Polish: 'Polacco',
    Danish: 'Danese',
    Swedish: 'Svedese',
    Norwegian: 'Norvegese',
    Finnish: 'Finlandese',
    Czech: 'Ceco',
    Turkish: 'Turco',
    Japanese: 'Giapponese',
    Korean: 'Coreano',
    Chinese: 'Cinese',
    Arabic: 'Arabo',
    Ukrainian: 'Ucraino',
  },
  dialog: {
    detectFromRecording: 'Rilevare dalla registrazione',
    chooseRecording: 'Scegliere la registrazione di una riunione',
    audioAndVideo: 'Audio e video',
    plainText: 'Testo semplice',
    exportTitle: (title: string) => `Esportare ${title}`,
  },

  settings: {
    memoryReported: (gb: number) => `${gb} GB di memoria rilevati`,
    themeAutomatic: 'Automatico',
    themeLight: 'Chiaro',
    themeDark: 'Scuro',
    modelSelected: 'Selezionato',
    useThisModel: 'Usa questo modello',
    useModel: 'Usa il modello',
    catalogueNote:
      'Il catalogo è volutamente ristretto. LocaLog non scarica modelli di nascosto e non presenta un mercato di modelli. Una voce diventa selezionabile solo dopo che se ne sono verificati ambiente, licenza, consumo di memoria e qualità in tedesco e in inglese.',
    managedCopiesNote:
      'LocaLog conserva copie proprie delle registrazioni importate, dell’audio preparato, delle trascrizioni, dei verbali e dei modelli scaricati nella sua cartella dati. Le esportazioni vengono scritte solo dove decide lei.',
    discoveredRuntime: (path: string) => `Ambiente rilevato: ${path}`,
    runtimeVersion: (version: string) => `Versione dell’ambiente: ${version}`,
    evaluatedIn: (languages: string) => `Valutato in ${languages}`,
    evaluationPending: 'Valutazione della qualità ancora in sospeso',
    otherModelNote:
      'Questo è per chi sa già quale modello locale vuole provare. LocaLog non lo valuta né lo consiglia, e resta soggetto agli stessi limiti di ambiente e memoria.',
    qualityLead:
      'Scelga la qualità che desidera. LocaLog scarica ciò che serve la prima volta e lo conserva su questo dispositivo.',
    speakerDiscovery:
      'LocaLog individua da sé l’ambiente per la separazione degli interlocutori, tra le proprie risorse o nel sistema. È facoltativo e non blocca mai una trascrizione.',
    noSpeakerRuntime:
      'Su questa macchina non è ancora stato trovato un ambiente compatibile per la separazione degli interlocutori.',
    readinessNote:
      'La verifica comprende un avvio di prova limitato, così che un eseguibile incompatibile o difettoso non venga presentato come disponibile.',
    restoreSummary: (name: string, projects: number, meetings: number, version: string) =>
      `${name} contiene ${projects} progetti e ${meetings} riunioni, copiati da LocaLog ${version}.`,
    restoreWarning:
      'Il ripristino sostituisce i progetti e le riunioni di questo spazio di lavoro con quelli. Non viene eliminato nulla — ciò che è qui viene conservato in una cartella accanto — ma LocaLog mostrerà il lavoro ripristinato, e dovrà chiuderlo e riaprirlo.',
    interfaceLanguage: 'Lingua dell’interfaccia',
    interfaceLanguageDetail:
      'La lingua di LocaLog stesso. Indipendente dalla lingua di ogni riunione.',
    application: 'Applicazione',
    title: 'Impostazioni',
    lead: 'Prima le scelte professionali. I dettagli tecnici restano richiusi.',
    sectionsLabel: 'Sezioni delle impostazioni',
    sectionGeneral: 'Generale',
    sectionModels: 'Modelli',
    sectionTranscription: 'Trascrizione',
    sectionStorage: 'Archiviazione',
    sectionAppearance: 'Impaginazione',
    sectionAdvanced: 'Avanzate',
    defaultExport: 'Esportazione predefinita',
    defaultExportDetail: 'Quale formato l’editor propone per primo. Gli altri restano a un clic.',
    defaultExportLabel: 'Formato di esportazione predefinito',
    formatPdf: 'PDF',
    formatWord: 'Word',
    formatMarkdown: 'Markdown',
    formatPlainText: 'Testo semplice',
    defaultForProtocols: 'Predefinito per i verbali',
    chooseOnce: 'Scelga una volta, poi continui a lavorare',
    modelLead:
      'LocaLog usa questo modello per le bozze locali dei verbali finché non lo cambia. Il percorso normale non le chiede di scegliere un modello per ogni riunione.',
    recommendedForMachine: 'Consigliato per questa macchina',
    notInstalledYet: 'Non ancora installato',
    baseline: 'Riferimento',
    european: 'Europeo',
    checkInstalled: 'Verifica i modelli installati',
    curatedModels: 'Modelli per i verbali selezionati',
    downloadModel: (size: string) => `Scaricare (${size})`,
    prepareSpeakerSeparation: 'Preparare la distinzione degli interlocutori',
    restoredBackup: (projects: number, meetings: number, previous: string) =>
      `Ripristinati ${projects} progetti e ${meetings} riunioni. Ciò che era qui è stato spostato in ${previous} anziché eliminato. Chiuda LocaLog e lo riapra per lavorare con lo spazio ripristinato.`,
    /** Si veda la nota in en.ts. */
    transcriptionPreset: {
      fast: { name: 'Rapida', detail: 'Bozze rapide, la più leggera in memoria' },
      balanced: { name: 'Equilibrata', detail: 'Per le riunioni di tutti i giorni' },
      accurate: { name: 'Precisa', detail: 'Qualità migliore, la più lenta' },
    },
    downloadingPreset: (name: string) => `Scaricamento di ${name}`,
    /** Si veda la nota in en.ts. */
    modelDescription: {
      'gemma4-12b':
        'Il più accurato e il più costante fra i modelli misurati: su tre esecuzioni ha conservato da 27 a 31 delle 35 cifre di una riunione, dove il successivo è sceso a 6. Più lento: circa quattordici minuti per una riunione di ottanta minuti.',
      'ministral-8b':
        'Misurato su una riunione in tedesco con tre impostazioni, ha scritto un verbale utilizzabile con una di esse: le altre hanno prodotto un abbozzo di due righe e un documento JSON dove era richiesto il markdown. Resta il candidato europeo, non ancora un’alternativa al riferimento.',
      'qwen3.5-4b':
        'Il modello misurato più veloce, circa cinque minuti per una riunione di ottanta minuti, e la scelta quando la memoria è poca. Non ha mai prodotto la tabella dei prossimi passi che lo stile formale richiede.',
      'ministral-3b': 'Il primo candidato europeo per il Mac meno potente fra quelli supportati.',
      'granite4.1-8b':
        'Misurato su una riunione in tedesco con tre impostazioni, ha conservato 22, 19 e 6 delle 35 cifre dette, a parità di ingresso. Un’esecuzione che perde cinque sesti di quanto è stato detto non è uno strumento per tenere un verbale, perciò non è consigliato.',
      'llama-8b': 'Un posto di confronto riservato a una versione di Llama verificata.',
    },
    modelOrigin: {
      international: 'Modello aperto internazionale',
      european: 'Modello europeo',
    },
    modelLicence: {
      apache2: 'Apache 2.0',
      gemma: 'Condizioni d’uso di Gemma',
      modelSpecific: 'Specifica del modello',
    },
    modelLanguage: {
      de: 'tedesco',
      en: 'inglese',
      ja: 'giapponese',
      more: 'e molte altre',
    },
    modelStatus: {
      installed: 'Installato',
      notInstalled: 'Non installato',
      plannedCandidate: 'Candidato previsto',
    },
    modelSizeInstalled: (gb: string) => `circa ${gb} GB installato`,
    modelSizeSmall: 'modello piccolo per la macchina',
    modelSizeLarger: 'modello locale più grande',
    useAnotherModel: 'Usa un altro modello installato',
    installedModel: 'Modello installato',
    chooseInstalledModel: 'Scegli un modello installato',
    useInstalledModel: 'Usa il modello installato',
    conservativeBaseline: 'In uso il riferimento prudente di 8 GB',
    transcriptionQuality: 'Qualità della trascrizione',
    cancel: 'Annulla',
    ready: 'Pronto',
    remove: 'Rimuovi',
    advancedDetails: 'Dettagli avanzati',
    modelsStoredNote:
      'I modelli sono conservati nella cartella dati di LocaLog e verificati prima dell’uso.',
    whisperExecutable: 'Eseguibile whisper-cli',
    whisperExecutablePlaceholder: '/percorso/di/whisper-cli',
    chooseFile: 'Scegli un file',
    whisperNote: 'Scelga il binario di trascrizione da riga di comando, non whisper-server.',
    saveRuntime: 'Salva l’ambiente',
    detected: (version: string) => `Rilevato: ${version}`,
    chooseWhisper: 'Scegli l’eseguibile whisper-cli',
    speakerDifferentiation: 'Distinzione degli interlocutori',
    speakerLead:
      'La separazione dei turni di parola indica chi ha parlato e quando. È facoltativa, non blocca mai una trascrizione, e i nomi restano modificabili durante la revisione.',
    runtimeUnavailable: 'Ambiente non disponibile in questa installazione',
    optional: 'Facoltativo',
    checkReadiness: 'Verifica la disponibilità',
    downloadingSpeakerModels: 'Scaricamento dei modelli per la separazione degli interlocutori',
    speakerRuntimeMissing:
      'I modelli sono pronti, ma questa installazione non ha un ambiente compatibile.',
    whereWorkIsKept: 'Dove è conservato il suo lavoro',
    workspaceNote:
      'LocaLog gestisce questa cartella perché i percorsi al suo interno restino validi, ma è sua e può guardarci dentro quando vuole.',
    showInFinder: 'Mostra nel Finder',
    backup: 'Copia di sicurezza',
    backupLead:
      'Tutto resta su questo dispositivo, il che vuol dire anche che se ne va insieme a lui. Una copia di sicurezza è una cartella comune, da mettere su un disco o dove tiene ciò che conta.',
    backUpNow: 'Fai una copia adesso',
    working: 'In corso…',
    backupContents:
      'Contiene ogni progetto, riunione, trascrizione e verbale, e le registrazioni stesse. Due cose sono escluse di proposito, perché non sono il suo lavoro ed entrambe si ricostruiscono all’occorrenza: i modelli scaricati e la copia preparata di ogni registrazione. Misurato su uno spazio di lavoro reale, quell’audio preparato era da solo tre quarti della copia.',
    restore: 'Ripristina',
    restoreLead:
      'Rimette a posto una copia di sicurezza. Prima viene verificata per intero, e ciò che c’è ora viene spostato di lato anziché eliminato.',
    chooseBackup: 'Scegli una copia…',
    chooseBackupTitle: 'Scegli una copia di sicurezza di LocaLog',
    whereToKeepBackup: 'Dove conservare la copia',
    replaceWorkspace: 'Sostituisci questo spazio di lavoro',
    restoring: 'Ripristino…',
    archived: 'Archiviato',
    archivedLead:
      'Progetti e riunioni messi da parte. Non è stato eliminato nulla: ogni riunione, trascrizione e verbale che contengono è ancora qui, e ancora in ogni copia di sicurezza.',
    show: 'Mostra',
    hide: 'Nascondi',
    nothingArchived: 'Non è stato archiviato nulla.',
    project: 'Progetto',
    meeting: 'Riunione',
    bringBack: 'Riporta indietro',
    theme: 'Tema',
    themeFollowing: (theme: string) => `Segue questo Mac, impostato su ${theme}.`,
    themeSetHere: 'Impostato qui, qualunque sia l’impostazione di questo Mac.',
    nextFakeJob: 'Prossima operazione simulata',
    nextFakeJobDetail:
      'Comando riservato allo sviluppo, per esaminare gli stati di errore e di ripetizione.',
    completeNormally: 'Si conclude normalmente',
    failOnce: 'Fallisce una volta, poi consente di riprovare',
    syntheticNote: 'Questo riguarda soltanto l’ambiente sintetico in memoria.',
  },

  project: {
    deleteMeeting: (title: string) => `Eliminare ${title}`,
    deleteWarning:
      'Eliminare una riunione rimuove da questo dispositivo la sua registrazione, la sua trascrizione e ogni versione del suo verbale. Non si può annullare.',
    eyebrow: 'Progetto',
    archiveProject: 'Archivia il progetto',
    newMeeting: 'Nuova riunione',
    meetings: 'Riunioni',
    newestFirst: 'Prima le più recenti',
    columnDate: 'Data',
    columnMeeting: 'Riunione',
    columnDuration: 'Durata',
    columnStatus: 'Stato',
    archive: 'Archivia',
    delete: 'Elimina',
    keep: 'Conserva',
    noMeetings: 'Ancora nessuna riunione',
    noMeetingsDetail:
      'Importi la prima registrazione per cominciare il registro delle riunioni di questo progetto.',
    importRecording: 'Importa una registrazione',
  },

  lifecycle: {
    draft: 'Bozza',
    sourceReady: 'Pronto da trascrivere',
    transcriptReady: 'Trascrizione pronta',
    protocolDraft: 'Bozza di verbale',
    reviewed: 'Rivisto',
    archived: 'Archiviato',
  },

  sections: {
    noHeadings: 'Questo verbale non ha ancora titoli, quindi non c’è nulla da elencare.',
    setAside: 'Metti da parte',
    addSection: 'Aggiungi una sezione',
    dragHint: 'Trascini, oppure usi i tasti freccia',
    setThisAside: 'Metti da parte questa sezione',
    putThisBack: 'Rimetti questa sezione',
    moveSection: (title: string) => `Sposta ${title}. Usi i tasti freccia.`,
    setAsideNamed: (title: string) => `Metti da parte ${title}`,
    putBackNamed: (title: string) => `Rimetti ${title}`,
    setAsideNote:
      'Una sezione messa da parte esce dal documento, quindi ciò che legge è esattamente ciò che verrà esportato. Resta conservata qui e può essere rimessa.',
  },

  jobErrors: {
    interrupted: {
      title: 'L’importazione è stata interrotta',
      detail:
        'LocaLog si è fermato prima che la copia gestita fosse confermata. L’originale esterno è invariato e può riprovare senza rischi.',
    },
    permission_denied: {
      title: 'LocaLog non è riuscito a leggere né ad archiviare la registrazione',
      detail:
        'Verifichi l’accesso al file scelto e alla cartella dati locale di LocaLog, poi riprovi. L’originale esterno non è stato modificato.',
    },
    insufficient_space: {
      title: 'Lo spazio di archiviazione locale non basta',
      detail:
        'Liberi dello spazio e riprovi. Nessuna registrazione parziale è stata presentata come completa.',
    },
    source_missing: {
      title: 'La registrazione scelta non è più disponibile',
      detail:
        'Rimetta il file al suo posto, oppure crei una nuova importazione. La riunione resta al sicuro in bozza.',
    },
    source_reselection_required: {
      title: 'Scelga di nuovo la registrazione',
      detail:
        'Questa riunione è stata creata da una versione di sviluppo precedente che non conservava la posizione dell’origine. Scelga di nuovo la registrazione per continuare; la riunione è stata conservata.',
    },
    unsupported_media: {
      title: 'Questo tipo di file non è ancora supportato',
      detail: 'Scelga un file audio o video comune. L’originale esterno non è stato modificato.',
    },
    empty_source: {
      title: 'La registrazione scelta è vuota',
      detail:
        'Scelga una registrazione che contenga dati audio o video. Il file esterno vuoto non è stato modificato.',
    },
    synthetic_failure: {
      title: 'L’adattatore di sviluppo si è fermato come richiesto',
      detail:
        'L’errore indotto si è verificato prima che una versione fosse confermata. La sua origine e il suo ultimo stato stabile sono al sicuro, e può riprovare.',
    },
    invalid_adapter_output: {
      title: 'Non è stato possibile validare l’esito locale',
      detail:
        'LocaLog non ha salvato quel risultato incompleto. La sua ultima origine stabile e le versioni dei documenti sono al sicuro.',
    },
    runtime_missing: {
      title: 'Scelga un ambiente di trascrizione locale',
      detail:
        'Selezioni un eseguibile whisper.cpp installato in Impostazioni → Trascrizione. LocaLog non scarica ambienti.',
    },
    model_missing: {
      title: 'Scelga un modello di trascrizione locale',
      detail:
        'Selezioni un modello whisper.cpp già disponibile in Impostazioni → Trascrizione. Nessun modello è stato scaricato o modificato.',
    },
    runtime_changed: {
      title: 'L’ambiente di trascrizione è cambiato',
      detail:
        'L’operazione in coda non è stata eseguita perché il suo eseguibile whisper.cpp non corrisponde più all’ambiente registrato. Riprovi per risolvere l’ambiente attuale.',
    },
    model_changed: {
      title: 'Il modello di trascrizione è cambiato',
      detail:
        'L’operazione in coda non è stata eseguita perché il suo modello non corrisponde più all’impronta registrata. Riprovi per risolvere il modello attuale.',
    },
    media_probe_failed: {
      title: 'Non è stato possibile esaminare la registrazione',
      detail:
        'Verifichi che FFprobe sia installato e che l’origine importata sia ancora leggibile. L’originale resta invariato.',
    },
    normalization_failed: {
      title: 'Non è stato possibile preparare la registrazione',
      detail:
        'Verifichi che FFmpeg sia installato e riprovi. La copia preparata si può rigenerare e l’originale resta invariato.',
    },
    transcription_failed: {
      title: 'La trascrizione locale non è andata a buon fine',
      detail:
        'L’ambiente whisper.cpp si è fermato prima che una versione della trascrizione fosse confermata. Verifichi il suo modello e riprovi.',
    },
    transcription_timeout: {
      title: 'La trascrizione locale ha richiesto troppo tempo',
      detail:
        'Il processo di trascrizione sorvegliato è stato fermato prima che una versione fosse confermata. Verifichi la registrazione e l’ambiente, poi riprovi.',
    },
    provider_model_missing: {
      title: 'Il modello locale selezionato non è disponibile',
      detail:
        'Il modello Ollama selezionato non è più installato. Ne scelga uno installato in Impostazioni → Generazione del verbale, poi riprovi.',
    },
    provider_model_changed: {
      title: 'Il modello locale è cambiato',
      detail:
        'L’impronta del modello è cambiata dopo che questa operazione era in coda. Riprovi per prendere il modello installato ora.',
    },
    provider_runtime_changed: {
      title: 'Il fornitore locale è cambiato',
      detail:
        'La versione dell’ambiente Ollama è cambiata dopo che questa operazione era in coda. Riprovi per prendere l’ambiente attuale.',
    },
    provider_unavailable: {
      title: 'La generazione locale del verbale non è riuscita a connettersi',
      detail:
        'Avvii la sua installazione di Ollama e riprovi. LocaLog non avvia né scarica ambienti.',
    },
    provider_invalid_output: {
      title: 'Non è stato possibile validare l’esito del modello locale',
      detail:
        'LocaLog non ha salvato quel verbale incompleto o malformato. La sua trascrizione è al sicuro e può riprovare.',
    },
    provider_incomplete_output: {
      title: 'Non è stato possibile validare l’esito del modello locale',
      detail:
        'LocaLog non ha salvato quel verbale incompleto o malformato. La sua trascrizione è al sicuro e può riprovare.',
    },
    provider_response_too_large: {
      title: 'La risposta del modello locale era troppo grande',
      detail:
        'La risposta ha superato il limite di sicurezza di LocaLog e non è stata salvata. Riprovi con una trascrizione più corta o con un altro modello locale.',
    },
    invalid_transcript_output: {
      title: 'Non è stato possibile validare l’esito della trascrizione',
      detail:
        'LocaLog non ha salvato l’esito dell’ambiente perché era incompleto o malformato. La sua origine è al sicuro.',
    },
    processing_failed: {
      title: 'L’elaborazione locale non è andata a buon fine',
      detail:
        'Nessuna trascrizione né alcun verbale incompleto è stato presentato come pronto. Il suo ultimo stato stabile resta disponibile e può riprovare.',
    },
    unknown: {
      title: 'L’importazione non è andata a buon fine',
      detail:
        'La riunione resta in bozza e l’originale esterno non è stato modificato. Può riprovare senza rischi.',
    },
  },

  jobStages: {
    transcriptSaved: 'Trascrizione salvata',
    protocolSaved: 'Verbale salvato',
    importComplete: 'Importazione completata — originale invariato',
    processingCancelled: 'Elaborazione locale annullata — stato stabile conservato',
    processingInterrupted: 'Elaborazione locale interrotta — stato stabile conservato',
    processingFailed: 'L’elaborazione locale non è riuscita — stato stabile conservato',

    ready_to_import: 'Pronto ad acquisire la registrazione',
    copying: 'Acquisizione della registrazione',
    stoppingSafely: 'Arresto in sicurezza',
    temporary_complete: 'Quasi fatto',
    finalizing: 'Messa al sicuro della registrazione',
    duplicate_confirmation: 'Questa registrazione potrebbe essere già qui',
    completed: 'La registrazione è arrivata',
    cancelled: 'Importazione annullata — originale invariato',
    interrupted: 'Importazione interrotta — originale invariato',
    failed: 'L’importazione non è riuscita — originale invariato',
    probing_media: 'Esame della registrazione',
    normalizing_audio: 'Preparazione dell’audio',
    output_staged: 'Salvataggio in sicurezza',

    transcription_queued: 'Pronto da trascrivere',
    checking_source: 'Verifica della registrazione',
    loading_transcription_model: 'Caricamento del modello',
    transcribing_audio: 'Trascrizione in corso',
    separating_speakers: 'Distinzione degli interlocutori',
    validating_transcript: 'Salvataggio della trascrizione',
    preparing_fake_transcriber: 'Preparazione',
    transcribing_synthetic_segments: 'Creazione dei passaggi della trascrizione',

    generation_queued: 'Pronto a redigere il verbale',
    checking_transcript: 'Verifica della trascrizione',
    resolving_protocol_inputs: 'Raccolta dello stile e dei termini',
    condensing_transcript: 'Lettura dell’intera riunione',
    generating_protocol: 'Stesura della bozza del verbale',
    validating_protocol: 'Salvataggio del verbale',
    reading_introductions: 'Lettura di chi si è presentato',

    protocol_would_not_fit:
      'Questa riunione è più lunga di quanto una sola passata possa contenere',
    segments_no_subject_claimed: 'Una parte della riunione non è rientrata in alcun argomento',
    sections_over_their_length: 'Alcune sezioni sono venute più lunghe del richiesto',

    finding_subjects: (detail: string) =>
      detail
        ? `Ricerca degli argomenti trattati — passaggio ${detail}`
        : 'Ricerca degli argomenti trattati',
    writing_section: (detail: string) =>
      detail ? `Stesura di ${detail}` : 'Stesura del verbale sezione per sezione',
    joining_subjects: (detail: string) =>
      detail
        ? `Unione degli argomenti affini — ${detail} trovati`
        : 'Unione degli argomenti affini',
    joined_subjects: (detail: string) =>
      detail ? `Argomenti uniti — ${detail}` : 'Argomenti uniti',
    joining_failed: (detail: string) =>
      detail
        ? `Non è stato possibile unire gli argomenti — ${detail}`
        : 'Non è stato possibile unire gli argomenti',

    working: 'In corso',
  },

  stages: {
    label: 'Fasi della riunione',
    source: 'Origine',
    transcript: 'Trascrizione',
    protocol: 'Verbale',
  },

  progress: {
    needsAttention: 'Richiede la sua attenzione',
    backgroundWork: 'Lavoro in secondo piano',
    cancellingSafely: 'Annullamento in sicurezza…',
    cancel: 'Annulla',
    speakerPassNote:
      'Questa passata legge l’intera registrazione per confrontare i turni di parola. Una registrazione lunga può richiedere qualche minuto; può annullare in sicurezza in qualsiasi momento.',
    latestRetained: 'Ultimo stato stabile conservato',
    originalUnchanged: ' · originale esterno invariato',
    retry: 'Riprova',
    importing: 'Importazione della registrazione',
    transcribing: 'Trascrizione in corso',
    generating: 'Generazione del verbale',
    separatingSpeakers: 'Separazione degli interlocutori',
    working: 'In corso…',
    duplicateNote:
      'Lo stesso contenuto è già presente in LocaLog. Non è stato unito né scartato nulla.',
    cancelImport: 'Annulla l’importazione',
    importAnotherCopy: 'Importa un’altra copia',
    chooseSourceAgain: 'Scegli di nuovo l’origine',
    continueImport: 'Prosegui l’importazione',
    transcribeAgain: 'Riavvia la trascrizione',
    generateAgain: 'Riavvia la generazione',
  },

  newProject: {
    namesHeading: 'Nomi e termini',
    namesLead:
      'Una trascrizione non può indovinare un nome che non ha mai sentito. Darglieli ora è il minuto più utile che possa dedicare a questo progetto: un nome capito male si ripete in ogni verbale tratto da quella registrazione, e nessun passaggio successivo lo recupera.',
    namesPeople: 'Persone',
    namesPeopleHint: 'Chi sarà probabilmente presente, o verrà nominato in riunione.',
    namesPeoplePlaceholder: 'Anna Waldt, Solvane, Rovelli',
    namesOrganisations: 'Imprese e committenti',
    namesOrganisationsHint: 'La committenza, gli altri professionisti, i fornitori.',
    namesOrganisationsPlaceholder: 'AVENTOR, Falkenstein-Weide',
    namesProject: 'Questo progetto',
    namesProjectHint: 'Come si chiamano il progetto, il lotto o l’edificio.',
    namesProjectPlaceholder: 'Riverside Pavilion, Halle 4',
    namesTerms: 'Tutto il resto che vale la pena scrivere bene',
    namesTermsHint:
      'Le parole proprie di questo lavoro che una trascrizione generica non conoscerebbe.',
    namesTermsPlaceholder: 'Tragwerk, Clusterwohnung',
    namesNote:
      'Li separi con virgole. È tutto facoltativo e nulla è definitivo: può aggiungere e correggere termini quando vuole in Nomi e termini, e una correzione fatta durante la revisione di una trascrizione viene conservata anche qui.',
    creating: 'Creazione…',
    createAndContinue: 'Crea e prosegui',
    afterCreated:
      'Lo stile del verbale, e i nomi e termini di questo lavoro, si possono impostare dopo aver creato il progetto. I nomi valgono bene un minuto: sono ciò che una trascrizione non può indovinare.',
    eyebrow: 'Progetti',
    title: 'Nuovo progetto',
    lead: 'Crei il contesto professionale a cui riunioni e origini appartengono.',
    defaults: 'Valori predefiniti del progetto',
    name: 'Nome del progetto',
    namePlaceholder: 'p. es. Studio centro civico',
    description: 'Descrizione',
    descriptionOptional: 'facoltativo',
    descriptionPlaceholder: 'Una breve descrizione interna',
    defaultLanguage: 'Lingua predefinita delle riunioni',
    defaultLanguageDetail: 'Indipendente dalla lingua dell’interfaccia.',
    cancel: 'Annulla',
  },

  appearance: {
    font: 'Carattere',
    appliesToProject: (project: string) =>
      `Si applica a tutti i verbali di ${project}, perché i documenti di uno studio si assomiglino. Cambia come il verbale è impaginato, mai ciò che dice: quello è lo stile qui sopra.`,
    bodySize: 'Corpo del testo',
    headingScale: 'Scala dei titoli',
    lineSpacing: 'Interlinea',
    pageWidth: 'Larghezza di pagina',
  },

  record: {
    recordingNow: 'Registrazione',
    recordThisMeeting: 'Registra questa riunione',
    lead: 'La stanza e la chiamata vengono captate su tracce separate, su questo dispositivo. Se i presenti abbiano dato il consenso spetta a lei stabilirlo; LocaLog non può saperlo.',
    notRecording: 'Nessuna registrazione',
    microphone: 'Microfono',
    theCall: 'La chiamata',
    trackRecording: 'In registrazione',
    trackSilent: 'Finora in silenzio',
    trackListening: 'In ascolto…',
    stopRecording: 'Ferma la registrazione',
    finishing: 'Conclusione…',
    startRecording: 'Avvia la registrazione',
    starting: 'Avvio…',
    backToMeeting: 'Torna alla riunione',
    noRecorder: 'Questa build non ha un registratore. Importi invece un file.',
    openTheSetting: 'Apri l’impostazione',
    grantedInSettings: 'Concesso in Impostazioni di Sistema, e recepito qui non appena torna.',
    callWouldNotRecordTitle: 'La chiamata non verrebbe registrata.',
    callWouldNotRecordBody:
      'macOS non ha concesso a LocaLog la registrazione dello schermo e dell’audio di sistema, e senza di essa una registrazione della chiamata è silenzio anziché un errore: conviene concederla ora invece di scoprirlo dopo. Il microfono della stanza verrebbe comunque captato.',
    roomWouldNotRecordTitle: 'La stanza non verrebbe registrata.',
    roomWouldNotRecordBody:
      'A LocaLog è stato negato il microfono. La chiamata verrebbe comunque captata se l’impostazione qui sopra lo consente.',
    recorderNotesTitle: 'Il registratore non è riuscito a fare tutto ciò che gli è stato chiesto.',
    stoppedOnItsOwn:
      'Il registratore si è fermato da solo. Ciò che aveva captato fino a quel punto è stato conservato.',
    quietCall: (seconds: number) =>
      `Dalla chiamata non arriva nulla da ${seconds} secondi. macOS dà silenzio anziché un errore a un’applicazione priva del permesso di registrare schermo e audio di sistema: conviene verificarlo ora e non dopo la riunione.`,
    quietMicrophone: (seconds: number) =>
      `Dal microfono non arriva nulla da ${seconds} secondi. Verifichi che sia selezionato l’ingresso giusto e che nessun altro lo stia occupando.`,
  },

  meeting: {
    browserPreview: 'Anteprima nel browser',
    speakersEstimateNote:
      'LocaLog raggruppa le voci che sente e le conta. È una stima, che può sostituire con un numero se le sembra sbagliata.',
    speakersCountNote:
      'Basta la sua stima migliore: è il numero di voci che LocaLog cercherà. Troppe possono dividere una persona in due, troppo poche possono confonderne due.',
    speakersTogetherNote: 'La trascrizione mantiene un solo nome di interlocutore.',
    importInterrupted:
      'LocaLog è stato chiuso prima che la copia gestita fosse confermata. La riunione resta in bozza e l’importazione può essere ripetuta senza rischi.',
    importCancelled:
      'La copia gestita è stata annullata. La riunione resta in bozza e il file esterno non è stato modificato.',
    importFailed:
      'Non è stato possibile confermare la copia gestita. La riunione resta in bozza e il file esterno non è stato modificato.',
    importRunning:
      'LocaLog sta copiando questa origine nel proprio archivio. Sarà pronta solo quando la copia sarà stata verificata e confermata.',
    sourceStored:
      'è conservato al sicuro con questa riunione. L’originale esterno non è stato modificato.',
    sourceSynthetic:
      'è assegnato a questa riunione dimostrativa. Nessun file reale è stato copiato.',
    syntheticFixture: 'Materiale dimostrativo',
    eyebrow: 'Riunione',
    titleLabel: 'Titolo della riunione',
    editTitle: 'Modifica il titolo della riunione',
    languageLabel: 'Lingua della riunione',
    changeLanguage: 'Cambia la lingua della riunione',
    save: 'Salva',
    saveLanguage: 'Salva la lingua',
    cancel: 'Annulla',
    recordingEyebrow: 'Registrazione',
    nothingRecorded: 'Ancora nulla di registrato',
    recordLead:
      'La stanza e la chiamata verranno captate su tracce separate, su questo dispositivo. Può fermarsi appena la riunione finisce.',
    recordThisMeeting: 'Registra questa riunione',
    sourceImport: 'Importazione dell’origine',
    originalUnchanged: 'Il suo originale resta invariato',
    sourceReady: 'Origine pronta',
    readyToTranscribe: 'Pronto da trascrivere',
    managedSource: 'Origine gestita',
    language: 'Lingua',
    languageHint: 'Impostazione della riunione · la cambi qui sopra prima di trascrivere',
    preset: 'Preimpostazione',
    globalDefault: 'Valore predefinito',
    notSelected: 'Non selezionato',
    peopleSpeaking: 'Persone che parlano',
    doNotSeparate: 'Non distinguere gli interlocutori',
    separateAndCount: 'Distinguerli, e capire quanti sono',
    prepareSpeakers: 'Prepara la separazione degli interlocutori',
    prepareSpeakersDetail:
      'A LocaLog servono due file di modello locali verificati prima di poter aggiungere nomi provvisori. La sua registrazione resta su questo dispositivo.',
    preparing: (percent: number) => `Preparazione ${percent} %`,
    prepare: 'Prepara',
    prepareWithSize: (size: string) => `Prepara (${size})`,
    speakerRuntimeMissing:
      'L’ambiente per la separazione degli interlocutori non è disponibile in questa installazione. La trascrizione può proseguire, ma userà nomi generici modificabili.',
    reviewAndTrim: 'Riveda e tagli prima la registrazione',
    trimDetail:
      '— tolga l’attesa prima dell’inizio e tutto ciò che alla riunione non serve. La sua registrazione non viene mai modificata.',
    gettingReady: 'Preparazione alla trascrizione…',
    useJobControls: 'Usi i comandi qui sopra',
    prepareSpeakersFirst: 'Prepari prima la separazione degli interlocutori',
    transcribe: 'Trascrivi',
    transcriptionFailedToStart: 'Non è stato possibile avviare la trascrizione. Riprovi.',
    transcriptReady: 'Trascrizione pronta',
    reviewBeforeGeneration: 'Da rivedere prima della generazione',
    transcriptReadyDetail:
      'La trascrizione con i tempi è pronta per le correzioni e per l’attribuzione degli interlocutori.',
    reviewTranscript: 'Rivedi la trascrizione',
    protocolAvailable: 'Verbale disponibile',
    continueInEditor: 'Prosegui nell’editor',
    protocolDetail: 'La trascrizione resta disponibile accanto alla versione attuale del verbale.',
    openProtocol: 'Apri il verbale',
  },

  newMeeting: {
    meetingOverride: 'Impostazione propria di questa riunione',
    preparing: 'Preparazione…',
    bringingRecordingIn: 'Acquisizione della registrazione…',
    noPerMeetingOverrides:
      'Le impostazioni proprie di una riunione e la scelta dei nomi e termini riunione per riunione non sono ancora disponibili.',
    chosenOnceNote:
      'La qualità della trascrizione e il modello che redige il verbale si scelgono una volta, nelle Impostazioni, e valgono per ogni riunione.',
    titleRecording: 'Registrazione',
    titleImport: 'Importazione strutturata',
    heading: 'Nuova riunione',
    leadRecording:
      'Dia un nome alla riunione e scelga il progetto. La registrazione comincia nella schermata successiva.',
    leadImport: 'Scelga la registrazione, confermi i dati, e LocaLog fa il resto.',
    context: 'Contesto',
    chooseProject: 'Scegli un progetto',
    project: 'Progetto',
    newProject: 'Nuovo progetto',
    noInbox:
      'Ogni origine appartiene a una riunione, e ogni riunione a un progetto. Non c’è una posta in arrivo.',
    source: 'Origine',
    importRecording: 'Importa una registrazione',
    originalStays: 'Il suo originale resta dov’è',
    readyToCopy: 'Pronto da copiare dopo che avrà confermato questa riunione',
    letGoToImport: 'Lasci per importare',
    originalStaysShort: 'L’originale resta dov’è.',
    dropHere: 'Trascini qui una registrazione, o clicchi per sceglierne una',
    dropDetail:
      'MP3, M4A, WAV, MP4, MOV e altri. L’originale resta intatto: LocaLog lo copia nel proprio archivio.',
    readyToAssign: 'Pronto da assegnare a questa riunione',
    chooseFile: 'Scegli un file audio o video',
    previewNote: 'L’anteprima nel browser mostra il percorso senza conservare il file.',
    useDemoRecording: 'Usa la registrazione dimostrativa',
    essentials: 'L’essenziale',
    meetingInformation: 'Dati della riunione',
    title: 'Titolo',
    titlePlaceholder: 'Ricavato dal file se lasciato vuoto',
    date: 'Data',
    language: 'Lingua della riunione',
    protocolStyle: 'Stile del verbale',
    projectDefault: 'Valore predefinito del progetto',
    qualityNote:
      'La qualità della trascrizione si sceglie una volta nelle Impostazioni e vale per ogni riunione.',
    advanced: 'Opzioni di elaborazione avanzate',
    cancel: 'Annulla',
    createAndRecord: 'Crea la riunione e registra',
    createAndImport: 'Crea la riunione e importa',
  },

  recordingReview: {
    lead: 'Tagli ciò che alla riunione non serve prima della trascrizione. La sua registrazione non viene mai modificata: tutto qui si può annullare.',
    noPreparedAudio:
      'Questa riunione non ha ancora audio preparato da rivedere. Sarà disponibile una volta confermata l’importazione.',
    dragToSelect:
      'Trascini sulla registrazione per selezionare un tratto, oppure usi i tasti freccia tenendo premuto Maiuscole.',
    selectedRange: (from: string, to: string) => `Selezionato da ${from} a ${to}.`,
    eyebrow: 'Registrazione',
    heading: 'Rivedi la registrazione',
    noAudio: 'Ancora nessun audio di lavoro',
    waveformLabel:
      'La registrazione. Si sposti con i tasti freccia, tenga premuto Maiuscole per selezionare.',
    keptOf: (kept: string, whole: string) => `${kept} di ${whole} conservati`,
    startsAt: (time: string) => `Inizia a ${time}`,
    endsAt: (time: string) => `Finisce a ${time}`,
    removedSpan: (from: string, to: string) => `Tolto da ${from} a ${to}`,
    startHere: 'Inizia qui',
    removeSelection: 'Togli la selezione',
    endHere: 'Finisci qui',
    edits: 'Tagli',
    nothingRemoved: 'Non è stato tolto nulla. Verrà trascritta tutta la registrazione.',
    undo: 'Annulla',
    putEverythingBack: 'Rimetti tutto',
    untouchedNote: 'La registrazione in sé è intatta. Queste sono indicazioni su che cosa usare.',
    undoStartTrim: 'Annulla il taglio iniziale',
    undoEndTrim: 'Annulla il taglio finale',
    putStretchBack: 'Rimetti questo tratto',
    next: 'Avanti',
    continueToTranscription: 'Passa alla trascrizione',
    backToMeeting: 'Torna alla riunione',
  },

  transcript: {
    heardAs: (heard: string) => `Sentito come «${heard}»`,
    askAboutTheRest: 'Esamina il resto',
    askingAboutTheRest: 'Lettura delle frasi…',
    askAboutTheRestNote:
      'Poche parole vengono sentite male in modo diverso ogni volta, quindi correggere una grafia non le trova. Questo legge ciascuna nella sua frase e propone un nome dall’elenco di questo progetto: non può proporre altro, e non cambia nulla finché non lo dice lei.',
    proposedNothing: 'Non è stato riconosciuto altro.',
    proposedNothingNote:
      'Che è la risposta abituale, ed è una buona risposta: può proporre solo un nome che questo progetto ha già, quindi preferisce tacere anziché inventarne uno.',
    proposalsHeading: (count: number) => (count === 1 ? '1 proposta' : `${count} proposte`),
    proposalSuggests: (heard: string, suggested: string) => `${heard} → ${suggested}`,
    spellingsToCheck: (count: number) =>
      count === 1 ? '1 grafia da verificare' : `${count} grafie da verificare`,
    questionedByProtocol: 'il verbale non ha riconosciuto questa parola',
    autosaveFailed:
      'Il salvataggio automatico non è riuscito — il suo ultimo stato salvato è intatto',
    correctCount: (count: number) => `Correggi ${count}`,
    audioCouldNotLoad: 'Non è stato possibile caricare l’audio di lavoro di questa riunione.',
    pauseAudio: 'Metti in pausa',
    playAudio: 'Riproduci',
    saving: 'Salvataggio…',
    editsSaved: 'Modifiche salvate',
    revisionSaved: 'Versione della trascrizione salvata',
    separationUnavailableHere:
      'La separazione degli interlocutori non è ancora disponibile in questa installazione. Può proseguire mettendo i nomi a mano.',
    rerunForSeparation:
      'Riesegua questa trascrizione per ottenere un risultato di separazione aggiornato.',
    separationUnavailableForRun:
      'La separazione degli interlocutori non era disponibile per questa esecuzione. Può proseguire mettendo i nomi a mano.',
    nothingChangedYet: 'Ancora nulla di cambiato',
    readingOpening: 'Lettura dell’inizio…',
    readWhoIsHere: 'Leggi chi partecipa a questa riunione',
    correcting: 'Correzione…',
    durationPending: 'Durata da determinare',
    introducedThemselves: (count: number) => `${count} si sono presentati`,
    noNamesYet: (project: string) => `Ancora nessun nome in ${project}`,
    speltAsHeard:
      'Scritti come li ha sentiti la trascrizione. Corregga quelli sbagliati: verranno corretti qui e ricordati per questo progetto.',
    openingNote:
      'Una riunione di solito si apre con chi dice chi è. Leggere quel passaggio dà a questo progetto i suoi nomi, cioè ciò che una trascrizione non può indovinare.',
    foundInPlaces: (count: number) =>
      `Trovato in ${count} ${count === 1 ? 'punto' : 'punti'}. Tolga la spunta a quelli che devono restare come sono.`,
    noneMisheardEveryTime: (count: number) =>
      `Nessuna parola è stata sentita male tutte le volte che è comparsa. Restano ${count} passaggi segnalati come poco chiari per altri motivi.`,
    nothingFlaggedNote:
      'Non è stato segnalato nulla come poco chiaro. Anche una trascrizione fatta prima che questo esistesse non mostra nulla qui, quindi conviene rileggere una trascrizione vecchia anziché fidarsene.',
    workingAudioLater: 'L’audio di lavoro sarà disponibile una volta trascritta questa riunione.',
    recordingEndsNote:
      'Se la riunione è proseguita oltre, la registrazione non l’ha captato e il verbale non lo conterrà.',
    heading: 'Revisione della trascrizione',
    exportTranscript: 'Esporta la trascrizione…',
    exportLabel: 'Esporta questa trascrizione',
    asMarkdown: 'In Markdown',
    asPlainText: 'In testo semplice',
    reviewDetails: 'Dettagli della revisione',
    sourceContext: 'Contesto dell’origine',
    seekAudio: 'Spostati nell’audio',
    follow: 'Segui',
    followLabel: 'Fai scorrere la trascrizione fino al passaggio in riproduzione',
    searchTranscript: 'Cerca nella trascrizione',
    editableTranscript: 'Trascrizione modificabile',
    removeLine: 'Togli questa riga dalla trascrizione',
    nothingFlagged: 'Nulla segnalato come poco chiaro',
    show: 'Mostra',
    showing: 'In mostra',
    onePassage: '1 passaggio poco chiaro',
    manyPassages: (count: number) => `${count} passaggi poco chiari`,
    speakerHint:
      'I nomi degli interlocutori sono un punto di partenza: li sostituisca con le persone che hanno parlato.',
    generateProtocol: 'Genera il verbale',
    review: 'Revisione',
    detailsLabel: 'Dettagli della revisione della trascrizione',
    closeInspector: 'Chiudi il pannello',
    speakers: 'Interlocutori',
    whereRecordingStops: 'Dove finisce la registrazione',
    transcriptionInput: 'Ingresso della trascrizione',
    language: 'Lingua',
    meetingLanguage: 'Lingua della riunione',
    saveLanguage: 'Salva la lingua',
    cancel: 'Annulla',
    changeLanguage: 'Cambia la lingua',
    rerunNote:
      'Da usare dopo aver cambiato la lingua o le impostazioni di trascrizione. La nuova esecuzione viene conservata come versione a sé.',
    rerun: 'Riesegui la trascrizione',
    rerunPreparing: 'Preparazione di una nuova trascrizione…',
    rerunConfirm: (language: string) =>
      `Rieseguire la trascrizione in ${language}? La trascrizione attuale resterà finché il nuovo risultato non sarà confermato, poi questa trascrizione di lavoro verrà sostituita.`,
    whoIsHere: 'Chi partecipa a questa riunione',
    close: 'Chiudi',
    aboutAMinute: 'Circa un minuto. Nel frattempo non può girare nient’altro.',
    unsureNames: 'Nomi che meritano un secondo sguardo',
    whatShouldItSay: 'Come dovrebbe essere scritto?',
    rememberForProject: 'Ricordalo per questo progetto, così la prossima riunione lo scriverà bene',
    areAnyNames:
      'Qualcuno di questi è un nome? Correggerne uno sistema questa trascrizione e viene ricordato.',
    nothingToCheck: 'Nulla da verificare',
    correctSpelling: 'Correggi la grafia',
    checkWording: 'Verifica la formulazione',
    checkWords: (words: string) => `Verifichi ${words}`,
    textAt: (time: string) => `Testo della trascrizione a ${time}`,
    jumpTo: (time: string) => `Andare a ${time}`,
    removeLineAt: (time: string) => `Togliere la riga a ${time}`,
    renameSpeaker: (speaker: string) => `Rinominare ${speaker}`,
    nameHeardAs: (heard: string) => `Nome sentito come ${heard}`,
    protocolStyle: 'Stile del verbale',
    audioUnplayable: 'Non è stato possibile riprodurre l’audio di lavoro di questa riunione.',
    speakersResolved:
      'I turni di parola sono stati risolti in locale. I nomi sono provvisori: li sostituisca solo quando sa di chi si tratta.',
    speakersFailed:
      'La separazione degli interlocutori non ha prodotto turni utilizzabili in questa esecuzione. La trascrizione è intatta e usa nomi neutri; può proseguire mettendoli a mano.',
    speakersUnavailable:
      'La separazione degli interlocutori non era disponibile per questa esecuzione. La trascrizione è intatta e usa un nome neutro; può comunque sostituirlo a mano.',
    speakersUnknown:
      'Questa trascrizione più vecchia non registra se la separazione degli interlocutori sia stata eseguita. I suoi nomi neutri non provano che parlasse una sola persona.',
  },

  library: {
    remove: 'Togli',
    edit: 'Modifica',
    keep: 'Conserva',
    notInUseSuffix: ' · non in uso',
    /** Si veda la nota in en.ts: solo finché uno stile non è stato rinominato. */
    shippedStyle: {
      'style-formal': {
        name: 'Verbale formale',
        description: 'Resoconto strutturato della discussione, delle decisioni e delle azioni.',
      },
      'style-working-note': {
        name: 'Nota di lavoro interna',
        description: 'Resoconto di lavoro conciso per una squadra di progetto interna.',
      },
      'style-decision-log': {
        name: 'Registro delle decisioni tecniche',
        description: 'Mette in evidenza alternative, vincoli e decisioni esplicite.',
      },
    },
    copyOf: (name: string) => `${name} (copia)`,
    enterATerm: 'Inserisca un termine.',
    reading: 'Lettura…',
    editTerm: 'Modifica il termine',
    inUse: 'In uso',
    notInUse: 'Non in uso',
    instructionsGiven:
      'Queste sono le istruzioni che vengono date al modello, nell’ordine in cui gli vengono date',
    asShipped: ', esattamente come questo stile è stato fornito',
    invariantsNote:
      'Non fanno parte di questo stile e non si possono modificare qui: non sono conservate con nessuno stile. Vengono aggiunte a ogni verbale mentre viene redatto, perché un documento che riporta una decisione che nessuno ha preso non è un verbale di stile diverso, ma un verbale sbagliato.',
    whichTermsHelp:
      'I nomi, le imprese e le sigle sono ciò che aiuta di più. La terminologia professionale corrente di solito viene trascritta bene senza essere elencata.',
    termsLeadLong:
      'Aggiunga i nomi, le imprese e le sigle che questo lavoro usa, così saranno trascritti bene. Su una riunione reale di ottanta minuti questo ha portato il nome del progetto stesso da mai scritto correttamente a sempre corretto.',
    eyebrow: 'Libreria',
    protocolStyles: 'Stili di verbale',
    namesAndTerms: 'Nomi e termini',
    stylesLead:
      'Che cosa dice un verbale, e in quale ordine. Non come è impaginato: quello è l’aspetto, e vive nell’editor accanto al documento che descrive.',
    termsLead:
      'I nomi che una trascrizione non può indovinare: il suo progetto, le imprese, le persone. Misurati su una riunione reale, valgono più di qualsiasi altra impostazione qui.',
    addTerm: 'Aggiungi un termine',
    saveTerm: 'Salva il termine',
    stylesUnreadable: 'Qui non si possono leggere gli stili.',
    length: 'Lunghezza',
    name: 'Nome',
    description: 'Descrizione',
    whatItAsksFor: 'Che cosa chiede questo stile',
    addInstruction: 'Aggiungi un’istruzione',
    removeInstruction: 'Togli questa istruzione',
    checkedOnProtocol: 'Verificato sul verbale finito',
    alwaysEveryStyle: 'Sempre, in ogni stile',
    saveStyle: 'Salva lo stile',
    cancel: 'Annulla',
    delete: 'Elimina',
    editThisStyle: 'Modifica questo stile',
    duplicate: 'Duplica',
    duplicateToEdit: 'Duplica per modificare',
    shippedStyleNote:
      'Uno stile fornito con l’applicazione resta com’è, così un verbale redatto l’anno scorso può essere redatto allo stesso modo oggi. Lo copi per farne uno suo.',
    ownershipAutomatic: 'L’attribuzione è automatica.',
    termsScopeNote:
      'I nomi e i termini di un progetto valgono per le sue riunioni senza doverli scegliere ogni volta.',
    term: 'Termine',
    spellingAsShown: 'Grafia come deve comparire',
    category: 'Categoria',
    appliesTo: 'Si applica a',
    everyProject: 'Tutti i progetti',
    unknownProject: 'Progetto sconosciuto',
    noTerms: 'Ancora nessun nome né termine',
    deleteThisTerm: 'Eliminare questo termine?',
    densityFull: 'Prosa completa',
    densityPlain: 'Enunciati semplici',
    densityLine: 'Una riga per punto',
    densityFullMeaning: 'Prosa completa. Chi non c’era può seguire la discussione.',
    densityPlainMeaning: 'Enunciati semplici. Ciò che è stato detto, senza il racconto.',
    densityLineMeaning: 'Una riga per punto. Il registro, e nulla intorno.',
    categoryPerson: 'Persona',
    categoryOrganisation: 'Impresa',
    categoryProject: 'Progetto',
    categoryAbbreviation: 'Sigla',
    categoryTechnicalTerm: 'Termine tecnico',
    categoryOther: 'Altro',
  },

  furniture: {
    header: 'Intestazione',
    footer: 'Piè di pagina',
    left: 'Sinistra',
    centre: 'Centro',
    insertInto: (where: string) => `Inserire un valore in ${where}`,
    right: 'Destra',
    insert: 'Inserisci…',
    lineHint:
      'Scriva la riga come deve leggersi, e ci metta un valore dove ne vuole uno: «Pagina », il numero, « di 12». Un valore è un solo oggetto: si seleziona e si cancella per intero.',
    appliesTo: (project: string) =>
      `Si applica a tutti i verbali di ${project}. Si ripete sulla pagina stampata e non fa parte del documento che sta modificando.`,
  },

  shell: {
    breadcrumbMeeting: 'Riunione',
    breadcrumbRecording: 'Registrazione',
    breadcrumbReview: 'Revisione',
    skipToWorkspace: 'Vai allo spazio di lavoro',
    workspace: 'Spazio di lavoro',
    workspaceFailed: 'Non è stato possibile aprire lo spazio di lavoro',
    workspaceFailedDetail: 'I suoi file esistenti non sono stati modificati.',
    tryAgain: 'Riprova',
    preparingWorkspace: 'Preparazione dello spazio di lavoro locale…',
    openNavigation: 'Apri la navigazione',

    notSelected: 'Non selezionato',

    jobNeedsDecision: 'Richiede la sua decisione',
    jobReadyToContinue: 'Pronto a proseguire',
    jobCancelling: 'Annullamento in sicurezza',

    formatWordDocument: 'Documento Word',
    formatPlainText: 'Testo semplice',
    exportSaved: (format: string) => `Esportazione in ${format} salvata`,
    exportFailed: (format: string, why: string) => `Esportazione in ${format} non riuscita: ${why}`,
    exportPrepared: (format: string) => `Esportazione in ${format} preparata`,
    exportNeedsDesktop: (format: string) =>
      `L’esportazione in ${format} richiede l’applicazione desktop.`,

    meetingArchived: 'Riunione archiviata. È in Impostazioni, sotto Archiviazione.',
    projectArchived: 'Progetto archiviato. È in Impostazioni, sotto Archiviazione.',
    transcriptExported: 'Trascrizione esportata',
  },

  protocol: {
    undo: 'Annulla',
    redo: 'Ripristina',
    next: 'Avanti',
    blockParagraph: 'Paragrafo',
    blockHeading1: 'Titolo 1',
    blockHeading2: 'Titolo 2',
    blockHeading3: 'Titolo 3',
    figuresMissingFromRewrite: (count: number) =>
      `In questa riformulazione mancano ${count} cifre che il passaggio riportava`,
    markdownView: 'Vista Markdown',
    documentView: 'Vista documento',
    looking: 'Ricerca…',
    replaceAll: 'Sostituisci tutto',
    rewrite: 'Riformula',
    rewriting: 'Riformulazione',
    figureMissingFromRewrite: 'In questa riformulazione manca una cifra che il passaggio riportava',
    reviewedRevisionPreserved:
      'La versione rivista è conservata. Queste modifiche di lavoro non sono state riviste.',
    thisRevisionReviewed: 'Questa esatta versione immutabile è stata contrassegnata come rivista.',
    generatedStaysEditable: 'Il contenuto generato resta rivedibile e modificabile.',
    notFound: 'Non trovato',
    matchCount: (count: number) => `${count} ${count === 1 ? 'corrispondenza' : 'corrispondenze'}`,
    replacedCount: (count: number) => ` · ${count} sostituite`,
    changesNotYetMade: (count: number) =>
      `${count} ${count === 1 ? 'modifica' : 'modifiche'}, non ancora applicate`,
    compoundNote:
      'Un nome con l’iniziale maiuscola viene cercato anche dentro le parole composte, dove una semplice sostituzione lo perde. Li legga, poi li tenga o li lasci.',
    andMore: (count: number) => `e altre ${count}, tutte delle stesse due forme.`,
    passageGoesAlone:
      'Il passaggio va da solo al suo modello locale. Numeri, nomi e date devono tornare invariati: li verifichi, e annulli se non è così.',
    nothingChangedYet:
      'Non è ancora stato cambiato nulla. Legga, poi tenga o lasci: un modello locale riformula bene e non va creduto sulla parola.',
    secondPassNote:
      'Chiesto al suo stesso modello, e sbaglia in entrambe le direzioni: si perde delle modifiche e segnala formulazioni che vanno bene. Merita uno sguardo, non un verdetto.',
    pageEdgesNote:
      'Dove finirebbero le pagine, misurato come le compone il foglio di stile per la stampa: un titolo o una tabella scendono interi anziché spezzarsi, la prosa no. La stampante decide l’ultima riga o due, quindi lo prenda a una riga di distanza e non al millimetro.',
    transcriptSourceNote:
      'Redatto a partire dalla trascrizione rivista di questa riunione. Nulla registra quale passaggio abbia prodotto quale frase, quindi ciò che segue cerca le parole anziché sostenere di saperlo: una parafrasi non troverà nulla, che è la risposta onesta.',
    noWordsTogether:
      'Queste parole non compaiono insieme nella trascrizione. Di solito vuol dire che la bozza l’ha detto con parole sue, cosa che le è consentita: è nella registrazione che va verificato.',
    revisionNote:
      'Ciò che scrive viene conservato come modifiche di lavoro e non crea una versione. Una versione si crea quando viene generata una bozza, quando ne chiede una, quando contrassegna un verbale come rivisto e quando ne viene ripristinato uno più vecchio, così questo elenco resta leggibile.',
    nothingRewrites:
      'Qui non c’è nulla che riscriva il suo testo al posto suo. La bozza è sua, e ogni versione viene conservata.',
    figuresKept: (kept: number, stated: number) => `${kept} cifre su ${stated} conservate`,
    figuresNote: (stated: number, kept: number) =>
      `La riunione ha riportato ${stated} cifre e questa bozza ne ripete ${kept}. Quante debbano starci dipende dallo stile che ha scelto, quindi è qualcosa da guardare e non un voto.`,
    figuresInvented: (count: number) =>
      count === 1
        ? 'Qui compare una cifra che la riunione non ha riportato'
        : `Qui compaiono ${count} cifre che la riunione non ha riportato`,
    confirmAgainstRecording: '. Conviene verificarlo sulla registrazione.',
    tasksUnowned: (count: number) =>
      count === 1
        ? 'Qui c’è un compito senza nessuno accanto'
        : `Qui ci sono ${count} compiti senza nessuno accanto`,
    unownedNote:
      '. La bozza preferisce lasciare fuori un responsabile anziché indovinarlo, quindi può essere esattamente ciò che la riunione ha deciso — e mettere un nome adesso costa molto meno che alla riunione successiva.',
    editor: 'Editor dei verbali',
    markdownBacked: 'basato su Markdown',
    noteMissingTableHeading: 'Nessuna tabella dei prossimi passi',
    noteMissingTableBody:
      'Questo verbale è stato redatto tre volte e nessuna delle versioni si è conclusa con una tabella dei compiti concordati e dei loro responsabili. Le azioni decise in riunione sono descritte nelle sezioni qui sopra ma non sono raccolte qui.',
    noteGapsHeading: 'Non coperto da questo verbale',
    noteOneGap:
      'Un tratto della registrazione non è stato possibile leggerlo, e nulla di quanto precede lo descrive. La registrazione in sé è completa e si può ancora ascoltare.',
    noteSeveralGaps:
      'Più tratti della registrazione non è stato possibile leggerli, e nulla di quanto precede li descrive. La registrazione in sé è completa e quei tratti si possono ancora ascoltare.',
    documentType: 'Verbale',
    statusDraft: 'Bozza',
    statusReviewed: 'Rivisto',
    statusChanged: 'Modificato dopo la revisione',
    fieldProjectName: 'Nome del progetto',
    fieldMeetingTitle: 'Titolo della riunione',
    fieldMeetingDate: 'Data della riunione',
    fieldDocumentType: 'Tipo di documento',
    fieldProtocolStatus: 'Stato',
    fieldPageNumber: 'Numero di pagina',
    fieldPageOfCount: 'Pagina n di m',
    fieldText: 'Testo libero',
    showPageBreaks: 'Mostra le interruzioni di pagina',
    hidePageBreaks: 'Nascondi le interruzioni di pagina',
    saving: 'Salvataggio…',
    autosaveFailed: 'Salvataggio automatico non riuscito',
    workingEditsSaved: 'Modifiche di lavoro salvate',
    revisionSaved: 'Versione salvata',
    editorTools: 'Strumenti',
    find: 'Cerca',
    findInProtocol: 'Cerca nel verbale',
    replaceWith: 'Sostituisci con',
    makeChanges: 'Applica queste modifiche',
    leaveIt: 'Lascia',
    zoomOut: 'Riduci',
    zoomIn: 'Ingrandisci',
    insertTable: 'Inserisci una tabella',
    insertDivider: 'Inserisci un separatore',
    documentMenu: 'Menu del documento',
    clearFormatting: 'Togli la formattazione',
    table: 'Tabella',
    blockType: 'Tipo di blocco',
    addColumnLeft: 'Aggiungi una colonna a sinistra',
    addColumnRight: 'Aggiungi una colonna a destra',
    deleteColumn: 'Elimina questa colonna',
    addRowAbove: 'Aggiungi una riga sopra',
    addRowBelow: 'Aggiungi una riga sotto',
    deleteRow: 'Elimina questa riga',
    formatting: 'Formattazione',
    bold: 'Grassetto',
    italic: 'Corsivo',
    bulletedList: 'Elenco puntato',
    numberedList: 'Elenco numerato',
    quotation: 'Citazione',
    askModel: 'Chiedi al modello di dirlo diversamente',
    customInstruction: 'Istruzione personalizzata…',
    whatShouldChange: 'Che cosa va cambiato?',
    proposedChange: 'Modifica proposta',
    proposedReplacement: 'Sostituzione proposta',
    proposedRewrite: 'Riformulazione proposta',
    unchanged: 'Il modello ha restituito il passaggio invariato.',
    factsMoved: 'Una seconda passata ritiene che questi dati si siano spostati',
    noFactMoved: 'Una seconda passata non ha visto spostarsi alcun dato. Qualcosa le sfugge.',
    useThis: 'Usa questo',
    improveClarity: 'Rendi più chiaro',
    improveClarityInstruction: 'Rendi questo passaggio più chiaro da leggere.',
    makeFormal: 'Rendi più formale',
    makeFormalInstruction:
      'Usa un registro più formale, come si redigerebbe un verbale professionale.',
    makePlainer: 'Rendi più diretto',
    makePlainerInstruction:
      'Rendi la formulazione più semplice e diretta, senza perdere precisione.',
    shorten: 'Accorcia',
    shortenInstruction: 'Di’ questo con meno parole.',
    rewriteUnavailable: 'Qui la riformulazione non è disponibile.',
    replaceUnavailable: 'Qui la sostituzione di un nome non è disponibile.',
    nameNotFound: 'Quel nome non è in questo verbale.',
    protocolMarkdown: 'Markdown del verbale',
    protocolLabel: 'Verbale',
    protocolDetails: 'Dettagli del verbale',
    documentDetails: 'Dettagli del documento',
    closeInspector: 'Chiudi il pannello',
    tabDocument: 'Documento',
    tabTranscript: 'Trascrizione',
    tabHistory: 'Cronologia',
    status: 'Stato',
    createRevision: 'Crea una versione',
    lineNumber: (line: number) => `Riga ${line}`,
    pageNumber: (page: number) => `Pagina ${page}`,
    revisionNumber: (ordinal: number) => `Versione ${ordinal}`,
    markReviewed: 'Contrassegna come rivisto',
    style: 'Stile',
    sections: 'Sezioni',
    newSection: 'Nuova sezione',
    appearance: 'Impaginazione',
    editAppearance: 'Modifica l’impaginazione',
    headerFooter: 'Intestazione e piè di pagina',
    editHeaderFooter: 'Modifica l’intestazione e il piè di pagina',
    nothingRepeated: 'Nulla si ripete sulla pagina',
    presets: 'Preimpostazioni',
    useOrSavePreset: 'Usa o salva una preimpostazione',
    noneSaved: 'Ancora nessuna salvata',
    savedCount: (count: number) => `${count} salvate`,
    use: 'Usa',
    remove: 'Togli',
    nameThisPreset: 'Dai un nome a questa preimpostazione',
    nameForPreset: 'Nome di questa preimpostazione',
    save: 'Salva',
    cancel: 'Annulla',
    saveAsPreset: 'Salva questa impaginazione e questa intestazione come preimpostazione',
    export: 'Esporta',
    exportPdf: 'Esporta in PDF',
    exportWord: 'Esporta in Word',
    exportMarkdown: 'Esporta in Markdown',
    exportPlainText: 'Esporta in testo semplice',
    exportNote:
      'Il PDF viene stampato dal documento che sta leggendo, impaginato come questo progetto impagina i suoi verbali: scelga «Salva come PDF» nella finestra di stampa.',
    source: 'Origine',
    findSelectedPassage: 'Trova il passaggio selezionato',
    lookingFor: 'In cerca di:',
    openReviewedTranscript: 'Apri la trascrizione rivista',
    whatToCheck: 'Che cosa verificare',
    revisions: 'Versioni',
    current: 'Attuale',
    restore: 'Ripristina',
  },

  sidebar: {
    projects: 'Progetti',
    newProject: 'Nuovo progetto',
    createProject: 'Crea il progetto',
    library: 'Libreria',
    protocolStyles: 'Stili di verbale',
    namesAndTerms: 'Nomi e termini',
    settings: 'Impostazioni',
    recording: 'Registrazione',
    primaryNavigation: 'Navigazione principale',
    closeNavigation: 'Chiudi la navigazione',
    openNavigation: 'Apri la navigazione',
    themeFollowingSystem: 'Segue il tema del sistema. Passa a sempre chiaro.',
    themeAlwaysLight: 'Sempre chiaro. Passa a sempre scuro.',
    themeAlwaysDark: 'Sempre scuro. Torna al tema del sistema.',
    themeFollowingShort: 'Segue il sistema',
    sidebarWidth: (width: number) => `${width} pixel`,
    resizeSidebar:
      'Ridimensiona il pannello. Usi i tasti freccia per regolarlo, o Invio per ripristinarlo.',
    themeAlwaysLightShort: 'Sempre chiaro',
    themeAlwaysDarkShort: 'Sempre scuro',

    importNeedsDecision: 'L’importazione richiede la sua decisione',
    needsAttention: 'Richiede la sua attenzione',
    importingRecording: 'Importazione della registrazione',
    transcribing: 'Trascrizione in corso',
    writingProtocol: 'Stesura del verbale',
    working: 'In corso',
    workingEllipsis: 'In corso…',
    separatingSpeakers: 'Separazione degli interlocutori',
    openMeetingNeedingAttention: 'Apri la riunione che richiede attenzione',
    openThisMeeting: 'Apri questa riunione',
  },

  start: {
    eyebrow: 'IA locale per verbali di riunione riservati',
    title: 'Comincia una riunione',
    lead: 'Importi un file audio o video. Verifichi ogni passaggio prima che diventi un verbale.',
    importTitle: 'Importa una registrazione',
    importDetail: 'Scelga un progetto, e tenga tutto nel suo contesto',
    recordTitle: 'Registra una riunione',
    recordDetail: 'Capti la stanza e la chiamata su questo dispositivo, su tracce separate',
    promiseTitle: 'Il suo lavoro sulle riunioni resta su questo dispositivo.',
    promiseDetail: 'Nessun account LocaLog, nessun servizio in rete, nessuna telemetria.',

    setupTitle: 'Uno scaricamento prima della prima trascrizione',
    setupBody: (quality: string, size: string) =>
      `LocaLog trascrive su questo dispositivo, quindi il modello deve stare qui. La qualità ${quality} occupa ${size} e si scarica una volta sola. Può importare prima una registrazione: il modello serve quando comincia la trascrizione, non prima.`,
    setupDownload: (size: string) => `Scaricalo adesso (${size})`,
    setupCancel: 'Annulla',
    setupAside: 'Le altre qualità, e la separazione degli interlocutori, sono nelle Impostazioni.',
  },
};
