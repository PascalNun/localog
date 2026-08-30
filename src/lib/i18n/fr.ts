/**
 * Every word the application says, in French.
 *
 * Typed against English, so this file cannot be missing a key or inventing one.
 *
 * ## Decisions taken once, here, so the whole application reads as one voice
 *
 * **Compte rendu, never "protocole".** This is the decision the whole file rests
 * on. *Protocole* in French is a procedure or a diplomatic form; it is not what a
 * meeting produces, and using it would mark the interface as translated by
 * somebody who had not read it. The professional record of a meeting is a
 * **compte rendu** — what a French architecture or engineering office actually
 * circulates. *Procès-verbal* exists but belongs to a constituted body taking
 * formal minutes, and would overstate most of these.
 *
 * **Vous, not tu**, matching the German Sie. Written for offices keeping the
 * record of formal meetings.
 *
 * **Réunion for a meeting, enregistrement for a recording, piste for a track.**
 * The ordinary words, not calques.
 *
 * **Transcription** for the machine's reading of the audio, and **transcrire**
 * for the verb. *Retranscription* is common in speech but names the act of
 * copying out, which is not what happens here.
 *
 * French takes a space before `?`, `!`, `:` and `;` — a narrow no-break space,
 * written here as the ordinary one because the rendering stack does not
 * substitute it. Quotation marks are « guillemets » where the English uses curly
 * quotes.
 */

import type { Strings } from './en';

const failures = {
  missingProject: 'Le projet sélectionné n’existe plus.',
  missingMeeting: 'La réunion sélectionnée n’existe plus.',
  missingJob: 'La tâche d’importation n’est plus disponible.',
  importBusy:
    'Un enregistrement est déjà en cours d’importation. Terminez-le ou annulez-le d’abord.',
  unsupportedSchema: (version: string) =>
    `Ces données LocaLog ont été créées par une version plus récente et non prise en charge (${version}).`,
  storageUnavailable: 'LocaLog n’a pas pu accéder à son espace de travail local.',

  styleMissing: 'Ce style n’existe plus.',
  styleNameRequired: 'Donnez un nom au style.',
  styleNotSaved: 'Le style n’a pas pu être enregistré.',
  styleUnavailable: 'Le style de compte rendu sélectionné n’est pas disponible.',
  styleUsedByMeeting: 'Une réunion utilise ce style. Modifiez-la d’abord.',
  styleUsedByProject: 'Un projet utilise ce style par défaut. Modifiez-le d’abord.',

  presetNameRequired: 'Donnez un nom au préréglage.',
  presetNotSaved: 'Le préréglage n’a pas pu être enregistré.',
  presetBuiltInUndeletable: 'Un préréglage livré avec LocaLog ne peut pas être supprimé.',

  transcriptInvalid: 'La transcription enregistrée n’est pas valide.',
  transcriptSegmentMissing: 'Ce passage de la transcription n’existe plus.',
  transcriptTextRequired: 'Saisissez un texte de transcription valide.',
  transcriptNeedsSegment: 'Une transcription a besoin d’au moins un passage.',
  transcriptSpeakerRequired: 'Saisissez un nom d’intervenant valide.',
  transcriptNotSaved: 'La transcription n’a pas pu être enregistrée.',
  transcriptNotCommitted: 'La transcription n’a pas pu être validée.',
  spellingRequired: 'Saisissez une orthographe valide.',

  protocolTextRequired: 'Saisissez un texte de compte rendu valide.',
  protocolRevisionMissing: 'La version du compte rendu sélectionnée n’existe plus.',
  protocolNeededBeforeExport: 'Générez un compte rendu avant de l’exporter.',
  protocolNeededBeforeSetAside: 'Générez un compte rendu avant d’en mettre une partie de côté.',
  sectionNotSetAside: 'Cette section n’a pas pu être mise de côté.',
  reviewBeforeGeneration: 'Relisez la transcription avant la génération.',
  vocabularyUnresolved: 'Les noms et termes n’ont pas pu être résolus.',

  selectionRequired: 'Sélectionnez le texte à modifier.',
  selectionTooLong:
    'Cela fait trop de texte à modifier d’un coup. Sélectionnez une section plutôt que le document.',
  passageNotRewritten: 'Ce passage n’a pas pu être reformulé.',
  openingNotRead: 'Le début de la réunion n’a pas pu être lu.',
  providerNeededForPassage:
    'Démarrez votre installation Ollama avant de faire reformuler un passage.',
  providerNeededForOpening: 'Démarrez votre installation Ollama avant de lire les présentations.',
  providerNeededForCorrections:
    'Démarrez votre installation Ollama avant de vérifier ces orthographes.',
  providerModelRequired:
    'Choisissez un modèle Ollama installé dans Réglages → Génération du compte rendu.',

  styleNotMigrated: 'Un style n’a pas pu être migré.',
  termMissing: 'Ce terme n’existe plus.',
  exportFormatInvalid: 'Choisissez un format d’export valide.',
  meetingDateInvalid: 'Choisissez une date de réunion valide.',
  scopeInvalid: 'Choisissez une portée valide.',
  sourceFileInvalid: 'Choisissez un fichier source valide.',
  workspaceViewInvalid: 'Choisissez une vue d’espace de travail valide.',
  recordingUnreadable: 'Cet enregistrement n’a pas pu être lu.',
  appearanceNotSaved: 'La mise en forme n’a pas pu être enregistrée.',
  furnitureNotSaved: 'L’en-tête et le pied de page n’ont pas pu être enregistrés.',
  documentOperationFailed: 'L’opération locale sur le document n’a pas pu aboutir.',
  providerConfigNotSaved:
    'La configuration du fournisseur de comptes rendus n’a pas pu être enregistrée.',
  runtimeConfigNotSaved:
    'La configuration de l’environnement de transcription n’a pas pu être enregistrée.',
  recorderNotStarted: 'L’enregistreur n’a pas pu démarrer.',
  tracksNotCombined: 'Les pistes de l’enregistrement n’ont pas pu être réunies.',
  protocolInvalid: 'Le compte rendu enregistré n’est pas valide.',
  protocolNotUtf8: 'Le compte rendu enregistré n’est pas en UTF-8 valide.',
  editsNotRecorded: 'Ces modifications ne peuvent pas être enregistrées.',

  recordingAlreadyRunning: 'Une réunion est déjà en cours d’enregistrement.',
  presetUnknown: 'Choisissez une qualité de transcription connue.',
  providerModelNotInstalled: 'Choisissez un modèle déjà installé dans Ollama.',
  diariserPathInvalid: 'Choisissez un programme de séparation des intervenants existant.',
  whisperPathInvalid: 'Choisissez un exécutable whisper.cpp existant.',
  nothingRecording: 'Aucun enregistrement n’est en cours.',
  revealOnlyOnMac:
    'L’ouverture du dossier n’est câblée que sur macOS. Le chemin ci-dessus est correct.',
  privacySettingsOnlyOnMac:
    'L’ouverture des réglages de confidentialité n’est câblée que sur macOS.',
  providerNeededForModel: 'Démarrez votre installation Ollama avant de choisir un modèle.',
  settingsNotOpened: 'Les Réglages Système n’ont pas pu être ouverts.',
  presetMissing: 'Ce modèle d’export n’est plus disponible.',
  downloadStopped: 'Le téléchargement s’est interrompu.',
  coordinatorUnavailable: 'Le coordinateur d’importation n’est pas disponible.',
  taskStopped: 'La tâche d’annulation locale s’est interrompue.',
  recorderPermissionsUnknown: 'L’enregistreur n’a pas pu être interrogé sur ses autorisations.',
  recorderStateUnknown: 'L’enregistreur est dans un état inconnu. Redémarrez LocaLog.',
  recordingNotFinished: 'L’enregistrement n’a pas pu être terminé.',
  replacementNotPrepared: 'Le remplacement n’a pas pu être préparé.',
  workspaceNotOpened: 'Le dossier de l’espace de travail n’a pas pu être ouvert.',
  settingsPaneUnknown: 'Ce volet de réglages n’existe pas.',
  meetingBusy: 'Cette réunion est encore en cours de traitement. Annulez-le d’abord.',
  printDialogUnavailable: 'Cette fenêtre n’a pas pu ouvrir la boîte de dialogue d’impression.',

  backupNameUnsafe: 'Ce nom de sauvegarde ne peut pas servir de nom de dossier.',
  notABackup: 'Ce dossier n’est pas une sauvegarde LocaLog : il n’a pas de manifest.json.',
  backupPathOutside: (path: string) =>
    `Cette sauvegarde mentionne un fichier hors de son propre dossier (${path}) ; elle n’a donc pas été restaurée.`,
  backupFormatUnknown: (format: string) =>
    `Cette sauvegarde a été écrite au format ${format}, que cette version de LocaLog ne sait pas lire. Une version plus récente le saura.`,
  backupDamaged: (what: string) =>
    `Cette sauvegarde est incomplète ou endommagée (${what}) ; rien n’a donc été modifié. Votre travail actuel est intact.`,
  backupNameTaken: (name: string) =>
    `Il y a déjà quelque chose appelé « ${name} » dans ce dossier.`,
  backupIoFailed: (what: string) => `La sauvegarde n’a pas pu être écrite ni lue : ${what}`,
  backupDatabaseFailed: (what: string) => `La base de données n’a pas pu être copiée : ${what}`,

  categoryRequired: 'Choisissez une catégorie.',
  meetingLanguageRequired: 'Choisissez une langue de réunion.',
  meetingLanguageInvalid: 'Choisissez une langue de réunion valide.',
  meetingInvalid: 'Choisissez une réunion valide.',
  projectInvalid: 'Choisissez un projet valide.',
  styleInvalid: 'Choisissez un style de compte rendu valide.',
  sourceRecordingInvalid: 'Choisissez un enregistrement source valide.',
  meetingTitleRequired: 'Saisissez un titre de réunion.',
  projectNameRequired: 'Saisissez un nom de projet.',
  termRequired: 'Saisissez un terme.',
  meetingTitleTooLong: 'Le titre de la réunion est trop long.',
  speakerPassCannotRead: (what: string) =>
    `La passe des intervenants n’a pas pu lire l’audio de travail : ${what}`,
  speakerPassCannotWrite: (what: string) =>
    `La passe des intervenants n’a pas pu écrire son audio : ${what}`,
  recordingNotStored: (what: string) => `L’enregistrement n’a pas pu être stocké : ${what}`,
  recordingNotRead: (what: string) => `L’enregistrement n’a pas pu être lu : ${what}`,
  modelNotDownloaded: (what: string) => `Le modèle n’a pas pu être téléchargé : ${what}`,
  modelNotSaved: (what: string) => `Le modèle n’a pas pu être enregistré : ${what}`,
  ollamaRequestFailed: (what: string) => `Ollama n’a pas pu traiter la requête locale : ${what}`,
  recorderStartFailed: (what: string) => `L’enregistreur n’a pas pu démarrer : ${what}`,

  embeddingsUnrecognisable:
    'La passe des intervenants n’a pas produit d’empreintes vocales exploitables.',
  embeddingsNoDimensions: 'Ces empreintes vocales ne décrivent aucune dimension.',
  embeddingsTruncated: 'Ces empreintes vocales sont plus courtes qu’annoncé.',
  probeInvalid: 'L’analyse du média a renvoyé des métadonnées non valides.',
  cachePathInvalid: 'Le chemin du cache normalisé n’est pas valide.',
  normalizerNoOutput: 'La préparation du média n’a produit aucun fichier audio.',
  speakerPassNoAudio: 'La passe des intervenants n’a rien à écouter.',
  speakerPassTooMuchAudio: 'La passe des intervenants a prévu plus d’audio qu’il n’est possible.',
  recordingEmpty: 'L’enregistrement a été stocké sous forme de fichier vide.',
  editsLeaveNothing: 'Ces coupes ne laisseraient aucun enregistrement.',
  workingAudioUnreadable: 'L’audio de travail n’est pas un fichier WAV lisible.',
  workingAudioNotWav: 'L’audio de travail n’est pas un fichier WAV.',
  workingAudioSilent: 'L’audio de travail ne contient aucun son.',
  workingAudioFormatUnreadable: 'L’audio de travail a un format illisible.',
  workingAudioNoFormat: 'L’audio de travail ne décrit aucun format.',
  condensedAudioTooLarge: 'L’audio condensé est trop volumineux pour être écrit.',
  combinedPathInvalid: 'Le chemin de l’enregistrement réuni n’est pas valide.',
  modelUnknown: 'Ce modèle de transcription n’est pas reconnu.',
  downloadCancelled: 'Le téléchargement a été annulé.',
  downloadCorrupt: 'Le téléchargement était incomplet ou corrompu et a été écarté.',
  ollamaModelGone:
    'Le modèle Ollama sélectionné n’est plus installé. Choisissez-en un autre et réessayez.',
  ollamaModelChanged:
    'Le modèle Ollama sélectionné a changé après la mise en file de cette tâche. Réessayez pour le résoudre à nouveau.',
  ollamaRuntimeChanged:
    'L’environnement Ollama a changé après la mise en file de cette tâche. Réessayez pour le résoudre à nouveau.',
  responseTooLarge:
    'La réponse du modèle local a dépassé la limite de sécurité et n’a pas été retenue.',
  responseIncomplete: 'Le modèle local s’est arrêté avant d’avoir rendu un compte rendu complet.',
  protocolWouldNotFit: (sizes: string) => {
    const [expected, ceiling] = sizes.split('/').map((n) => Number(n).toLocaleString('fr-FR'));
    return `Cette réunion est assez longue pour qu’un compte rendu — environ ${expected} caractères — ne tienne pas dans une seule réponse, qui en contient à peu près ${ceiling}. Rien n’a été tenté : c’est une question d’arithmétique et non d’un mauvais tirage, et réessayer échouerait de la même façon. Choisissez un style plus bref, ou coupez l’enregistrement.`;
  },
  generationConfigUnreadable:
    'Cette tâche a été préparée par une version antérieure de LocaLog et ne peut pas être lue. Rien n’a été retenu et votre transcription est inchangée. Relancez la génération.',
  ollamaUnchecked: 'Ollama n’a pas encore été vérifié.',
  responseUnusable:
    'Le modèle local a rendu une réponse que LocaLog ne peut pas utiliser comme compte rendu. Rien n’a été retenu et votre transcription est inchangée. Réessayer aboutit souvent, car un modèle répond différemment à chaque fois.',
  recorderMissing:
    'Aucun enregistreur n’est installé. LocaLog en fournit un ; cette version ne le trouve pas.',
  recorderSilentAboutPermissions: 'L’enregistreur n’a pas indiqué ce qu’il est autorisé à faire.',
  recorderCannotReportPermissions:
    'Cet enregistreur ne sait pas indiquer ce qu’il est autorisé à faire.',
  runtimePathsMustBeAbsolute:
    'Choisissez des chemins absolus pour l’exécutable et le modèle whisper.cpp.',
  whisperExecutableMissing: 'L’exécutable whisper.cpp sélectionné est introuvable.',
  whisperModelMissing: 'Le modèle whisper.cpp sélectionné est introuvable.',
  embeddingsVersion: (version: string) =>
    `Ces empreintes vocales sont en version ${version}, que cette build ne sait pas lire.`,
  recordingTooSmall: (what: string) =>
    `L’enregistrement stocké est trop petit pour sa durée (${what}).`,
  workingAudioFormatWrong: (what: string) =>
    `La passe des intervenants a besoin d’un audio 16 kHz mono 16 bits, et celui-ci est ${what}.`,
  notEnoughSpace: (what: string) => `Espace insuffisant pour ce modèle (${what}).`,

  // Voir la note dans en.ts : des phrases que la partie Rust écrivait encore elle-même.
  settingInvalid: 'Ce réglage d’exécution ne peut pas être enregistré.',
  meetingTitleRequiredToRecord:
    'Donnez un titre à la réunion. Il n’y a aucun fichier dont le reprendre.',
  importSourceGone: 'Choisissez à nouveau le fichier d’origine avant de relancer cet import.',
  termProjectRequired: 'Choisissez le projet auquel ce terme appartient.',
  termAlreadyPresent: 'Ce terme figure déjà ici.',
  sourceRecordingRequired: 'Choisissez à nouveau l’enregistrement source.',
  managedPathInvalid: 'Le chemin vers ce fichier enregistré n’est pas valide.',
  documentChecksumFailed: 'Un document enregistré n’a pas passé son contrôle d’intégrité local.',
  transcriptOutputInvalid:
    'La transcription a produit quelque chose que LocaLog ne peut pas lire comme une transcription.',
  speakerCountOutOfRange: 'Le nombre d’intervenants attendu doit être compris entre 2 et 64.',
  sourceNotCommitted: 'Validez la source de la réunion avant de la transcrire.',
  providerNeededForGeneration:
    'Démarrez votre installation Ollama avant de générer un compte rendu.',
  exportDestinationInvalid: 'Choisissez une destination d’export valide.',
  exportFileExists:
    'Choisissez un nouveau nom de fichier. Un fichier existant n’est jamais écrasé sans être demandé.',
  exportFolderMissing: 'Le dossier d’export choisi n’est pas disponible.',
  processingBusy: 'Une autre tâche locale est déjà en cours. Attendez-la, ou annulez-la d’abord.',
  ffmpegMissingForRecording:
    'FFmpeg est nécessaire pour terminer un enregistrement et reste introuvable.',

  // La ligne Ollama dans les réglages. Voir la note dans en.ts.
  ollamaNotRunning: (detail: string) =>
    `Démarrez votre installation Ollama, puis actualisez.${detail ? ` ${detail}` : ''}`,
  ollamaModelsUnreadable: (detail: string) =>
    `Ollama fonctionne mais n’a pas indiqué quels modèles sont installés.${detail ? ` ${detail}` : ''}`,
  ollamaReadyNoModel:
    'Ollama est prêt. Choisissez un modèle installé pour générer des comptes rendus.',
  ollamaModelReady: 'Le modèle local choisi est prêt.',
  ollamaSelectedModelMissing:
    'Le modèle choisi n’est pas installé. Choisissez-en un autre, déjà installé.',
};

export const fr: Strings = {
  locale: 'fr-FR',

  failures,

  /** Voir la note dans en.ts : la clé est la valeur enregistrée. */
  meetingLanguages: {
    English: 'Anglais',
    German: 'Allemand',
    French: 'Français',
    Spanish: 'Espagnol',
    Italian: 'Italien',
    Dutch: 'Néerlandais',
    Portuguese: 'Portugais',
    Polish: 'Polonais',
    Danish: 'Danois',
    Swedish: 'Suédois',
    Norwegian: 'Norvégien',
    Finnish: 'Finnois',
    Czech: 'Tchèque',
    Turkish: 'Turc',
    Japanese: 'Japonais',
    Korean: 'Coréen',
    Chinese: 'Chinois',
    Arabic: 'Arabe',
    Ukrainian: 'Ukrainien',
  },
  dialog: {
    detectFromRecording: 'Détecter d’après l’enregistrement',
    chooseRecording: 'Choisir un enregistrement de réunion',
    audioAndVideo: 'Audio et vidéo',
    plainText: 'Texte brut',
    exportTitle: (title: string) => `Exporter ${title}`,
  },

  settings: {
    memoryReported: (gb: number) => `${gb} Go de mémoire détectés`,
    themeAutomatic: 'Automatique',
    themeLight: 'Clair',
    themeDark: 'Sombre',
    modelSelected: 'Sélectionné',
    useThisModel: 'Utiliser ce modèle',
    useModel: 'Utiliser le modèle',
    catalogueNote:
      'Le catalogue est volontairement restreint. LocaLog ne télécharge aucun modèle en douce et ne présente pas une place de marché. Une entrée ne devient sélectionnable qu’après vérification de son environnement, de sa licence, de sa consommation de mémoire et de sa qualité en allemand et en anglais.',
    managedCopiesNote:
      'LocaLog conserve ses propres copies des enregistrements importés, de l’audio préparé, des transcriptions, des comptes rendus et des modèles téléchargés dans son dossier de données. Les exports ne sont écrits qu’à l’endroit que vous choisissez.',
    discoveredRuntime: (path: string) => `Environnement détecté : ${path}`,
    runtimeVersion: (version: string) => `Version de l’environnement : ${version}`,
    evaluatedIn: (languages: string) => `Évalué en ${languages}`,
    evaluationPending: 'Évaluation de la qualité encore en attente',
    otherModelNote:
      'Ceci s’adresse à qui sait déjà quel modèle local essayer. Il n’est ni évalué ni recommandé par LocaLog, et reste soumis aux mêmes limites d’environnement et de mémoire.',
    qualityLead:
      'Choisissez la qualité voulue. LocaLog télécharge ce qu’il lui faut la première fois et le conserve sur cet appareil.',
    speakerDiscovery:
      'LocaLog détecte tout seul l’environnement de séparation des intervenants, parmi ses ressources ou sur le système. Il est facultatif et ne bloque jamais une transcription.',
    noSpeakerRuntime:
      'Aucun environnement compatible de séparation des intervenants n’a encore été trouvé sur cette machine.',
    readinessNote:
      'La vérification comprend un test de démarrage borné, afin qu’un exécutable incompatible ou défectueux ne soit pas présenté comme disponible.',
    restoreSummary: (name: string, projects: number, meetings: number, version: string) =>
      `${name} contient ${projects} projets et ${meetings} réunions, sauvegardés depuis LocaLog ${version}.`,
    restoreWarning:
      'Restaurer remplace les projets et réunions de cet espace de travail par ceux-là. Rien n’est supprimé — ce qui est ici est conservé dans un dossier à côté — mais LocaLog affichera le travail restauré, et il faudra quitter puis rouvrir l’application.',
    interfaceLanguage: 'Langue de l’interface',
    interfaceLanguageDetail:
      'La langue de LocaLog lui-même. Indépendante de la langue de chaque réunion.',
    application: 'Application',
    title: 'Réglages',
    lead: 'D’abord les choix professionnels. Les détails techniques restent repliés.',
    sectionsLabel: 'Sections des réglages',
    sectionGeneral: 'Général',
    sectionModels: 'Modèles',
    sectionTranscription: 'Transcription',
    sectionStorage: 'Stockage',
    sectionAppearance: 'Mise en forme',
    sectionAdvanced: 'Avancé',
    defaultExport: 'Export par défaut',
    defaultExportDetail:
      'Le format que l’éditeur propose en premier. Les autres restent à un clic.',
    defaultExportLabel: 'Format d’export par défaut',
    formatPdf: 'PDF',
    formatWord: 'Word',
    formatMarkdown: 'Markdown',
    formatPlainText: 'Texte brut',
    defaultForProtocols: 'Par défaut pour les comptes rendus',
    chooseOnce: 'Choisissez une fois, puis continuez',
    modelLead:
      'LocaLog utilise ce modèle pour les brouillons de comptes rendus locaux jusqu’à ce que vous en changiez. Le déroulement normal ne vous demande pas de choisir un modèle pour chaque réunion.',
    recommendedForMachine: 'Recommandé pour cette machine',
    notInstalledYet: 'Pas encore installé',
    baseline: 'Référence',
    european: 'Européen',
    checkInstalled: 'Vérifier les modèles installés',
    curatedModels: 'Modèles de compte rendu retenus',
    downloadModel: (size: string) => `Télécharger (${size})`,
    prepareSpeakerSeparation: 'Préparer la distinction des intervenants',
    restoredBackup: (projects: number, meetings: number, previous: string) =>
      `${projects} projets et ${meetings} réunions restaurés. Ce qui se trouvait ici a été déplacé vers ${previous} plutôt que supprimé. Quittez LocaLog et rouvrez-le pour travailler avec l’espace restauré.`,
    /** Voir la note dans en.ts. */
    transcriptionPreset: {
      fast: { name: 'Rapide', detail: 'Brouillons rapides, la plus légère en mémoire' },
      balanced: { name: 'Équilibrée', detail: 'Pour les réunions de tous les jours' },
      accurate: { name: 'Précise', detail: 'Meilleure qualité, la plus lente' },
    },
    downloadingPreset: (name: string) => `Téléchargement de ${name}`,
    /** Voir la note dans en.ts. */
    modelDescription: {
      'gemma4-12b':
        'Le plus exact et le plus régulier des modèles mesurés : sur trois exécutions, il a conservé 27 à 31 des 35 chiffres d’une réunion, là où le suivant est descendu à 6. Plus lent — environ quatorze minutes pour une réunion de quatre-vingts minutes.',
      'ministral-8b':
        'Mesuré sur une réunion en allemand à trois réglages ; il a rédigé un compte rendu utilisable à l’un d’eux : les autres ont produit une ébauche de deux lignes et un document JSON là où du markdown était demandé. Conservé comme candidat européen, pas encore une solution de rechange à la référence.',
      'qwen3.5-4b':
        'Le modèle mesuré le plus rapide — environ cinq minutes pour une réunion de quatre-vingts minutes — et le choix quand la mémoire est limitée. Il n’a jamais produit le tableau des prochaines étapes que le style formel demande.',
      'ministral-3b': 'Le premier candidat européen pour le Mac le moins puissant pris en charge.',
      'granite4.1-8b':
        'Mesuré sur une réunion en allemand à trois réglages, il a conservé 22, 19 puis 6 des 35 chiffres énoncés, à entrée identique. Une exécution qui perd cinq sixièmes de ce qui a été dit n’est pas un outil pour tenir un compte rendu : il n’est donc pas recommandé.',
      'llama-8b': 'Une place de comparaison réservée à une version de Llama vérifiée.',
    },
    modelOrigin: {
      international: 'Modèle ouvert international',
      european: 'Modèle européen',
    },
    modelLicence: {
      apache2: 'Apache 2.0',
      gemma: 'Conditions d’utilisation Gemma',
      modelSpecific: 'Propre au modèle',
    },
    modelLanguage: {
      de: 'allemand',
      en: 'anglais',
      ja: 'japonais',
      more: 'et bien d’autres',
    },
    modelStatus: {
      installed: 'Installé',
      notInstalled: 'Non installé',
      plannedCandidate: 'Candidat prévu',
    },
    modelSizeInstalled: (gb: string) => `environ ${gb} Go une fois installé`,
    modelSizeSmall: 'petit modèle embarqué',
    modelSizeLarger: 'modèle local plus grand',
    useAnotherModel: 'Utiliser un autre modèle installé',
    installedModel: 'Modèle installé',
    chooseInstalledModel: 'Choisir un modèle installé',
    useInstalledModel: 'Utiliser le modèle installé',
    conservativeBaseline: 'Référence prudente de 8 Go retenue',
    transcriptionQuality: 'Qualité de transcription',
    cancel: 'Annuler',
    ready: 'Prêt',
    remove: 'Supprimer',
    advancedDetails: 'Détails avancés',
    modelsStoredNote:
      'Les modèles sont conservés dans le dossier de données de LocaLog et vérifiés avant usage.',
    whisperExecutable: 'Exécutable whisper-cli',
    whisperExecutablePlaceholder: '/chemin/vers/whisper-cli',
    chooseFile: 'Choisir un fichier',
    whisperNote: 'Choisissez le binaire de transcription en ligne de commande, pas whisper-server.',
    saveRuntime: 'Enregistrer l’environnement',
    detected: (version: string) => `Détecté : ${version}`,
    chooseWhisper: 'Choisir l’exécutable whisper-cli',
    speakerDifferentiation: 'Distinction des intervenants',
    speakerLead:
      'La séparation des tours de parole indique qui a parlé quand. Elle est facultative, ne bloque jamais une transcription, et les noms restent modifiables pendant la relecture.',
    runtimeUnavailable: 'Environnement non disponible dans cette installation',
    optional: 'Facultatif',
    checkReadiness: 'Vérifier la disponibilité',
    downloadingSpeakerModels: 'Téléchargement des modèles de séparation des intervenants',
    speakerRuntimeMissing:
      'Les modèles sont prêts, mais cette installation n’a pas d’environnement compatible.',
    whereWorkIsKept: 'Où votre travail est conservé',
    workspaceNote:
      'LocaLog gère ce dossier pour que les chemins qu’il contient restent valides, mais il est à vous et vous pouvez y regarder quand vous voulez.',
    showInFinder: 'Afficher dans le Finder',
    backup: 'Sauvegarde',
    backupLead:
      'Tout reste sur cet appareil, ce qui veut aussi dire que tout part avec lui. Une sauvegarde est un dossier ordinaire, à mettre sur un disque ou là où vous gardez ce qui compte.',
    backUpNow: 'Sauvegarder maintenant',
    working: 'En cours…',
    backupContents:
      'Contient chaque projet, réunion, transcription et compte rendu, ainsi que les enregistrements eux-mêmes. Deux choses sont volontairement laissées de côté, car ce n’est pas votre travail et l’une comme l’autre se reconstruisent : les modèles téléchargés, et la copie préparée de chaque enregistrement. Mesuré sur un espace de travail réel, cet audio préparé représentait à lui seul les trois quarts de la sauvegarde.',
    restore: 'Restaurer',
    restoreLead:
      'Remet une sauvegarde en place. Elle est d’abord vérifiée en entier, et ce qui est ici est mis de côté plutôt que supprimé.',
    chooseBackup: 'Choisir une sauvegarde…',
    chooseBackupTitle: 'Choisir une sauvegarde LocaLog',
    whereToKeepBackup: 'Où conserver la sauvegarde',
    replaceWorkspace: 'Remplacer cet espace de travail',
    restoring: 'Restauration…',
    archived: 'Archivé',
    archivedLead:
      'Projets et réunions mis de côté. Rien n’a été supprimé : chaque réunion, transcription et compte rendu qu’ils contiennent est toujours là, et toujours dans chaque sauvegarde.',
    show: 'Afficher',
    hide: 'Masquer',
    nothingArchived: 'Rien n’a été archivé.',
    project: 'Projet',
    meeting: 'Réunion',
    bringBack: 'Restaurer',
    theme: 'Thème',
    themeFollowing: (theme: string) => `Suit ce Mac, réglé sur ${theme}.`,
    themeSetHere: 'Défini ici, quel que soit le réglage de ce Mac.',
    nextFakeJob: 'Prochaine tâche fictive',
    nextFakeJobDetail:
      'Commande réservée au développement, pour examiner les états d’échec et de reprise.',
    completeNormally: 'Se termine normalement',
    failOnce: 'Échoue une fois, puis autorise une reprise',
    syntheticNote: 'Cela ne concerne que l’environnement synthétique en mémoire.',
  },

  project: {
    deleteMeeting: (title: string) => `Supprimer ${title}`,
    deleteWarning:
      'Supprimer une réunion retire de cet appareil son enregistrement, sa transcription et toutes les versions de son compte rendu. C’est irréversible.',
    eyebrow: 'Projet',
    archiveProject: 'Archiver le projet',
    newMeeting: 'Nouvelle réunion',
    meetings: 'Réunions',
    newestFirst: 'Les plus récentes d’abord',
    columnDate: 'Date',
    columnMeeting: 'Réunion',
    columnDuration: 'Durée',
    columnStatus: 'État',
    archive: 'Archiver',
    delete: 'Supprimer',
    keep: 'Conserver',
    noMeetings: 'Aucune réunion pour l’instant',
    noMeetingsDetail: 'Importez le premier enregistrement pour commencer le suivi de ce projet.',
    importRecording: 'Importer un enregistrement',
  },

  lifecycle: {
    draft: 'Brouillon',
    sourceReady: 'Prêt à transcrire',
    transcriptReady: 'Transcription prête',
    protocolDraft: 'Brouillon de compte rendu',
    reviewed: 'Relu',
    archived: 'Archivé',
  },

  sections: {
    noHeadings: 'Ce compte rendu n’a pas encore de titres ; il n’y a donc rien à lister.',
    setAside: 'Mettre de côté',
    addSection: 'Ajouter une section',
    dragHint: 'Faites glisser, ou utilisez les flèches',
    setThisAside: 'Mettre cette section de côté',
    putThisBack: 'Remettre cette section',
    moveSection: (title: string) => `Déplacer ${title}. Utilisez les flèches.`,
    setAsideNamed: (title: string) => `Mettre ${title} de côté`,
    putBackNamed: (title: string) => `Remettre ${title}`,
    setAsideNote:
      'Une section mise de côté quitte le document : ce que vous lisez est donc exactement ce qui sera exporté. Elle est conservée ici et peut être remise.',
  },

  jobErrors: {
    interrupted: {
      title: 'L’importation a été interrompue',
      detail:
        'LocaLog s’est arrêté avant que la copie gérée ne soit validée. L’original externe est inchangé et vous pouvez réessayer sans risque.',
    },
    permission_denied: {
      title: 'LocaLog n’a pas pu lire ni stocker l’enregistrement',
      detail:
        'Vérifiez l’accès au fichier choisi et au dossier de données local de LocaLog, puis réessayez. L’original externe n’a pas été modifié.',
    },
    insufficient_space: {
      title: 'L’espace de stockage local est insuffisant',
      detail:
        'Libérez de la place et réessayez. Aucun enregistrement partiel n’a été présenté comme complet.',
    },
    source_missing: {
      title: 'L’enregistrement choisi n’est plus disponible',
      detail:
        'Remettez le fichier à son emplacement d’origine, ou créez une nouvelle importation. La réunion reste en brouillon, à l’abri.',
    },
    source_reselection_required: {
      title: 'Choisissez à nouveau l’enregistrement',
      detail:
        'Cette réunion a été créée par une version de développement antérieure qui ne retenait pas l’emplacement de la source. Choisissez de nouveau l’enregistrement pour continuer ; la réunion a été conservée.',
    },
    unsupported_media: {
      title: 'Ce type de média n’est pas encore pris en charge',
      detail:
        'Choisissez un fichier audio ou vidéo courant. L’original externe n’a pas été modifié.',
    },
    empty_source: {
      title: 'L’enregistrement choisi est vide',
      detail:
        'Choisissez un enregistrement contenant des données audio ou vidéo. Le fichier externe vide n’a pas été modifié.',
    },
    synthetic_failure: {
      title: 'L’adaptateur de développement s’est arrêté comme demandé',
      detail:
        'L’échec provoqué s’est produit avant qu’une version ne soit validée. Votre source et votre dernier état stable sont intacts, et vous pouvez réessayer.',
    },
    invalid_adapter_output: {
      title: 'La sortie locale n’a pas pu être validée',
      detail:
        'LocaLog n’a pas retenu ce résultat incomplet. Votre dernière source stable et les versions de vos documents sont intactes.',
    },
    runtime_missing: {
      title: 'Choisissez un environnement de transcription local',
      detail:
        'Sélectionnez un exécutable whisper.cpp installé dans Réglages → Transcription. LocaLog ne télécharge pas d’environnements.',
    },
    model_missing: {
      title: 'Choisissez un modèle de transcription local',
      detail:
        'Sélectionnez un modèle whisper.cpp déjà disponible dans Réglages → Transcription. Aucun modèle n’a été téléchargé ni modifié.',
    },
    runtime_changed: {
      title: 'L’environnement de transcription a changé',
      detail:
        'La tâche en file n’a pas été exécutée : son exécutable whisper.cpp ne correspond plus à l’environnement enregistré. Réessayez pour résoudre l’environnement actuel.',
    },
    model_changed: {
      title: 'Le modèle de transcription a changé',
      detail:
        'La tâche en file n’a pas été exécutée : son modèle ne correspond plus à l’empreinte enregistrée. Réessayez pour résoudre le modèle actuel.',
    },
    media_probe_failed: {
      title: 'L’enregistrement n’a pas pu être analysé',
      detail:
        'Vérifiez que FFprobe est installé et que la source importée est toujours lisible. L’original reste inchangé.',
    },
    normalization_failed: {
      title: 'L’enregistrement n’a pas pu être préparé',
      detail:
        'Vérifiez que FFmpeg est installé, puis réessayez. La copie préparée peut être régénérée et l’original reste inchangé.',
    },
    transcription_failed: {
      title: 'La transcription locale n’a pas pu aboutir',
      detail:
        'L’environnement whisper.cpp s’est arrêté avant qu’une version de la transcription ne soit validée. Vérifiez son modèle et réessayez.',
    },
    transcription_timeout: {
      title: 'La transcription locale a pris trop de temps',
      detail:
        'Le processus de transcription supervisé a été arrêté avant qu’une version ne soit validée. Vérifiez l’enregistrement et l’environnement, puis réessayez.',
    },
    provider_model_missing: {
      title: 'Le modèle local sélectionné n’est pas disponible',
      detail:
        'Le modèle Ollama sélectionné n’est plus installé. Choisissez un modèle installé dans Réglages → Génération du compte rendu, puis réessayez.',
    },
    provider_model_changed: {
      title: 'Le modèle local a changé',
      detail:
        'L’empreinte du modèle a changé après la mise en file de cette tâche. Réessayez pour prendre le modèle actuellement installé.',
    },
    provider_runtime_changed: {
      title: 'Le fournisseur local a changé',
      detail:
        'La version de l’environnement Ollama a changé après la mise en file de cette tâche. Réessayez pour prendre l’environnement actuel.',
    },
    provider_unavailable: {
      title: 'La génération locale du compte rendu n’a pas pu se connecter',
      detail:
        'Démarrez votre installation Ollama, puis réessayez. LocaLog ne démarre ni ne télécharge d’environnements.',
    },
    provider_invalid_output: {
      title: 'La sortie du modèle local n’a pas pu être validée',
      detail:
        'LocaLog n’a pas retenu ce compte rendu incomplet ou mal formé. Votre transcription est intacte et vous pouvez réessayer.',
    },
    provider_incomplete_output: {
      title: 'La sortie du modèle local n’a pas pu être validée',
      detail:
        'LocaLog n’a pas retenu ce compte rendu incomplet ou mal formé. Votre transcription est intacte et vous pouvez réessayer.',
    },
    provider_response_too_large: {
      title: 'La réponse du modèle local était trop volumineuse',
      detail:
        'La réponse a dépassé la limite de sécurité de LocaLog et n’a pas été retenue. Réessayez avec une transcription plus courte ou un autre modèle local.',
    },
    invalid_transcript_output: {
      title: 'La sortie de la transcription n’a pas pu être validée',
      detail:
        'LocaLog n’a pas retenu la sortie de l’environnement, incomplète ou mal formée. Votre source est intacte.',
    },
    processing_failed: {
      title: 'Le traitement local n’a pas pu aboutir',
      detail:
        'Aucune transcription ni aucun compte rendu incomplet n’a été présenté comme prêt. Votre dernier état stable reste disponible et vous pouvez réessayer.',
    },
    unknown: {
      title: 'L’importation n’a pas pu aboutir',
      detail:
        'La réunion reste en brouillon et l’original externe n’a pas été modifié. Vous pouvez réessayer sans risque.',
    },
  },

  jobStages: {
    transcriptSaved: 'Transcription enregistrée',
    protocolSaved: 'Compte rendu enregistré',
    importComplete: 'Importation terminée — original inchangé',
    processingCancelled: 'Traitement local annulé — état stable conservé',
    processingInterrupted: 'Traitement local interrompu — état stable conservé',
    processingFailed: 'Le traitement local n’a pas abouti — état stable conservé',

    ready_to_import: 'Prêt à récupérer l’enregistrement',
    copying: 'Récupération de l’enregistrement',
    stoppingSafely: 'Arrêt en cours, sans risque',
    temporary_complete: 'Presque terminé',
    finalizing: 'Mise à l’abri de l’enregistrement',
    duplicate_confirmation: 'Cet enregistrement est peut-être déjà ici',
    completed: 'L’enregistrement est arrivé',
    cancelled: 'Importation annulée — original inchangé',
    interrupted: 'Importation interrompue — original inchangé',
    failed: 'L’importation n’a pas abouti — original inchangé',
    probing_media: 'Examen de l’enregistrement',
    normalizing_audio: 'Préparation de l’audio',
    output_staged: 'Enregistrement sécurisé',

    transcription_queued: 'Prêt à transcrire',
    checking_source: 'Vérification de l’enregistrement',
    loading_transcription_model: 'Chargement du modèle',
    transcribing_audio: 'Transcription en cours',
    separating_speakers: 'Distinction des intervenants',
    validating_transcript: 'Enregistrement de la transcription',
    preparing_fake_transcriber: 'Préparation',
    transcribing_synthetic_segments: 'Création des passages de transcription',

    generation_queued: 'Prêt à rédiger le compte rendu',
    checking_transcript: 'Vérification de la transcription',
    resolving_protocol_inputs: 'Rassemblement du style et des termes',
    condensing_transcript: 'Lecture de la réunion',
    generating_protocol: 'Rédaction du brouillon',
    validating_protocol: 'Enregistrement du compte rendu',
    reading_introductions: 'Lecture des présentations',

    protocol_would_not_fit: 'Cette réunion est plus longue qu’une seule passe ne peut contenir',
    segments_no_subject_claimed: 'Une partie de la réunion n’entrait dans aucun sujet',
    sections_over_their_length: 'Certaines sections sont plus longues que prévu',

    finding_subjects: (detail: string) =>
      detail ? `Recherche des sujets abordés — passage ${detail}` : 'Recherche des sujets abordés',
    writing_section: (detail: string) =>
      detail ? `Rédaction de ${detail}` : 'Rédaction du compte rendu section par section',
    joining_subjects: (detail: string) =>
      detail
        ? `Regroupement des sujets proches — ${detail} trouvés`
        : 'Regroupement des sujets proches',
    joined_subjects: (detail: string) =>
      detail ? `Sujets regroupés — ${detail}` : 'Sujets regroupés',
    joining_failed: (detail: string) =>
      detail
        ? `Les sujets n’ont pas pu être regroupés — ${detail}`
        : 'Les sujets n’ont pas pu être regroupés',

    working: 'En cours',
  },

  stages: {
    label: 'Étapes de la réunion',
    source: 'Source',
    transcript: 'Transcription',
    protocol: 'Compte rendu',
  },

  progress: {
    needsAttention: 'Demande votre attention',
    backgroundWork: 'Travail en arrière-plan',
    cancellingSafely: 'Annulation en cours, sans risque…',
    cancel: 'Annuler',
    speakerPassNote:
      'Cette passe lit tout l’enregistrement pour comparer les tours de parole. Un long enregistrement peut demander quelques minutes ; vous pouvez annuler à tout moment sans risque.',
    latestRetained: 'Dernier état stable conservé',
    originalUnchanged: ' · original externe inchangé',
    retry: 'Réessayer',
    importing: 'Importation de l’enregistrement',
    transcribing: 'Transcription en cours',
    generating: 'Génération du compte rendu',
    separatingSpeakers: 'Séparation des intervenants',
    working: 'En cours…',
    duplicateNote: 'Ce contenu est déjà présent dans LocaLog. Rien n’a été fusionné ni écarté.',
    cancelImport: 'Annuler l’importation',
    importAnotherCopy: 'Importer une autre copie',
    chooseSourceAgain: 'Choisir de nouveau la source',
    continueImport: 'Poursuivre l’importation',
    transcribeAgain: 'Relancer la transcription',
    generateAgain: 'Relancer la génération',
  },

  newProject: {
    namesHeading: 'Noms et termes',
    namesLead:
      'Une transcription ne peut pas deviner un nom qu’elle n’a jamais entendu. Les donner maintenant est la minute la plus utile que vous puissiez consacrer à ce projet : un nom mal entendu se retrouve tel quel dans chaque compte rendu tiré de cet enregistrement, et aucune étape ultérieure ne le rattrape.',
    namesPeople: 'Personnes',
    namesPeopleHint: 'Celles qui seront sans doute présentes, ou citées en réunion.',
    namesOrganisations: 'Entreprises et maîtres d’ouvrage',
    namesOrganisationsHint: 'Le maître d’ouvrage, les autres intervenants, les fournisseurs.',
    namesProject: 'Ce projet',
    namesProjectHint: 'Le nom du projet, du terrain ou du bâtiment.',
    namesTerms: 'Tout ce qui mérite d’être bien orthographié',
    namesTermsHint:
      'Les mots propres à ce travail qu’une transcription générale ne connaîtrait pas.',
    namesNote:
      'Séparez-les par des virgules. Tout est facultatif et rien n’est définitif : vous pouvez compléter et corriger à tout moment dans Noms et termes, et une correction faite pendant la relecture d’une transcription y est également conservée.',
    creating: 'Création…',
    createAndContinue: 'Créer et continuer',
    afterCreated:
      'Le style de compte rendu, ainsi que les noms et termes de ce travail, peuvent être définis après la création du projet. Les noms valent bien une minute : ce sont eux qu’une transcription ne peut pas deviner.',
    eyebrow: 'Projets',
    title: 'Nouveau projet',
    lead: 'Créez le cadre professionnel auquel les réunions et les sources appartiennent.',
    defaults: 'Valeurs par défaut du projet',
    name: 'Nom du projet',
    namePlaceholder: 'p. ex. Étude salle des fêtes',
    description: 'Description',
    descriptionOptional: 'facultatif',
    descriptionPlaceholder: 'Une description interne concise',
    defaultLanguage: 'Langue de réunion par défaut',
    defaultLanguageDetail: 'Indépendante de la langue de l’interface.',
    cancel: 'Annuler',
  },

  appearance: {
    font: 'Police',
    appliesToProject: (project: string) =>
      `S’applique à tous les comptes rendus de ${project}, pour que les documents d’un cabinet se ressemblent. Cela change la mise en forme, jamais le contenu — cela, c’est le style ci-dessus.`,
    bodySize: 'Taille du texte',
    headingScale: 'Échelle des titres',
    lineSpacing: 'Interligne',
    pageWidth: 'Largeur de page',
  },

  record: {
    recordingNow: 'Enregistrement',
    recordThisMeeting: 'Enregistrer cette réunion',
    lead: 'La salle et l’appel sont captés sur des pistes séparées, sur cet appareil. Savoir si les personnes présentes ont donné leur accord vous revient ; LocaLog ne peut pas le savoir.',
    notRecording: 'Aucun enregistrement',
    microphone: 'Microphone',
    theCall: 'L’appel',
    trackRecording: 'Enregistrement',
    trackSilent: 'Silencieux jusqu’ici',
    trackListening: 'À l’écoute…',
    stopRecording: 'Arrêter l’enregistrement',
    finishing: 'Finalisation…',
    startRecording: 'Démarrer l’enregistrement',
    starting: 'Démarrage…',
    backToMeeting: 'Retour à la réunion',
    noRecorder: 'Cette version n’a pas d’enregistreur. Importez plutôt un fichier.',
    openTheSetting: 'Ouvrir le réglage',
    grantedInSettings: 'Accordé dans les Réglages Système, et pris en compte ici dès votre retour.',
    callWouldNotRecordTitle: 'L’appel ne serait pas enregistré.',
    callWouldNotRecordBody:
      'macOS n’a pas accordé à LocaLog l’enregistrement de l’écran et de l’audio système ; sans cela, l’enregistrement de l’appel est du silence plutôt qu’une erreur. Mieux vaut l’accorder maintenant que le découvrir après. Le microphone de la salle serait tout de même capté.',
    roomWouldNotRecordTitle: 'La salle ne serait pas enregistrée.',
    roomWouldNotRecordBody:
      'Le microphone a été refusé à LocaLog. L’appel serait tout de même capté si le réglage ci-dessus l’autorise.',
    recorderNotesTitle: 'L’enregistreur n’a pas pu faire tout ce qui lui était demandé.',
    stoppedOnItsOwn:
      'L’enregistreur s’est arrêté de lui-même. Ce qu’il avait capté jusque-là a été conservé.',
    quietCall: (seconds: number) =>
      `Rien n’est arrivé de l’appel depuis ${seconds} secondes. macOS donne du silence plutôt qu’une erreur à une application qui n’a pas l’autorisation d’enregistrer l’écran et l’audio système ; mieux vaut vérifier maintenant qu’après la réunion.`,
    quietMicrophone: (seconds: number) =>
      `Rien n’est arrivé du microphone depuis ${seconds} secondes. Vérifiez que la bonne entrée est sélectionnée et que rien d’autre ne l’occupe.`,
  },

  meeting: {
    browserPreview: 'Aperçu navigateur',
    speakersEstimateNote:
      'LocaLog regroupe les voix qu’il entend et les compte. C’est une estimation, que vous pouvez remplacer par un nombre si elle vous paraît fausse.',
    speakersCountNote:
      'Votre meilleure estimation suffit : c’est le nombre de voix que LocaLog cherchera. Trop, et une personne peut être coupée en deux ; trop peu, et deux personnes peuvent être confondues.',
    speakersTogetherNote: 'La transcription garde un seul nom d’intervenant.',
    importInterrupted:
      'LocaLog a été fermé avant que la copie gérée ne soit validée. La réunion reste en brouillon et l’importation peut être relancée sans risque.',
    importCancelled:
      'La copie gérée a été annulée. La réunion reste en brouillon et le fichier externe n’a pas été modifié.',
    importFailed:
      'La copie gérée n’a pas pu être validée. La réunion reste en brouillon et le fichier externe n’a pas été modifié.',
    importRunning:
      'LocaLog copie cette source dans son propre stockage. Elle ne sera prête qu’une fois la copie vérifiée et validée.',
    sourceStored:
      'est conservé en sécurité avec cette réunion. L’original externe n’a pas été modifié.',
    sourceSynthetic:
      'est associé à cette réunion de démonstration. Aucun fichier média réel n’a été copié.',
    syntheticFixture: 'Jeu de démonstration',
    eyebrow: 'Réunion',
    titleLabel: 'Titre de la réunion',
    editTitle: 'Modifier le titre de la réunion',
    languageLabel: 'Langue de la réunion',
    changeLanguage: 'Changer la langue de la réunion',
    save: 'Enregistrer',
    saveLanguage: 'Enregistrer la langue',
    cancel: 'Annuler',
    recordingEyebrow: 'Enregistrement',
    nothingRecorded: 'Rien d’enregistré pour l’instant',
    recordLead:
      'La salle et l’appel seront captés sur des pistes séparées, sur cet appareil. Vous pouvez arrêter dès que la réunion se termine.',
    recordThisMeeting: 'Enregistrer cette réunion',
    sourceImport: 'Importation de la source',
    originalUnchanged: 'Votre original reste inchangé',
    sourceReady: 'Source prête',
    readyToTranscribe: 'Prêt à transcrire',
    managedSource: 'Source gérée',
    language: 'Langue',
    languageHint: 'Réglage de la réunion · à modifier ci-dessus avant de transcrire',
    preset: 'Préréglage',
    globalDefault: 'Valeur par défaut',
    notSelected: 'Non sélectionné',
    peopleSpeaking: 'Personnes qui parlent',
    doNotSeparate: 'Ne pas distinguer les intervenants',
    separateAndCount: 'Les distinguer, et déterminer combien ils sont',
    prepareSpeakers: 'Préparer la séparation des intervenants',
    prepareSpeakersDetail:
      'LocaLog a besoin de deux fichiers de modèle locaux vérifiés avant de pouvoir ajouter des noms provisoires. Votre enregistrement reste sur cet appareil.',
    preparing: (percent: number) => `Préparation ${percent} %`,
    prepare: 'Préparer',
    prepareWithSize: (size: string) => `Préparer (${size})`,
    speakerRuntimeMissing:
      'L’environnement de séparation des intervenants n’est pas disponible dans cette installation. La transcription peut continuer, mais elle utilisera des noms génériques modifiables.',
    reviewAndTrim: 'Revoir et couper l’enregistrement d’abord',
    trimDetail:
      '— retirez l’attente avant le début et tout ce dont la réunion n’a pas besoin. Votre enregistrement n’est jamais modifié.',
    gettingReady: 'Préparation de la transcription…',
    useJobControls: 'Utilisez les commandes ci-dessus',
    prepareSpeakersFirst: 'Préparez d’abord la séparation des intervenants',
    transcribe: 'Transcrire',
    transcriptionFailedToStart: 'La transcription n’a pas pu démarrer. Réessayez.',
    transcriptReady: 'Transcription prête',
    reviewBeforeGeneration: 'À relire avant génération',
    transcriptReadyDetail:
      'La transcription horodatée est prête pour les corrections et l’attribution des intervenants.',
    reviewTranscript: 'Relire la transcription',
    protocolAvailable: 'Compte rendu disponible',
    continueInEditor: 'Continuer dans l’éditeur',
    protocolDetail:
      'La transcription reste disponible à côté de la version actuelle du compte rendu.',
    openProtocol: 'Ouvrir le compte rendu',
  },

  newMeeting: {
    meetingOverride: 'Réglage propre à cette réunion',
    preparing: 'Préparation…',
    bringingRecordingIn: 'Récupération de l’enregistrement…',
    noPerMeetingOverrides:
      'Les réglages propres à une réunion et le choix des noms et termes réunion par réunion ne sont pas encore disponibles.',
    chosenOnceNote:
      'La qualité de transcription et le modèle qui rédige le compte rendu se choisissent une fois, dans les Réglages, et servent pour chaque réunion.',
    titleRecording: 'Enregistrement',
    titleImport: 'Importation structurée',
    heading: 'Nouvelle réunion',
    leadRecording:
      'Nommez la réunion et choisissez son projet. L’enregistrement commence à l’écran suivant.',
    leadImport: 'Choisissez l’enregistrement, confirmez les détails, et LocaLog s’occupe du reste.',
    context: 'Cadre',
    chooseProject: 'Choisir un projet',
    project: 'Projet',
    newProject: 'Nouveau projet',
    noInbox:
      'Chaque source appartient à une réunion, et chaque réunion à un projet. Il n’y a pas de boîte de réception.',
    source: 'Source',
    importRecording: 'Importer un enregistrement',
    originalStays: 'Votre original reste où il est',
    readyToCopy: 'Prêt à être copié après confirmation de cette réunion',
    letGoToImport: 'Relâchez pour importer',
    originalStaysShort: 'L’original reste où il est.',
    dropHere: 'Déposez un enregistrement ici, ou cliquez pour en choisir un',
    dropDetail:
      'MP3, M4A, WAV, MP4, MOV et d’autres. L’original reste intact — LocaLog le copie dans son propre stockage.',
    readyToAssign: 'Prêt à être associé à cette réunion',
    chooseFile: 'Choisir un fichier audio ou vidéo',
    previewNote: 'L’aperçu navigateur montre le déroulement sans conserver le fichier.',
    useDemoRecording: 'Utiliser l’enregistrement de démonstration',
    essentials: 'L’essentiel',
    meetingInformation: 'Informations sur la réunion',
    title: 'Titre',
    titlePlaceholder: 'Repris du fichier s’il est laissé vide',
    date: 'Date',
    language: 'Langue de la réunion',
    protocolStyle: 'Style de compte rendu',
    projectDefault: 'Valeur par défaut du projet',
    qualityNote:
      'La qualité de transcription se choisit une fois dans les Réglages et vaut pour chaque réunion.',
    advanced: 'Options de traitement avancées',
    cancel: 'Annuler',
    createAndRecord: 'Créer la réunion et enregistrer',
    createAndImport: 'Créer la réunion et importer',
  },

  recordingReview: {
    lead: 'Coupez ce dont la réunion n’a pas besoin avant la transcription. Votre enregistrement n’est jamais modifié — tout est réversible.',
    noPreparedAudio:
      'Cette réunion n’a pas encore d’audio préparé à revoir. Il devient disponible une fois l’importation validée.',
    dragToSelect:
      'Faites glisser sur l’enregistrement pour sélectionner un passage, ou utilisez les flèches en maintenant Maj.',
    selectedRange: (from: string, to: string) => `Sélection de ${from} à ${to}.`,
    eyebrow: 'Enregistrement',
    heading: 'Revoir l’enregistrement',
    noAudio: 'Pas encore d’audio de travail',
    waveformLabel:
      'L’enregistrement. Déplacez-vous avec les flèches, maintenez Maj pour sélectionner.',
    keptOf: (kept: string, whole: string) => `${kept} sur ${whole} conservés`,
    startsAt: (time: string) => `Commence à ${time}`,
    endsAt: (time: string) => `Se termine à ${time}`,
    removedSpan: (from: string, to: string) => `Supprimé de ${from} à ${to}`,
    startHere: 'Commencer ici',
    removeSelection: 'Supprimer la sélection',
    endHere: 'Terminer ici',
    edits: 'Coupes',
    nothingRemoved: 'Rien de supprimé. Tout l’enregistrement sera transcrit.',
    undo: 'Annuler',
    putEverythingBack: 'Tout remettre',
    untouchedNote:
      'L’enregistrement lui-même est intact. Ce sont des instructions sur ce qu’il faut utiliser.',
    undoStartTrim: 'Annuler la coupe au début',
    undoEndTrim: 'Annuler la coupe à la fin',
    putStretchBack: 'Remettre ce passage',
    next: 'Suivant',
    continueToTranscription: 'Passer à la transcription',
    backToMeeting: 'Retour à la réunion',
  },

  transcript: {
    heardAs: (heard: string) => `Entendu comme « ${heard} »`,
    askAboutTheRest: 'Examiner le reste',
    askingAboutTheRest: 'Lecture des phrases…',
    askAboutTheRestNote:
      'Quelques mots sont mal entendus différemment à chaque fois ; corriger une orthographe ne les trouve donc pas. Ceci lit chacun d’eux dans sa propre phrase et propose un nom tiré de la liste de ce projet — il ne peut rien proposer d’autre, et ne change rien tant que vous ne l’avez pas dit.',
    proposedNothing: 'Rien de plus n’a été reconnu.',
    proposedNothingNote:
      'C’est la réponse habituelle, et une bonne : il ne peut proposer qu’un nom déjà présent dans ce projet, alors il se tait plutôt que d’en inventer un.',
    proposalsHeading: (count: number) => (count === 1 ? '1 proposition' : `${count} propositions`),
    proposalSuggests: (heard: string, suggested: string) => `${heard} → ${suggested}`,
    spellingsToCheck: (count: number) =>
      count === 1 ? '1 orthographe à vérifier' : `${count} orthographes à vérifier`,
    questionedByProtocol: 'le compte rendu n’a pas reconnu ce mot',
    autosaveFailed:
      'L’enregistrement automatique a échoué — votre dernier état enregistré est intact',
    correctCount: (count: number) => `Corriger ${count}`,
    audioCouldNotLoad: 'L’audio de travail de cette réunion n’a pas pu être chargé.',
    pauseAudio: 'Mettre en pause',
    playAudio: 'Lire',
    saving: 'Enregistrement…',
    editsSaved: 'Modifications enregistrées',
    revisionSaved: 'Version de la transcription enregistrée',
    separationUnavailableHere:
      'La séparation des intervenants n’est pas encore disponible dans cette installation. Vous pouvez continuer avec des noms saisis à la main.',
    rerunForSeparation:
      'Relancez cette transcription pour obtenir un résultat de séparation à jour.',
    separationUnavailableForRun:
      'La séparation des intervenants n’était pas disponible pour ce traitement. Vous pouvez continuer avec des noms saisis à la main.',
    nothingChangedYet: 'Rien de changé pour l’instant',
    readingOpening: 'Lecture du début…',
    readWhoIsHere: 'Lire qui participe à cette réunion',
    correcting: 'Correction…',
    durationPending: 'Durée à déterminer',
    introducedThemselves: (count: number) => `${count} se sont présentés`,
    noNamesYet: (project: string) => `Aucun nom pour l’instant dans ${project}`,
    speltAsHeard:
      'Orthographiés comme la transcription les a entendus. Corrigez ceux qui sont faux — ils seront corrigés ici et retenus pour ce projet.',
    openingNote:
      'Une réunion s’ouvre en général sur des gens qui disent qui ils sont. Lire ce passage donne à ce projet ses noms, c’est-à-dire ce qu’une transcription ne peut pas deviner.',
    foundInPlaces: (count: number) =>
      `Trouvé à ${count} ${count === 1 ? 'endroit' : 'endroits'}. Décochez ceux qui doivent rester tels quels.`,
    noneMisheardEveryTime: (count: number) =>
      `Aucun mot n’a été mal entendu à chacune de ses occurrences. ${count} passages restent signalés comme peu clairs pour d’autres raisons.`,
    nothingFlaggedNote:
      'Rien n’a été signalé comme peu clair. Une transcription faite avant cette fonction n’affiche rien non plus ici : mieux vaut donc relire une ancienne transcription que lui faire confiance.',
    workingAudioLater: 'L’audio de travail devient disponible une fois cette réunion transcrite.',
    recordingEndsNote:
      'Si la réunion s’est poursuivie au-delà, l’enregistrement ne l’a pas capté et le compte rendu ne le contiendra pas.',
    heading: 'Relecture de la transcription',
    exportTranscript: 'Exporter la transcription…',
    exportLabel: 'Exporter cette transcription',
    asMarkdown: 'En Markdown',
    asPlainText: 'En texte brut',
    reviewDetails: 'Détails de la relecture',
    sourceContext: 'Contexte de la source',
    seekAudio: 'Se déplacer dans l’audio',
    follow: 'Suivre',
    followLabel: 'Faire défiler la transcription jusqu’au passage en cours de lecture',
    searchTranscript: 'Rechercher dans la transcription',
    editableTranscript: 'Transcription modifiable',
    removeLine: 'Retirer cette ligne de la transcription',
    nothingFlagged: 'Rien signalé comme peu clair',
    show: 'Afficher',
    showing: 'Affichage',
    onePassage: '1 passage peu clair',
    manyPassages: (count: number) => `${count} passages peu clairs`,
    speakerHint:
      'Les noms d’intervenants sont un point de départ — renommez-les avec les personnes qui ont parlé.',
    generateProtocol: 'Générer le compte rendu',
    review: 'Relecture',
    detailsLabel: 'Détails de la relecture de transcription',
    closeInspector: 'Fermer le panneau',
    speakers: 'Intervenants',
    whereRecordingStops: 'Là où l’enregistrement s’arrête',
    transcriptionInput: 'Entrée de la transcription',
    language: 'Langue',
    meetingLanguage: 'Langue de la réunion',
    saveLanguage: 'Enregistrer la langue',
    cancel: 'Annuler',
    changeLanguage: 'Changer la langue',
    rerunNote:
      'À utiliser après avoir changé la langue ou les réglages de transcription. Le nouveau traitement est conservé comme une version distincte.',
    rerun: 'Relancer la transcription',
    rerunPreparing: 'Préparation d’une nouvelle transcription…',
    rerunConfirm: (language: string) =>
      `Relancer la transcription en ${language} ? La transcription actuelle restera jusqu’à ce que le nouveau résultat soit validé, puis cette transcription de travail sera remplacée.`,
    whoIsHere: 'Qui participe à cette réunion',
    close: 'Fermer',
    aboutAMinute: 'Environ une minute. Rien d’autre ne peut tourner pendant ce temps.',
    unsureNames: 'Noms à regarder de plus près',
    whatShouldItSay: 'Comment cela doit-il s’écrire ?',
    rememberForProject:
      'Retenir pour ce projet, pour que la prochaine réunion l’écrive correctement',
    areAnyNames:
      'Est-ce que ce sont des noms ? Une correction répare cette transcription et est retenue.',
    nothingToCheck: 'Rien à vérifier',
    correctSpelling: 'Corriger l’orthographe',
    checkWording: 'Vérifier la formulation',
    checkWords: (words: string) => `Vérifier ${words}`,
    textAt: (time: string) => `Texte de la transcription à ${time}`,
    jumpTo: (time: string) => `Aller à ${time}`,
    removeLineAt: (time: string) => `Supprimer la ligne à ${time}`,
    renameSpeaker: (speaker: string) => `Renommer ${speaker}`,
    nameHeardAs: (heard: string) => `Nom entendu comme ${heard}`,
    protocolStyle: 'Style de compte rendu',
    audioUnplayable: 'L’audio de travail de cette réunion n’a pas pu être lu.',
    speakersResolved:
      'Les tours de parole ont été attribués localement. Les noms sont provisoires — ne les remplacez que si vous connaissez la personne.',
    speakersFailed:
      'La séparation des intervenants n’a pas produit de tours de parole exploitables. La transcription est intacte et utilise des noms neutres ; vous pouvez continuer à la main.',
    speakersUnavailable:
      'La séparation des intervenants n’était pas disponible pour ce traitement. La transcription est intacte et utilise un nom neutre ; vous pouvez toujours le remplacer.',
    speakersUnknown:
      'Cette ancienne transcription n’indique pas si la séparation des intervenants a eu lieu. Ses noms neutres ne prouvent pas qu’il n’y avait qu’une seule personne.',
  },

  library: {
    remove: 'Supprimer',
    edit: 'Modifier',
    keep: 'Conserver',
    notInUseSuffix: ' · inutilisé',
    /** Voir la note dans en.ts : seulement tant qu’un style n’a pas été renommé. */
    shippedStyle: {
      'style-formal': {
        name: 'Compte rendu officiel',
        description: 'Compte rendu structuré des échanges, des décisions et des actions.',
      },
      'style-working-note': {
        name: 'Note de travail interne',
        description: 'Compte rendu de travail concis pour une équipe de projet interne.',
      },
      'style-decision-log': {
        name: 'Journal des décisions techniques',
        description:
          'Met en avant les solutions envisagées, les contraintes et les décisions explicites.',
      },
    },
    copyOf: (name: string) => `${name} (copie)`,
    enterATerm: 'Saisissez un terme.',
    reading: 'Lecture…',
    editTerm: 'Modifier le terme',
    inUse: 'Utilisé',
    notInUse: 'Inutilisé',
    instructionsGiven:
      'Voici les instructions données au modèle, dans l’ordre où elles lui sont données',
    asShipped: ', exactement telles que ce style a été livré',
    invariantsNote:
      'Elles ne font pas partie de ce style et ne peuvent pas être modifiées ici — elles ne sont pas enregistrées avec un style du tout. Elles sont ajoutées à chaque compte rendu au moment de sa rédaction, car un document qui rapporte une décision que personne n’a prise n’est pas un compte rendu d’un autre style : c’est un compte rendu faux.',
    whichTermsHelp:
      'Les noms, les entreprises et les sigles aident le plus. La terminologie professionnelle courante est en général transcrite correctement sans être listée.',
    termsLeadLong:
      'Ajoutez les noms, entreprises et sigles employés dans ce travail pour qu’ils soient transcrits correctement. Sur une vraie réunion de quatre-vingts minutes, cela a fait passer le nom du projet de jamais correct à toujours correct.',
    eyebrow: 'Bibliothèque',
    protocolStyles: 'Styles de compte rendu',
    namesAndTerms: 'Noms et termes',
    stylesLead:
      'Ce que dit un compte rendu, et dans quel ordre. Pas sa mise en forme — cela, c’est l’apparence, et elle vit dans l’éditeur à côté du document qu’elle décrit.',
    termsLead:
      'Les noms qu’une transcription ne peut pas deviner : votre projet, les entreprises, les personnes. Mesuré sur une vraie réunion, ils valent plus que tout autre réglage ici.',
    addTerm: 'Ajouter un terme',
    saveTerm: 'Enregistrer le terme',
    stylesUnreadable: 'Les styles ne peuvent pas être lus ici.',
    length: 'Longueur',
    name: 'Nom',
    description: 'Description',
    whatItAsksFor: 'Ce que ce style demande',
    addInstruction: 'Ajouter une instruction',
    removeInstruction: 'Retirer cette instruction',
    checkedOnProtocol: 'Vérifié sur le compte rendu terminé',
    alwaysEveryStyle: 'Toujours, dans chaque style',
    saveStyle: 'Enregistrer le style',
    cancel: 'Annuler',
    delete: 'Supprimer',
    editThisStyle: 'Modifier ce style',
    duplicate: 'Dupliquer',
    duplicateToEdit: 'Dupliquer pour modifier',
    shippedStyleNote:
      'Un style livré reste tel quel, pour qu’un compte rendu rédigé l’an dernier puisse l’être de la même façon aujourd’hui. Copiez-le pour faire le vôtre.',
    ownershipAutomatic: 'L’attribution est automatique.',
    termsScopeNote:
      'Les noms et termes d’un projet s’appliquent à ses réunions sans avoir à les choisir à chaque fois.',
    term: 'Terme',
    spellingAsShown: 'Orthographe telle qu’elle doit apparaître',
    category: 'Catégorie',
    appliesTo: 'S’applique à',
    everyProject: 'Tous les projets',
    unknownProject: 'Projet inconnu',
    noTerms: 'Pas encore de noms ni de termes',
    deleteThisTerm: 'Supprimer ce terme ?',
    densityFull: 'Prose complète',
    densityPlain: 'Énoncés simples',
    densityLine: 'Une ligne par point',
    densityFullMeaning: 'Prose complète. Un lecteur absent peut suivre la discussion.',
    densityPlainMeaning: 'Énoncés simples. Ce qui a été dit, sans le récit.',
    densityLineMeaning: 'Une ligne par point. Le relevé, et rien autour.',
    categoryPerson: 'Personne',
    categoryOrganisation: 'Entreprise',
    categoryProject: 'Projet',
    categoryAbbreviation: 'Sigle',
    categoryTechnicalTerm: 'Terme technique',
    categoryOther: 'Autre',
  },

  furniture: {
    header: 'En-tête',
    footer: 'Pied de page',
    left: 'Gauche',
    centre: 'Centre',
    insertInto: (where: string) => `Insérer une valeur dans ${where}`,
    right: 'Droite',
    insert: 'Insérer…',
    lineHint:
      'Écrivez la ligne telle qu’elle doit se lire, et placez-y une valeur là où vous en voulez une — « Page », le numéro, « sur 12 ». Une valeur est un seul objet : elle se sélectionne et se supprime d’un bloc.',
    appliesTo: (project: string) =>
      `S’applique à tous les comptes rendus de ${project}. Cela se répète sur la page imprimée et ne fait pas partie du document que vous modifiez.`,
  },

  shell: {
    breadcrumbMeeting: 'Réunion',
    breadcrumbRecording: 'Enregistrement',
    breadcrumbReview: 'Relecture',
    skipToWorkspace: 'Aller à l’espace de travail',
    workspace: 'Espace de travail',
    workspaceFailed: 'L’espace de travail n’a pas pu être ouvert',
    workspaceFailedDetail: 'Vos fichiers existants n’ont pas été modifiés.',
    tryAgain: 'Réessayer',
    preparingWorkspace: 'Préparation de l’espace de travail local…',
    openNavigation: 'Ouvrir la navigation',

    notSelected: 'Non sélectionné',

    jobNeedsDecision: 'Demande votre décision',
    jobReadyToContinue: 'Prêt à continuer',
    jobCancelling: 'Annulation en cours, sans risque',

    formatWordDocument: 'Document Word',
    formatPlainText: 'Texte brut',
    exportSaved: (format: string) => `Export ${format} enregistré`,
    exportFailed: (format: string, why: string) => `Échec de l’export ${format} : ${why}`,
    exportPrepared: (format: string) => `Export ${format} préparé`,
    exportNeedsDesktop: (format: string) => `L’export ${format} nécessite l’application de bureau.`,

    meetingArchived: 'Réunion archivée. Elle est dans Réglages, sous Stockage.',
    projectArchived: 'Projet archivé. Il est dans Réglages, sous Stockage.',
    transcriptExported: 'Transcription exportée',
  },

  protocol: {
    undo: 'Annuler',
    redo: 'Rétablir',
    next: 'Suivant',
    blockParagraph: 'Paragraphe',
    blockHeading1: 'Titre 1',
    blockHeading2: 'Titre 2',
    blockHeading3: 'Titre 3',
    figuresMissingFromRewrite: (count: number) =>
      `${count} chiffres énoncés dans le passage manquent dans cette reformulation`,
    markdownView: 'Vue Markdown',
    documentView: 'Vue document',
    looking: 'Recherche…',
    replaceAll: 'Tout remplacer',
    rewrite: 'Reformuler',
    rewriting: 'Reformulation',
    figureMissingFromRewrite: 'Un chiffre énoncé dans le passage manque dans cette reformulation',
    reviewedRevisionPreserved:
      'La version relue est conservée. Ces modifications de travail n’ont pas été relues.',
    thisRevisionReviewed: 'Cette version exacte et immuable a été marquée comme relue.',
    generatedStaysEditable: 'Le contenu généré reste relisible et modifiable.',
    notFound: 'Introuvable',
    matchCount: (count: number) => `${count} ${count === 1 ? 'occurrence' : 'occurrences'}`,
    replacedCount: (count: number) => ` · ${count} remplacées`,
    changesNotYetMade: (count: number) =>
      `${count} ${count === 1 ? 'modification' : 'modifications'}, pas encore appliquées`,
    compoundNote:
      'Un nom avec majuscule est aussi cherché à l’intérieur des mots composés, là où un simple remplacement le manque. Lisez-les, puis gardez-les ou laissez-les.',
    andMore: (count: number) => `et ${count} de plus, toutes des deux mêmes formes.`,
    passageGoesAlone:
      'Le passage part seul vers votre modèle local. Les nombres, les noms et les dates doivent revenir inchangés — vérifiez-les, et annulez si ce n’est pas le cas.',
    nothingChangedYet:
      'Rien n’a encore été modifié. Lisez, puis gardez ou laissez — un modèle local reformule bien et ne doit pas être cru sur parole.',
    secondPassNote:
      'Demandé à votre propre modèle, et il se trompe dans les deux sens : il rate des changements et signale des formulations correctes. À regarder, pas à croire.',
    pageEdgesNote:
      'Là où les pages se termineraient, mesuré comme la feuille de style d’impression les compose : un titre ou un tableau descend en entier plutôt que de se couper, la prose non. L’imprimante décide de la dernière ligne ou deux, alors prenez ceci à une ligne près plutôt qu’au trait.',
    transcriptSourceNote:
      'Rédigé à partir de la transcription relue de cette réunion. Rien n’enregistre quel passage a produit quelle phrase ; ce qui suit cherche donc les mots plutôt que de prétendre savoir — une paraphrase ne trouvera rien, ce qui est la réponse honnête.',
    noWordsTogether:
      'Ces mots n’apparaissent nulle part ensemble dans la transcription. Cela veut en général dire que le brouillon l’a dit avec ses propres mots, ce qu’il a le droit de faire — c’est dans l’enregistrement qu’il faut vérifier.',
    revisionNote:
      'Ce que vous tapez est conservé comme modifications de travail et ne crée pas de version. Une version est créée quand un brouillon est généré, quand vous en demandez une, quand vous marquez un compte rendu comme relu, et quand une version ancienne est restaurée — pour que cette liste reste lisible.',
    nothingRewrites:
      'Rien ici ne réécrit votre texte à votre place. Le brouillon est à vous, et chaque version est conservée.',
    figuresKept: (kept: number, stated: number) => `${kept} chiffres sur ${stated} conservés`,
    figuresNote: (stated: number, kept: number) =>
      `La réunion a énoncé ${stated} chiffres et ce brouillon en reprend ${kept}. Combien doivent y figurer dépend du style que vous avez choisi : c’est donc à regarder plutôt qu’une note.`,
    figuresInvented: (count: number) =>
      count === 1
        ? 'Un chiffre figure ici que la réunion n’a pas énoncé'
        : `${count} chiffres figurent ici que la réunion n’a pas énoncés`,
    confirmAgainstRecording: '. À confirmer sur l’enregistrement.',
    tasksUnowned: (count: number) =>
      count === 1
        ? 'Une tâche ici n’a personne en face'
        : `${count} tâches ici n’ont personne en face`,
    unownedNote:
      '. Le brouillon préfère laisser un responsable de côté plutôt que de le deviner ; c’est donc peut-être exactement ce que la réunion a décidé — et il est bien moins coûteux d’y mettre un nom maintenant qu’à la prochaine réunion.',
    editor: 'Éditeur de compte rendu',
    markdownBacked: 'fondé sur Markdown',
    noteMissingTableHeading: 'Pas de tableau des suites à donner',
    noteMissingTableBody:
      'Ce compte rendu a été rédigé trois fois et aucune des versions ne s’est terminée par un tableau des tâches convenues et de leurs responsables. Les actions décidées en réunion sont décrites dans les sections ci-dessus mais ne sont pas rassemblées ici.',
    noteGapsHeading: 'Non couvert par ce compte rendu',
    noteOneGap:
      'Un passage de l’enregistrement n’a pas pu être lu, et rien ci-dessus ne le décrit. L’enregistrement lui-même est complet et peut toujours être écouté.',
    noteSeveralGaps:
      'Plusieurs passages de l’enregistrement n’ont pas pu être lus, et rien ci-dessus ne les décrit. L’enregistrement lui-même est complet et ces passages peuvent toujours être écoutés.',
    documentType: 'Compte rendu',
    statusDraft: 'Brouillon',
    statusReviewed: 'Relu',
    statusChanged: 'Modifié depuis la relecture',
    fieldProjectName: 'Nom du projet',
    fieldMeetingTitle: 'Titre de la réunion',
    fieldMeetingDate: 'Date de la réunion',
    fieldDocumentType: 'Type de document',
    fieldProtocolStatus: 'État',
    fieldPageNumber: 'Numéro de page',
    fieldPageOfCount: 'Page n sur m',
    fieldText: 'Texte libre',
    showPageBreaks: 'Afficher les sauts de page',
    hidePageBreaks: 'Masquer les sauts de page',
    saving: 'Enregistrement…',
    autosaveFailed: 'Échec de l’enregistrement automatique',
    workingEditsSaved: 'Modifications de travail enregistrées',
    revisionSaved: 'Version enregistrée',
    editorTools: 'Outils',
    find: 'Rechercher',
    findInProtocol: 'Rechercher dans le compte rendu',
    replaceWith: 'Remplacer par',
    makeChanges: 'Appliquer ces modifications',
    leaveIt: 'Laisser',
    zoomOut: 'Réduire',
    zoomIn: 'Agrandir',
    insertTable: 'Insérer un tableau',
    insertDivider: 'Insérer un séparateur',
    documentMenu: 'Menu du document',
    clearFormatting: 'Effacer la mise en forme',
    table: 'Tableau',
    blockType: 'Type de bloc',
    addColumnLeft: 'Ajouter une colonne à gauche',
    addColumnRight: 'Ajouter une colonne à droite',
    deleteColumn: 'Supprimer cette colonne',
    addRowAbove: 'Ajouter une ligne au-dessus',
    addRowBelow: 'Ajouter une ligne en dessous',
    deleteRow: 'Supprimer cette ligne',
    formatting: 'Mise en forme',
    bold: 'Gras',
    italic: 'Italique',
    bulletedList: 'Liste à puces',
    numberedList: 'Liste numérotée',
    quotation: 'Citation',
    askModel: 'Demander au modèle de le dire autrement',
    customInstruction: 'Instruction personnalisée…',
    whatShouldChange: 'Que faut-il changer ?',
    proposedChange: 'Modification proposée',
    proposedReplacement: 'Remplacement proposé',
    proposedRewrite: 'Reformulation proposée',
    unchanged: 'Le modèle a rendu le passage inchangé.',
    factsMoved: 'Une seconde passe pense que ces faits ont bougé',
    noFactMoved: 'Une seconde passe n’a vu aucun fait bouger. Elle en rate.',
    useThis: 'Utiliser ceci',
    improveClarity: 'Clarifier',
    improveClarityInstruction: 'Rends ce passage plus clair à lire.',
    makeFormal: 'Rendre plus formel',
    makeFormalInstruction:
      'Adopte un registre plus formel, comme dans un compte rendu professionnel.',
    makePlainer: 'Rendre plus direct',
    makePlainerInstruction:
      'Rends la formulation plus simple et plus directe, sans perdre en précision.',
    shorten: 'Raccourcir',
    shortenInstruction: 'Dis ceci en moins de mots.',
    rewriteUnavailable: 'La reformulation n’est pas disponible ici.',
    replaceUnavailable: 'Le remplacement d’un nom n’est pas disponible ici.',
    nameNotFound: 'Ce nom ne figure pas dans ce compte rendu.',
    protocolMarkdown: 'Markdown du compte rendu',
    protocolLabel: 'Compte rendu',
    protocolDetails: 'Détails du compte rendu',
    documentDetails: 'Détails du document',
    closeInspector: 'Fermer le panneau',
    tabDocument: 'Document',
    tabTranscript: 'Transcription',
    tabHistory: 'Historique',
    status: 'État',
    createRevision: 'Créer une version',
    lineNumber: (line: number) => `Ligne ${line}`,
    pageNumber: (page: number) => `Page ${page}`,
    revisionNumber: (ordinal: number) => `Version ${ordinal}`,
    markReviewed: 'Marquer comme relu',
    style: 'Style',
    sections: 'Sections',
    newSection: 'Nouvelle section',
    appearance: 'Mise en forme',
    editAppearance: 'Modifier la mise en forme',
    headerFooter: 'En-tête et pied de page',
    editHeaderFooter: 'Modifier l’en-tête et le pied de page',
    nothingRepeated: 'Rien ne se répète sur la page',
    presets: 'Préréglages',
    useOrSavePreset: 'Utiliser ou enregistrer un préréglage',
    noneSaved: 'Aucun enregistré pour l’instant',
    savedCount: (count: number) => `${count} enregistrés`,
    use: 'Utiliser',
    remove: 'Retirer',
    nameThisPreset: 'Nommer ce préréglage',
    nameForPreset: 'Nom de ce préréglage',
    save: 'Enregistrer',
    cancel: 'Annuler',
    saveAsPreset: 'Enregistrer cette mise en forme et cet en-tête comme préréglage',
    export: 'Exporter',
    exportPdf: 'Exporter en PDF',
    exportWord: 'Exporter en Word',
    exportMarkdown: 'Exporter en Markdown',
    exportPlainText: 'Exporter en texte brut',
    exportNote:
      'Le PDF est imprimé depuis le document que vous lisez, composé comme ce projet compose ses comptes rendus — choisissez « Enregistrer au format PDF » dans la boîte de dialogue d’impression.',
    source: 'Source',
    findSelectedPassage: 'Retrouver le passage sélectionné',
    lookingFor: 'Recherche de :',
    openReviewedTranscript: 'Ouvrir la transcription relue',
    whatToCheck: 'Ce qu’il faut vérifier',
    revisions: 'Versions',
    current: 'Actuelle',
    restore: 'Restaurer',
  },

  sidebar: {
    projects: 'Projets',
    newProject: 'Nouveau projet',
    createProject: 'Créer le projet',
    library: 'Bibliothèque',
    protocolStyles: 'Styles de compte rendu',
    namesAndTerms: 'Noms et termes',
    settings: 'Réglages',
    recording: 'Enregistrement',
    primaryNavigation: 'Navigation principale',
    closeNavigation: 'Fermer la navigation',
    openNavigation: 'Ouvrir la navigation',
    themeFollowingSystem: 'Suit le thème du système. Passer à toujours clair.',
    themeAlwaysLight: 'Toujours clair. Passer à toujours sombre.',
    themeAlwaysDark: 'Toujours sombre. Revenir au thème du système.',
    themeFollowingShort: 'Suit le système',
    sidebarWidth: (width: number) => `${width} pixels`,
    resizeSidebar:
      'Redimensionner le panneau. Utilisez les flèches pour ajuster, ou Entrée pour réinitialiser.',
    themeAlwaysLightShort: 'Toujours clair',
    themeAlwaysDarkShort: 'Toujours sombre',

    importNeedsDecision: 'L’importation demande votre décision',
    needsAttention: 'Demande votre attention',
    importingRecording: 'Importation de l’enregistrement',
    transcribing: 'Transcription en cours',
    writingProtocol: 'Rédaction du compte rendu',
    working: 'En cours',
    workingEllipsis: 'En cours…',
    separatingSpeakers: 'Séparation des intervenants',
    openMeetingNeedingAttention: 'Ouvrir la réunion qui demande votre attention',
    openThisMeeting: 'Ouvrir cette réunion',
  },

  start: {
    eyebrow: 'IA locale pour des comptes rendus de réunion confidentiels',
    title: 'Commencer une réunion',
    lead: 'Importez un fichier audio ou vidéo. Vérifiez chaque étape avant qu’il ne devienne un compte rendu.',
    importTitle: 'Importer un enregistrement',
    importDetail: 'Choisissez un projet, et gardez tout dans son contexte',
    recordTitle: 'Enregistrer une réunion',
    recordDetail: 'Captez la salle et l’appel sur cet appareil, sur des pistes séparées',
    promiseTitle: 'Votre travail de réunion reste sur cet appareil.',
    promiseDetail: 'Pas de compte LocaLog, pas de service en ligne, pas de télémétrie.',

    setupProviderTitle: 'Encore une chose avant le premier compte rendu',
    setupProviderBody:
      'La transcription fonctionne déjà. Rédiger le compte rendu demande en plus un modèle de langue sur cet appareil, qui se configure dans les réglages. Vous pouvez importer et transcrire un enregistrement avant.',
    setupProviderAction: 'Configurer dans les réglages',
    setupTitle: 'Un téléchargement avant la première transcription',
    setupBody: (quality: string, size: string) =>
      `LocaLog transcrit sur cet appareil, il faut donc que le modèle y soit. La qualité ${quality} pèse ${size} et se télécharge une fois. Vous pouvez importer un enregistrement avant : c’est au démarrage de la transcription que le modèle est nécessaire, pas avant.`,
    setupDownload: (size: string) => `Télécharger maintenant (${size})`,
    setupCancel: 'Annuler',
    setupAside: 'Les autres qualités, et la séparation des intervenants, sont dans les Réglages.',
  },
};
