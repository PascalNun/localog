/**
 * Every word the application says, in Spanish.
 *
 * Typed against English, so this file cannot be missing a key or inventing one.
 *
 * ## Decisions taken once, here, so the whole application reads as one voice
 *
 * **Acta, never "protocolo".** The decision the whole file rests on. *Protocolo*
 * in Spanish is a procedure or a diplomatic form; the record a meeting produces
 * is an **acta**, and that is what a Spanish architecture or engineering office
 * actually circulates. Using *protocolo* would mark the interface as translated
 * by somebody who had not read it.
 *
 * **Usted, not tú.** Spanish software in Spain has drifted towards *tú*, but this
 * is written for offices keeping the formal record of meetings, it matches the
 * German *Sie* and the French *vous*, and it is the safer register across Latin
 * America as well.
 *
 * **Reunión for a meeting, grabación for a recording, pista for a track.** The
 * ordinary words, not calques.
 *
 * **Peninsular spelling and vocabulary** where the two diverge — *ordenador* is
 * avoided entirely rather than chosen, since nothing here needs the word.
 */

import type { Strings } from './en';

const failures = {
  missingProject: 'El proyecto seleccionado ya no existe.',
  missingMeeting: 'La reunión seleccionada ya no existe.',
  missingJob: 'La tarea de importación ya no está disponible.',
  importBusy: 'Ya se está importando otra grabación. Termínela o cancélela primero.',
  unsupportedSchema: (version: string) =>
    `Estos datos de LocaLog los creó una versión más reciente y no compatible (${version}).`,
  storageUnavailable: 'LocaLog no ha podido acceder a su espacio de trabajo local.',

  styleMissing: 'Ese estilo ya no existe.',
  styleNameRequired: 'Póngale un nombre al estilo.',
  styleNotSaved: 'No se ha podido guardar el estilo.',
  styleUnavailable: 'El estilo de acta seleccionado no está disponible.',
  styleUsedByMeeting: 'Hay una reunión que usa este estilo. Cámbiela primero.',
  styleUsedByProject: 'Un proyecto usa este estilo por omisión. Cámbielo primero.',

  presetNameRequired: 'Póngale un nombre al ajuste.',
  presetNotSaved: 'No se ha podido guardar el ajuste.',
  presetBuiltInUndeletable: 'Un ajuste que viene con LocaLog no se puede eliminar.',

  transcriptInvalid: 'La transcripción guardada no es válida.',
  transcriptSegmentMissing: 'Ese fragmento de la transcripción ya no existe.',
  transcriptTextRequired: 'Escriba un texto de transcripción válido.',
  transcriptNeedsSegment: 'Una transcripción necesita al menos un fragmento.',
  transcriptSpeakerRequired: 'Escriba un nombre de interviniente válido.',
  transcriptNotSaved: 'No se ha podido guardar la transcripción.',
  transcriptNotCommitted: 'No se ha podido confirmar la transcripción.',
  spellingRequired: 'Escriba una grafía válida.',

  protocolTextRequired: 'Escriba un texto de acta válido.',
  protocolRevisionMissing: 'La versión del acta seleccionada ya no existe.',
  protocolNeededBeforeExport: 'Genere un acta antes de exportarla.',
  protocolNeededBeforeSetAside: 'Genere un acta antes de apartar una sección.',
  sectionNotSetAside: 'No se ha podido apartar esa sección.',
  reviewBeforeGeneration: 'Revise la transcripción antes de generar.',
  vocabularyUnresolved: 'No se han podido resolver los nombres y términos.',

  selectionRequired: 'Seleccione el texto que quiere cambiar.',
  selectionTooLong:
    'Es demasiado texto para cambiarlo de una vez. Seleccione una sección en lugar del documento.',
  passageNotRewritten: 'No se ha podido reformular ese pasaje.',
  openingNotRead: 'No se ha podido leer el comienzo de la reunión.',
  providerNeededForPassage: 'Inicie su instalación de Ollama antes de reformular un pasaje.',
  providerNeededForOpening: 'Inicie su instalación de Ollama antes de leer las presentaciones.',
  providerNeededForCorrections: 'Inicie su instalación de Ollama antes de comprobar estas grafías.',
  providerModelRequired: 'Elija un modelo de Ollama instalado en Ajustes → Generación del acta.',

  styleNotMigrated: 'No se ha podido migrar un estilo.',
  termMissing: 'Ese término ya no existe.',
  exportFormatInvalid: 'Elija un formato de exportación válido.',
  meetingDateInvalid: 'Elija una fecha de reunión válida.',
  scopeInvalid: 'Elija un ámbito válido.',
  sourceFileInvalid: 'Elija un archivo de origen válido.',
  workspaceViewInvalid: 'Elija una vista del espacio de trabajo válida.',
  recordingUnreadable: 'No se ha podido leer esa grabación.',
  appearanceNotSaved: 'No se ha podido guardar la presentación.',
  furnitureNotSaved: 'No se han podido guardar el encabezado y el pie de página.',
  documentOperationFailed: 'No ha podido completarse la operación local sobre el documento.',
  providerConfigNotSaved: 'No se ha podido guardar la configuración del proveedor de actas.',
  runtimeConfigNotSaved: 'No se ha podido guardar la configuración del entorno de transcripción.',
  recorderNotStarted: 'No se ha podido iniciar el grabador.',
  tracksNotCombined: 'No se han podido unir las pistas de la grabación.',
  protocolInvalid: 'El acta guardada no es válida.',
  protocolNotUtf8: 'El acta guardada no está en UTF-8 válido.',
  editsNotRecorded: 'Esos cambios no se pueden registrar.',

  recordingAlreadyRunning: 'Ya se está grabando una reunión.',
  presetUnknown: 'Elija una calidad de transcripción conocida.',
  providerModelNotInstalled: 'Elija un modelo que ya esté instalado en Ollama.',
  diariserPathInvalid: 'Elija un programa de separación de intervinientes existente.',
  whisperPathInvalid: 'Elija un ejecutable de whisper.cpp existente.',
  nothingRecording: 'No se está grabando nada.',
  revealOnlyOnMac:
    'Abrir la carpeta solo está implementado en macOS. La ruta de arriba es correcta.',
  privacySettingsOnlyOnMac: 'Abrir los ajustes de privacidad solo está implementado en macOS.',
  providerNeededForModel: 'Inicie su instalación de Ollama antes de elegir un modelo.',
  settingsNotOpened: 'No se han podido abrir los Ajustes del Sistema.',
  presetMissing: 'Esa plantilla de exportación ya no está disponible.',
  downloadStopped: 'La descarga se ha interrumpido.',
  coordinatorUnavailable: 'El coordinador de importación no está disponible.',
  taskStopped: 'La tarea local de cancelación se ha interrumpido.',
  recorderPermissionsUnknown: 'No se ha podido preguntar al grabador por sus permisos.',
  recorderStateUnknown: 'El grabador está en un estado desconocido. Reinicie LocaLog.',
  recordingNotFinished: 'No se ha podido finalizar la grabación.',
  replacementNotPrepared: 'No se ha podido preparar el reemplazo.',
  workspaceNotOpened: 'No se ha podido abrir la carpeta del espacio de trabajo.',
  settingsPaneUnknown: 'No existe ese panel de ajustes.',
  meetingBusy: 'Esta reunión aún se está procesando. Cancélelo primero.',
  printDialogUnavailable: 'Esta ventana no ha podido abrir el diálogo de impresión.',

  backupNameUnsafe: 'Ese nombre de copia no se puede usar como nombre de carpeta.',
  notABackup: 'Esa carpeta no es una copia de LocaLog: no tiene manifest.json.',
  backupPathOutside: (path: string) =>
    `Esta copia menciona un archivo fuera de su propia carpeta (${path}), así que no se ha restaurado.`,
  backupFormatUnknown: (format: string) =>
    `Esta copia se escribió en el formato ${format}, que esta versión de LocaLog no sabe leer. Una versión posterior sí sabrá.`,
  backupDamaged: (what: string) =>
    `Esta copia está incompleta o dañada (${what}), así que no se ha cambiado nada. Su trabajo actual está intacto.`,
  backupNameTaken: (name: string) => `Ya hay algo llamado «${name}» en esa carpeta.`,
  backupIoFailed: (what: string) => `No se ha podido escribir ni leer la copia: ${what}`,
  backupDatabaseFailed: (what: string) => `No se ha podido copiar la base de datos: ${what}`,

  categoryRequired: 'Elija una categoría.',
  meetingLanguageRequired: 'Elija un idioma de reunión.',
  meetingLanguageInvalid: 'Elija un idioma de reunión válido.',
  meetingInvalid: 'Elija una reunión válida.',
  projectInvalid: 'Elija un proyecto válido.',
  styleInvalid: 'Elija un estilo de acta válido.',
  sourceRecordingInvalid: 'Elija una grabación de origen válida.',
  meetingTitleRequired: 'Escriba un título de reunión.',
  projectNameRequired: 'Escriba un nombre de proyecto.',
  termRequired: 'Escriba un término.',
  meetingTitleTooLong: 'El título de la reunión es demasiado largo.',
  speakerPassCannotRead: (what: string) =>
    `La pasada de intervinientes no ha podido leer el audio de trabajo: ${what}`,
  speakerPassCannotWrite: (what: string) =>
    `La pasada de intervinientes no ha podido escribir su audio: ${what}`,
  recordingNotStored: (what: string) => `No se ha podido almacenar la grabación: ${what}`,
  recordingNotRead: (what: string) => `No se ha podido leer la grabación: ${what}`,
  modelNotDownloaded: (what: string) => `No se ha podido descargar el modelo: ${what}`,
  modelNotSaved: (what: string) => `No se ha podido guardar el modelo: ${what}`,
  ollamaRequestFailed: (what: string) =>
    `Ollama no ha podido completar la solicitud local: ${what}`,
  recorderStartFailed: (what: string) => `No se ha podido iniciar el grabador: ${what}`,

  embeddingsUnrecognisable:
    'La pasada de intervinientes no ha producido huellas de voz reconocibles.',
  embeddingsNoDimensions: 'Esas huellas de voz no describen ninguna dimensión.',
  embeddingsTruncated: 'Esas huellas de voz son más cortas de lo que declaran.',
  probeInvalid: 'El análisis del medio ha devuelto metadatos no válidos.',
  cachePathInvalid: 'La ruta de la caché normalizada no es válida.',
  normalizerNoOutput: 'La preparación del medio no ha producido ningún archivo de audio.',
  speakerPassNoAudio: 'La pasada de intervinientes no tiene nada que escuchar.',
  speakerPassTooMuchAudio:
    'La pasada de intervinientes ha previsto más audio del que se puede manejar.',
  recordingEmpty: 'La grabación se ha almacenado como un archivo vacío.',
  editsLeaveNothing: 'Estos cortes no dejarían grabación alguna.',
  workingAudioUnreadable: 'El audio de trabajo no es un archivo WAV legible.',
  workingAudioNotWav: 'El audio de trabajo no es un archivo WAV.',
  workingAudioSilent: 'El audio de trabajo no contiene sonido.',
  workingAudioFormatUnreadable: 'El audio de trabajo tiene un formato ilegible.',
  workingAudioNoFormat: 'El audio de trabajo no describe ningún formato.',
  condensedAudioTooLarge: 'El audio condensado es demasiado grande para escribirlo.',
  combinedPathInvalid: 'La ruta de la grabación unida no es válida.',
  modelUnknown: 'Ese modelo de transcripción no se reconoce.',
  downloadCancelled: 'Se ha cancelado la descarga.',
  downloadCorrupt: 'La descarga estaba incompleta o dañada y se ha descartado.',
  ollamaModelGone:
    'El modelo de Ollama seleccionado ya no está instalado. Elija otro y vuelva a intentarlo.',
  ollamaModelChanged:
    'El modelo de Ollama seleccionado ha cambiado después de encolar esta tarea. Vuelva a intentarlo para resolverlo de nuevo.',
  ollamaRuntimeChanged:
    'El entorno de Ollama ha cambiado después de encolar esta tarea. Vuelva a intentarlo para resolverlo de nuevo.',
  responseTooLarge:
    'La respuesta del modelo local ha superado el límite de seguridad y no se ha guardado.',
  responseIncomplete: 'El modelo local se ha detenido antes de devolver un acta completa.',
  protocolWouldNotFit: (sizes: string) => {
    const [expected, ceiling] = sizes.split('/').map((n) => Number(n).toLocaleString('es-ES'));
    return `Esta reunión es lo bastante larga como para que un acta suya —unos ${expected} caracteres— no quepa en una sola respuesta, en la que caben unos ${ceiling}. No se ha intentado nada, porque esto es aritmética y no una mala tirada: repetirlo fallaría igual. Elija un estilo de acta más breve, o divida la grabación.`;
  },
  generationConfigUnreadable:
    'Esta tarea la preparó una versión anterior de LocaLog y no se puede leer. No se ha guardado nada y su transcripción está intacta. Vuelva a iniciar la generación.',
  ollamaUnchecked: 'Todavía no se ha comprobado Ollama.',
  responseUnusable:
    'El modelo local ha devuelto una respuesta que LocaLog no puede usar como acta. No se ha guardado nada y su transcripción está intacta. Reintentarlo suele funcionar, porque un modelo responde distinto cada vez.',
  recorderMissing:
    'No hay ningún grabador instalado. LocaLog incluye uno; esta versión no lo encuentra.',
  recorderSilentAboutPermissions: 'El grabador no ha dicho qué tiene permitido hacer.',
  recorderCannotReportPermissions: 'Este grabador no sabe decir qué tiene permitido hacer.',
  runtimePathsMustBeAbsolute:
    'Elija rutas absolutas para el ejecutable y el modelo de whisper.cpp.',
  whisperExecutableMissing: 'No se ha encontrado el ejecutable de whisper.cpp seleccionado.',
  whisperModelMissing: 'No se ha encontrado el modelo de whisper.cpp seleccionado.',
  embeddingsVersion: (version: string) =>
    `Estas huellas de voz son de la versión ${version}, que esta compilación no lee.`,
  recordingTooSmall: (what: string) =>
    `La grabación almacenada es demasiado pequeña para su duración (${what}).`,
  workingAudioFormatWrong: (what: string) =>
    `La pasada de intervinientes necesita audio de 16 kHz mono de 16 bits, y este es ${what}.`,
  notEnoughSpace: (what: string) => `No hay espacio suficiente para este modelo (${what}).`,

  // Véase la nota en en.ts: frases que la parte Rust seguía escribiendo por su cuenta.
  settingInvalid: 'Ese ajuste de ejecución no se puede guardar.',
  meetingTitleRequiredToRecord: 'Dé un título a la reunión. No hay ningún archivo del que tomarlo.',
  importSourceGone: 'Elija de nuevo el archivo original antes de reintentar esta importación.',
  termProjectRequired: 'Elija el proyecto al que pertenece este término.',
  termAlreadyPresent: 'Ese término ya figura aquí.',
  sourceRecordingRequired: 'Elija de nuevo la grabación de origen.',
  managedPathInvalid: 'La ruta a ese archivo guardado no es válida.',
  documentChecksumFailed:
    'Un documento guardado no ha superado su comprobación local de integridad.',
  transcriptOutputInvalid:
    'La transcripción ha producido algo que LocaLog no puede leer como transcripción.',
  speakerCountOutOfRange: 'El número previsto de intervinientes debe estar entre 2 y 64.',
  sourceNotCommitted: 'Confirme el origen de la reunión antes de transcribirla.',
  providerNeededForGeneration: 'Inicie su instalación de Ollama antes de generar un acta.',
  exportDestinationInvalid: 'Elija un destino de exportación válido.',
  exportFileExists:
    'Elija un nombre de archivo nuevo. Un archivo existente nunca se sobrescribe sin preguntar.',
  exportFolderMissing: 'La carpeta de exportación elegida no está disponible.',
  processingBusy: 'Ya hay otra tarea local en curso. Espere a que termine, o cancélela primero.',
  ffmpegMissingForRecording:
    'FFmpeg hace falta para finalizar una grabación y no se ha encontrado.',

  // La fila de Ollama en los ajustes. Véase la nota en en.ts.
  ollamaNotRunning: (detail: string) =>
    `Inicie su instalación de Ollama y luego actualice.${detail ? ` ${detail}` : ''}`,
  ollamaModelsUnreadable: (detail: string) =>
    `Ollama está en marcha pero no ha dicho qué modelos hay instalados.${detail ? ` ${detail}` : ''}`,
  ollamaReadyNoModel: 'Ollama está listo. Elija un modelo instalado para generar actas.',
  ollamaModelReady: 'El modelo local elegido está listo.',
  ollamaSelectedModelMissing: 'El modelo elegido no está instalado. Elija otro que ya lo esté.',
};

export const es: Strings = {
  locale: 'es-ES',

  failures,

  /** Véase la nota en en.ts: la clave es el valor guardado. */
  meetingLanguages: {
    English: 'Inglés',
    German: 'Alemán',
    French: 'Francés',
    Spanish: 'Español',
    Italian: 'Italiano',
    Dutch: 'Neerlandés',
    Portuguese: 'Portugués',
    Polish: 'Polaco',
    Danish: 'Danés',
    Swedish: 'Sueco',
    Norwegian: 'Noruego',
    Finnish: 'Finés',
    Czech: 'Checo',
    Turkish: 'Turco',
    Japanese: 'Japonés',
    Korean: 'Coreano',
    Chinese: 'Chino',
    Arabic: 'Árabe',
    Ukrainian: 'Ucraniano',
  },
  dialog: {
    detectFromRecording: 'Detectar a partir de la grabación',
    chooseRecording: 'Elegir una grabación de reunión',
    audioAndVideo: 'Audio y vídeo',
    plainText: 'Texto sin formato',
    exportTitle: (title: string) => `Exportar ${title}`,
  },

  settings: {
    memoryReported: (gb: number) => `${gb} GB de memoria detectados`,
    themeAutomatic: 'Automático',
    themeLight: 'Claro',
    themeDark: 'Oscuro',
    modelSelected: 'Seleccionado',
    useThisModel: 'Usar este modelo',
    useModel: 'Usar el modelo',
    catalogueNote:
      'El catálogo está deliberadamente acotado. LocaLog no descarga modelos en silencio ni presenta un mercado de modelos. Una entrada solo pasa a ser seleccionable después de comprobar su entorno, su licencia, su consumo de memoria y su calidad en alemán e inglés.',
    managedCopiesNote:
      'LocaLog guarda copias propias de las grabaciones importadas, del audio preparado, de las transcripciones, de las actas y de los modelos descargados en su carpeta de datos. Las exportaciones solo se escriben en el lugar que usted elija.',
    discoveredRuntime: (path: string) => `Entorno detectado: ${path}`,
    runtimeVersion: (version: string) => `Versión del entorno: ${version}`,
    evaluatedIn: (languages: string) => `Evaluado en ${languages}`,
    evaluationPending: 'Evaluación de calidad todavía pendiente',
    otherModelNote:
      'Esto es para quien ya sabe qué modelo local quiere probar. LocaLog no lo evalúa ni lo recomienda, y sigue sujeto a los mismos límites de entorno y memoria.',
    qualityLead:
      'Elija la calidad que quiera. LocaLog descarga lo que necesita la primera vez y lo conserva en este dispositivo.',
    speakerDiscovery:
      'LocaLog detecta por su cuenta el entorno de separación de intervinientes, entre sus propios recursos o en el sistema. Es opcional y nunca bloquea una transcripción.',
    noSpeakerRuntime:
      'Todavía no se ha encontrado en esta máquina ningún entorno compatible de separación de intervinientes.',
    readinessNote:
      'La comprobación incluye una prueba de arranque acotada, para que un ejecutable incompatible o defectuoso no se presente como disponible.',
    restoreSummary: (name: string, projects: number, meetings: number, version: string) =>
      `${name} contiene ${projects} proyectos y ${meetings} reuniones, copiados desde LocaLog ${version}.`,
    restoreWarning:
      'Restaurar sustituye los proyectos y reuniones de este espacio de trabajo por esos. No se elimina nada —lo que hay aquí se conserva en una carpeta al lado—, pero LocaLog mostrará el trabajo restaurado y tendrá que cerrarlo y volver a abrirlo.',
    interfaceLanguage: 'Idioma de la interfaz',
    interfaceLanguageDetail:
      'El idioma de LocaLog en sí. Independiente del idioma de cada reunión.',
    application: 'Aplicación',
    title: 'Ajustes',
    lead: 'Primero lo profesional. Los detalles técnicos siguen plegados.',
    sectionsLabel: 'Secciones de los ajustes',
    sectionGeneral: 'General',
    sectionModels: 'Modelos',
    sectionTranscription: 'Transcripción',
    sectionStorage: 'Almacenamiento',
    sectionAppearance: 'Presentación',
    sectionAdvanced: 'Avanzado',
    defaultExport: 'Exportación por omisión',
    defaultExportDetail: 'Qué formato ofrece primero el editor. Los demás quedan a un clic.',
    defaultExportLabel: 'Formato de exportación por omisión',
    formatPdf: 'PDF',
    formatWord: 'Word',
    formatMarkdown: 'Markdown',
    formatPlainText: 'Texto sin formato',
    defaultForProtocols: 'Por omisión para las actas',
    chooseOnce: 'Elija una vez y siga trabajando',
    modelLead:
      'LocaLog usa este modelo para los borradores locales de actas hasta que usted lo cambie. El flujo normal no le pide elegir un modelo en cada reunión.',
    recommendedForMachine: 'Recomendado para esta máquina',
    notInstalledYet: 'Todavía no instalado',
    baseline: 'Referencia',
    european: 'Europeo',
    checkInstalled: 'Comprobar los modelos instalados',
    curatedModels: 'Modelos de acta seleccionados',
    downloadModel: (size: string) => `Descargar (${size})`,
    prepareSpeakerSeparation: 'Preparar la separación de intervinientes',
    restoredBackup: (projects: number, meetings: number, previous: string) =>
      `Se han restaurado ${projects} proyectos y ${meetings} reuniones. Lo que había aquí se ha movido a ${previous} en vez de borrarse. Salga de LocaLog y vuelva a abrirlo para trabajar con el espacio restaurado.`,
    /** Véase la nota en en.ts. */
    transcriptionPreset: {
      fast: { name: 'Rápida', detail: 'Borradores rápidos, la más ligera en memoria' },
      balanced: { name: 'Equilibrada', detail: 'Para las reuniones del día a día' },
      accurate: { name: 'Precisa', detail: 'La mejor calidad, la más lenta' },
    },
    downloadingPreset: (name: string) => `Descargando ${name}`,
    /** Véase la nota en en.ts. */
    modelDescription: {
      'gemma4-12b':
        'El más exacto y el más constante de los modelos medidos: en tres ejecuciones conservó de 27 a 31 de las 35 cifras de una reunión, donde el siguiente bajó hasta 6. Más lento: unos catorce minutos para una reunión de ochenta minutos.',
      'ministral-8b':
        'Medido en una reunión en alemán con tres ajustes, redactó un acta utilizable en uno de ellos: los demás produjeron un esbozo de dos líneas y un documento JSON donde se pedía markdown. Se mantiene como candidato europeo, todavía no como alternativa a la referencia.',
      'qwen3.5-4b':
        'El modelo medido más rápido, unos cinco minutos para una reunión de ochenta minutos, y la elección cuando la memoria escasea. Nunca produjo la tabla de próximos pasos que pide el estilo formal.',
      'ministral-3b': 'El primer candidato europeo para el Mac menos potente que se admite.',
      'granite4.1-8b':
        'Medido en una reunión en alemán con tres ajustes, conservó 22, 19 y 6 de las 35 cifras enunciadas, con la misma entrada. Una ejecución que pierde cinco sextos de lo dicho no sirve para dejar constancia, así que no se recomienda.',
      'llama-8b': 'Un puesto de comparación reservado para una versión de Llama verificada.',
    },
    modelOrigin: {
      international: 'Modelo abierto internacional',
      european: 'Modelo europeo',
    },
    modelLicence: {
      apache2: 'Apache 2.0',
      gemma: 'Condiciones de uso de Gemma',
      modelSpecific: 'Según el modelo',
    },
    modelLanguage: {
      de: 'alemán',
      en: 'inglés',
      ja: 'japonés',
      more: 'y muchos más',
    },
    modelStatus: {
      installed: 'Instalado',
      notInstalled: 'No instalado',
      plannedCandidate: 'Candidato previsto',
    },
    modelSizeInstalled: (gb: string) => `unos ${gb} GB instalado`,
    modelSizeSmall: 'modelo pequeño para el equipo',
    modelSizeLarger: 'modelo local más grande',
    useAnotherModel: 'Usar otro modelo instalado',
    installedModel: 'Modelo instalado',
    chooseInstalledModel: 'Elegir un modelo instalado',
    useInstalledModel: 'Usar el modelo instalado',
    conservativeBaseline: 'Usando la referencia prudente de 8 GB',
    transcriptionQuality: 'Calidad de la transcripción',
    cancel: 'Cancelar',
    ready: 'Listo',
    remove: 'Eliminar',
    advancedDetails: 'Detalles avanzados',
    modelsStoredNote:
      'Los modelos se guardan en la carpeta de datos de LocaLog y se verifican antes de usarse.',
    whisperExecutable: 'Ejecutable whisper-cli',
    whisperExecutablePlaceholder: '/ruta/a/whisper-cli',
    chooseFile: 'Elegir archivo',
    whisperNote: 'Elija el binario de transcripción de línea de órdenes, no whisper-server.',
    saveRuntime: 'Guardar el entorno',
    detected: (version: string) => `Detectado: ${version}`,
    chooseWhisper: 'Elegir el ejecutable whisper-cli',
    speakerDifferentiation: 'Distinción de intervinientes',
    speakerLead:
      'La separación de turnos de palabra indica quién habló y cuándo. Es opcional, nunca bloquea una transcripción y los nombres siguen siendo editables durante la revisión.',
    runtimeUnavailable: 'Entorno no disponible en esta instalación',
    optional: 'Opcional',
    checkReadiness: 'Comprobar disponibilidad',
    downloadingSpeakerModels: 'Descargando los modelos de separación de intervinientes',
    speakerRuntimeMissing:
      'Los modelos están preparados, pero esta instalación no tiene un entorno compatible.',
    whereWorkIsKept: 'Dónde se guarda su trabajo',
    workspaceNote:
      'LocaLog gestiona esta carpeta para que las rutas de su interior sigan siendo válidas, pero es suya y puede mirar dentro cuando quiera.',
    showInFinder: 'Mostrar en el Finder',
    backup: 'Copia de seguridad',
    backupLead:
      'Todo se queda en este dispositivo, lo que también significa que se va con él. Una copia de seguridad es una carpeta corriente, que puede poner en un disco o donde guarde lo que importa.',
    backUpNow: 'Hacer una copia ahora',
    working: 'En curso…',
    backupContents:
      'Contiene todos los proyectos, reuniones, transcripciones y actas, y las grabaciones mismas. Dos cosas se dejan fuera a propósito, porque no son su trabajo y ambas se reconstruyen cuando hacen falta: los modelos descargados y la copia preparada de cada grabación. Medido en un espacio de trabajo real, ese audio preparado era por sí solo tres cuartas partes de la copia.',
    restore: 'Restaurar',
    restoreLead:
      'Vuelve a poner una copia de seguridad. Primero se comprueba entera, y lo que hay aquí se aparta en lugar de eliminarse.',
    chooseBackup: 'Elegir una copia…',
    chooseBackupTitle: 'Elegir una copia de seguridad de LocaLog',
    whereToKeepBackup: 'Dónde guardar la copia',
    replaceWorkspace: 'Sustituir este espacio de trabajo',
    restoring: 'Restaurando…',
    archived: 'Archivado',
    archivedLead:
      'Proyectos y reuniones apartados. No se ha eliminado nada: cada reunión, transcripción y acta que contienen sigue aquí, y sigue en todas las copias de seguridad.',
    show: 'Mostrar',
    hide: 'Ocultar',
    nothingArchived: 'No se ha archivado nada.',
    project: 'Proyecto',
    meeting: 'Reunión',
    bringBack: 'Recuperar',
    theme: 'Tema',
    themeFollowing: (theme: string) => `Sigue a este Mac, que está en ${theme}.`,
    themeSetHere: 'Fijado aquí, sea cual sea el ajuste de este Mac.',
    nextFakeJob: 'Próxima tarea simulada',
    nextFakeJobDetail:
      'Control solo para desarrollo, para revisar los estados de fallo y reintento.',
    completeNormally: 'Termina con normalidad',
    failOnce: 'Falla una vez y luego permite reintentar',
    syntheticNote: 'Esto solo afecta al entorno sintético en memoria.',
  },

  project: {
    deleteMeeting: (title: string) => `Eliminar ${title}`,
    deleteWarning:
      'Eliminar una reunión borra de este dispositivo su grabación, su transcripción y todas las versiones de su acta. No se puede deshacer.',
    eyebrow: 'Proyecto',
    archiveProject: 'Archivar el proyecto',
    newMeeting: 'Nueva reunión',
    meetings: 'Reuniones',
    newestFirst: 'Las más recientes primero',
    columnDate: 'Fecha',
    columnMeeting: 'Reunión',
    columnDuration: 'Duración',
    columnStatus: 'Estado',
    archive: 'Archivar',
    delete: 'Eliminar',
    keep: 'Conservar',
    noMeetings: 'Todavía no hay reuniones',
    noMeetingsDetail:
      'Importe la primera grabación para empezar el registro de reuniones de este proyecto.',
    importRecording: 'Importar una grabación',
  },

  lifecycle: {
    draft: 'Borrador',
    sourceReady: 'Listo para transcribir',
    transcriptReady: 'Transcripción lista',
    protocolDraft: 'Borrador del acta',
    reviewed: 'Revisado',
    archived: 'Archivado',
  },

  sections: {
    noHeadings: 'Esta acta todavía no tiene títulos, así que no hay nada que listar.',
    setAside: 'Apartar',
    addSection: 'Añadir una sección',
    dragHint: 'Arrastre, o use las flechas',
    setThisAside: 'Apartar esta sección',
    putThisBack: 'Devolver esta sección',
    moveSection: (title: string) => `Mover ${title}. Use las flechas.`,
    setAsideNamed: (title: string) => `Apartar ${title}`,
    putBackNamed: (title: string) => `Devolver ${title}`,
    setAsideNote:
      'Una sección apartada sale del documento, así que lo que lee es exactamente lo que se exporta. Se conserva aquí y se puede devolver.',
  },

  jobErrors: {
    interrupted: {
      title: 'La importación se ha interrumpido',
      detail:
        'LocaLog se detuvo antes de confirmar la copia gestionada. El original externo sigue intacto y puede reintentarlo sin riesgo.',
    },
    permission_denied: {
      title: 'LocaLog no ha podido leer ni almacenar la grabación',
      detail:
        'Compruebe el acceso al archivo elegido y a la carpeta de datos local de LocaLog, y vuelva a intentarlo. El original externo no se ha modificado.',
    },
    insufficient_space: {
      title: 'No hay suficiente almacenamiento local',
      detail:
        'Libere espacio y vuelva a intentarlo. No se ha presentado como completa ninguna grabación parcial.',
    },
    source_missing: {
      title: 'La grabación elegida ya no está disponible',
      detail:
        'Vuelva a poner el archivo en su sitio, o cree una importación nueva. La reunión sigue a salvo en borrador.',
    },
    source_reselection_required: {
      title: 'Vuelva a elegir la grabación',
      detail:
        'Esta reunión la creó una versión de desarrollo anterior que no conservaba la ubicación del origen. Vuelva a elegir la grabación para continuar; la reunión se ha conservado.',
    },
    unsupported_media: {
      title: 'Este tipo de medio todavía no es compatible',
      detail:
        'Elija un archivo de audio o vídeo corriente. El original externo no se ha modificado.',
    },
    empty_source: {
      title: 'La grabación elegida está vacía',
      detail:
        'Elija una grabación que contenga datos de audio o vídeo. El archivo externo vacío no se ha modificado.',
    },
    synthetic_failure: {
      title: 'El adaptador de desarrollo se ha detenido como se le pidió',
      detail:
        'El fallo provocado ocurrió antes de confirmar ninguna versión. Su origen y su último estado estable están a salvo, y puede reintentarlo.',
    },
    invalid_adapter_output: {
      title: 'No se ha podido validar la salida local',
      detail:
        'LocaLog no ha guardado ese resultado incompleto. Su último origen estable y las versiones de sus documentos están a salvo.',
    },
    runtime_missing: {
      title: 'Elija un entorno de transcripción local',
      detail:
        'Seleccione un ejecutable de whisper.cpp instalado en Ajustes → Transcripción. LocaLog no descarga entornos.',
    },
    model_missing: {
      title: 'Elija un modelo de transcripción local',
      detail:
        'Seleccione un modelo de whisper.cpp ya disponible en Ajustes → Transcripción. No se ha descargado ni cambiado ningún modelo.',
    },
    runtime_changed: {
      title: 'El entorno de transcripción ha cambiado',
      detail:
        'La tarea en cola no se ejecutó porque su ejecutable de whisper.cpp ya no coincide con el entorno registrado. Reinténtelo para resolver el entorno actual.',
    },
    model_changed: {
      title: 'El modelo de transcripción ha cambiado',
      detail:
        'La tarea en cola no se ejecutó porque su modelo ya no coincide con la huella registrada. Reinténtelo para resolver el modelo actual.',
    },
    media_probe_failed: {
      title: 'No se ha podido examinar la grabación',
      detail:
        'Compruebe que FFprobe está instalado y que el origen importado sigue siendo legible. El original queda intacto.',
    },
    normalization_failed: {
      title: 'No se ha podido preparar la grabación',
      detail:
        'Compruebe que FFmpeg está instalado y vuelva a intentarlo. La copia preparada se puede regenerar y el original queda intacto.',
    },
    transcription_failed: {
      title: 'La transcripción local no ha podido terminar',
      detail:
        'El entorno de whisper.cpp se detuvo antes de confirmar una versión de la transcripción. Compruebe su modelo y vuelva a intentarlo.',
    },
    transcription_timeout: {
      title: 'La transcripción local ha tardado demasiado',
      detail:
        'El proceso supervisado de transcripción se detuvo antes de confirmar una versión. Compruebe la grabación y el entorno, y vuelva a intentarlo.',
    },
    provider_model_missing: {
      title: 'El modelo local seleccionado no está disponible',
      detail:
        'El modelo de Ollama seleccionado ya no está instalado. Elija uno instalado en Ajustes → Generación del acta y vuelva a intentarlo.',
    },
    provider_model_changed: {
      title: 'El modelo local ha cambiado',
      detail:
        'La huella del modelo cambió después de encolar esta tarea. Reinténtelo para tomar el modelo instalado ahora.',
    },
    provider_runtime_changed: {
      title: 'El proveedor local ha cambiado',
      detail:
        'La versión del entorno de Ollama cambió después de encolar esta tarea. Reinténtelo para tomar el entorno actual.',
    },
    provider_unavailable: {
      title: 'La generación local del acta no ha podido conectar',
      detail:
        'Inicie su instalación de Ollama y vuelva a intentarlo. LocaLog no inicia ni descarga entornos.',
    },
    provider_invalid_output: {
      title: 'No se ha podido validar la salida del modelo local',
      detail:
        'LocaLog no ha guardado ese acta incompleta o mal formada. Su transcripción está a salvo y puede reintentarlo.',
    },
    provider_incomplete_output: {
      title: 'No se ha podido validar la salida del modelo local',
      detail:
        'LocaLog no ha guardado ese acta incompleta o mal formada. Su transcripción está a salvo y puede reintentarlo.',
    },
    provider_response_too_large: {
      title: 'La respuesta del modelo local era demasiado grande',
      detail:
        'La respuesta superó el límite de seguridad de LocaLog y no se guardó. Inténtelo con una transcripción más corta o con otro modelo local.',
    },
    invalid_transcript_output: {
      title: 'No se ha podido validar la salida de la transcripción',
      detail:
        'LocaLog no ha guardado la salida del entorno porque estaba incompleta o mal formada. Su origen está a salvo.',
    },
    processing_failed: {
      title: 'El procesamiento local no ha podido terminar',
      detail:
        'No se ha presentado como lista ninguna transcripción ni acta incompleta. Su último estado estable sigue disponible y puede reintentarlo.',
    },
    unknown: {
      title: 'La importación no ha podido terminar',
      detail:
        'La reunión sigue en borrador y el original externo no se ha modificado. Puede reintentarlo sin riesgo.',
    },
  },

  jobStages: {
    transcriptSaved: 'Transcripción guardada',
    protocolSaved: 'Acta guardada',
    importComplete: 'Importación terminada: original intacto',
    processingCancelled: 'Procesamiento local cancelado: se conserva el estado estable',
    processingInterrupted: 'Procesamiento local interrumpido: se conserva el estado estable',
    processingFailed: 'El procesamiento local no ha terminado: se conserva el estado estable',

    ready_to_import: 'Listo para traer la grabación',
    copying: 'Trayendo la grabación',
    stoppingSafely: 'Deteniendo con seguridad',
    temporary_complete: 'Ya casi',
    finalizing: 'Guardando la grabación con seguridad',
    duplicate_confirmation: 'Puede que esta grabación ya esté aquí',
    completed: 'La grabación ya está',
    cancelled: 'Importación cancelada: original intacto',
    interrupted: 'Importación interrumpida: original intacto',
    failed: 'La importación no ha terminado: original intacto',
    probing_media: 'Examinando la grabación',
    normalizing_audio: 'Preparando el audio',
    output_staged: 'Guardando con seguridad',

    transcription_queued: 'Listo para transcribir',
    checking_source: 'Comprobando la grabación',
    loading_transcription_model: 'Cargando el modelo',
    transcribing_audio: 'Transcribiendo',
    separating_speakers: 'Distinguiendo a los intervinientes',
    validating_transcript: 'Guardando la transcripción',
    preparing_fake_transcriber: 'Preparando',
    transcribing_synthetic_segments: 'Creando los fragmentos de la transcripción',

    generation_queued: 'Listo para redactar el acta',
    checking_transcript: 'Comprobando la transcripción',
    resolving_protocol_inputs: 'Reuniendo el estilo y los términos',
    condensing_transcript: 'Leyendo la reunión entera',
    generating_protocol: 'Redactando el borrador del acta',
    validating_protocol: 'Guardando el acta',
    reading_introductions: 'Leyendo quién se ha presentado',

    protocol_would_not_fit: 'Esta reunión es más larga de lo que cabe en una sola pasada',
    segments_no_subject_claimed: 'Parte de la reunión no ha entrado en ningún tema',
    sections_over_their_length: 'Algunas secciones han salido más largas de lo pedido',

    finding_subjects: (detail: string) =>
      detail ? `Buscando de qué se habló — pasaje ${detail}` : 'Buscando de qué se habló',
    writing_section: (detail: string) =>
      detail ? `Redactando ${detail}` : 'Redactando el acta sección por sección',
    joining_subjects: (detail: string) =>
      detail ? `Uniendo temas afines — ${detail} encontrados` : 'Uniendo temas afines',
    joined_subjects: (detail: string) => (detail ? `Temas unidos — ${detail}` : 'Temas unidos'),
    joining_failed: (detail: string) =>
      detail ? `No se han podido unir los temas — ${detail}` : 'No se han podido unir los temas',

    working: 'En curso',
  },

  stages: {
    label: 'Fases de la reunión',
    source: 'Origen',
    transcript: 'Transcripción',
    protocol: 'Acta',
  },

  progress: {
    needsAttention: 'Necesita su atención',
    backgroundWork: 'Trabajo en segundo plano',
    cancellingSafely: 'Cancelando con seguridad…',
    cancel: 'Cancelar',
    speakerPassNote:
      'Esta pasada lee la grabación entera para comparar los turnos de palabra. Una grabación larga puede tardar unos minutos; puede cancelar con seguridad en cualquier momento.',
    latestRetained: 'Se conserva el último estado estable',
    originalUnchanged: ' · original externo intacto',
    retry: 'Reintentar',
    importing: 'Importando la grabación',
    transcribing: 'Transcribiendo',
    generating: 'Generando el acta',
    separatingSpeakers: 'Separando a los intervinientes',
    working: 'En curso…',
    duplicateNote: 'Ese mismo contenido ya está en LocaLog. No se ha unido ni descartado nada.',
    cancelImport: 'Cancelar la importación',
    importAnotherCopy: 'Importar otra copia',
    chooseSourceAgain: 'Volver a elegir el origen',
    continueImport: 'Continuar la importación',
    transcribeAgain: 'Volver a iniciar la transcripción',
    generateAgain: 'Volver a iniciar la generación',
  },

  newProject: {
    namesHeading: 'Nombres y términos',
    namesLead:
      'Una transcripción no puede adivinar un nombre que nunca ha oído. Darlos ahora es el minuto más útil que puede dedicar a este proyecto: un nombre mal oído se repite en cada acta hecha a partir de esa grabación, y ningún paso posterior lo recupera.',
    namesPeople: 'Personas',
    namesPeopleHint: 'Quienes es probable que asistan, o a quienes se mencione en una reunión.',
    namesOrganisations: 'Empresas y clientes',
    namesOrganisationsHint: 'La propiedad, los demás técnicos, los proveedores.',
    namesProject: 'Este proyecto',
    namesProjectHint: 'Cómo se llaman el proyecto, la parcela o el edificio.',
    namesTerms: 'Cualquier otra cosa que convenga escribir bien',
    namesTermsHint:
      'Las palabras propias de este trabajo que una transcripción general no conocería.',
    namesNote:
      'Sepárelos con comas. Todo es opcional y nada es definitivo: puede añadir y corregir términos cuando quiera en Nombres y términos, y una corrección hecha durante la revisión de una transcripción también se conserva aquí.',
    creating: 'Creando…',
    createAndContinue: 'Crear y continuar',
    afterCreated:
      'El estilo del acta, y los nombres y términos de este trabajo, se pueden fijar después de crear el proyecto. Los nombres bien valen un minuto: son lo que una transcripción no puede adivinar.',
    eyebrow: 'Proyectos',
    title: 'Nuevo proyecto',
    lead: 'Cree el marco profesional al que pertenecen las reuniones y los orígenes.',
    defaults: 'Valores por omisión del proyecto',
    name: 'Nombre del proyecto',
    namePlaceholder: 'p. ej. Estudio del centro cívico',
    description: 'Descripción',
    descriptionOptional: 'opcional',
    descriptionPlaceholder: 'Una descripción interna breve',
    defaultLanguage: 'Idioma de reunión por omisión',
    defaultLanguageDetail: 'Independiente del idioma de la interfaz.',
    cancel: 'Cancelar',
  },

  appearance: {
    font: 'Tipografía',
    appliesToProject: (project: string) =>
      `Se aplica a todas las actas de ${project}, para que los documentos de un estudio se parezcan. Cambia cómo se compone el acta, nunca lo que dice: eso es el estilo de arriba.`,
    bodySize: 'Tamaño del texto',
    headingScale: 'Escala de los títulos',
    lineSpacing: 'Interlineado',
    pageWidth: 'Ancho de página',
  },

  record: {
    recordingNow: 'Grabando',
    recordThisMeeting: 'Grabar esta reunión',
    lead: 'La sala y la llamada se captan en pistas separadas, en este dispositivo. Si los presentes han dado su consentimiento es cosa suya; LocaLog no puede saberlo.',
    notRecording: 'Sin grabar',
    microphone: 'Micrófono',
    theCall: 'La llamada',
    trackRecording: 'Grabando',
    trackSilent: 'En silencio hasta ahora',
    trackListening: 'Escuchando…',
    stopRecording: 'Detener la grabación',
    finishing: 'Finalizando…',
    startRecording: 'Empezar a grabar',
    starting: 'Empezando…',
    backToMeeting: 'Volver a la reunión',
    noRecorder: 'Esta compilación no tiene grabador. Importe un archivo en su lugar.',
    openTheSetting: 'Abrir el ajuste',
    grantedInSettings: 'Concedido en los Ajustes del Sistema, y reconocido aquí en cuanto vuelva.',
    callWouldNotRecordTitle: 'La llamada no se grabaría.',
    callWouldNotRecordBody:
      'macOS no ha concedido a LocaLog la grabación de pantalla y audio del sistema, y sin ella una grabación de la llamada es silencio en vez de un error, así que conviene concederla ahora y no descubrirlo después. El micrófono de la sala sí se captaría.',
    roomWouldNotRecordTitle: 'La sala no se grabaría.',
    roomWouldNotRecordBody:
      'Se le ha denegado el micrófono a LocaLog. La llamada sí se captaría si el ajuste de arriba lo permite.',
    recorderNotesTitle: 'El grabador no ha podido hacer todo lo que se le pidió.',
    stoppedOnItsOwn:
      'El grabador se ha detenido solo. Lo que hubiera captado hasta ese momento se ha conservado.',
    quietCall: (seconds: number) =>
      `No llega nada de la llamada desde hace ${seconds} segundos. macOS le da silencio en vez de un error a una aplicación sin permiso de grabación de pantalla y audio del sistema, así que conviene comprobarlo ahora y no después de la reunión.`,
    quietMicrophone: (seconds: number) =>
      `No llega nada del micrófono desde hace ${seconds} segundos. Compruebe que está seleccionada la entrada correcta y que no la está ocupando otra cosa.`,
  },

  meeting: {
    browserPreview: 'Vista previa en el navegador',
    speakersEstimateNote:
      'LocaLog agrupa las voces que oye y las cuenta. Es una estimación, y puede sustituirla por un número si le parece equivocada.',
    speakersCountNote:
      'Basta con su mejor estimación: es el número de voces que LocaLog buscará. Demasiadas pueden partir a una persona en dos; muy pocas pueden juntar a dos personas.',
    speakersTogetherNote: 'La transcripción mantiene un único nombre de interviniente.',
    importInterrupted:
      'LocaLog se cerró antes de confirmar la copia gestionada. La reunión sigue en borrador y la importación se puede reintentar sin riesgo.',
    importCancelled:
      'Se ha cancelado la copia gestionada. La reunión sigue en borrador y el archivo externo no se ha modificado.',
    importFailed:
      'No se ha podido confirmar la copia gestionada. La reunión sigue en borrador y el archivo externo no se ha modificado.',
    importRunning:
      'LocaLog está copiando este origen a su propio almacenamiento. Estará listo solo cuando la copia se haya verificado y confirmado.',
    sourceStored:
      'está guardado con seguridad junto a esta reunión. El original externo no se ha modificado.',
    sourceSynthetic:
      'está asignado a esta reunión sintética del navegador. No se ha copiado ningún archivo real.',
    syntheticFixture: 'Material de demostración',
    eyebrow: 'Reunión',
    titleLabel: 'Título de la reunión',
    editTitle: 'Editar el título de la reunión',
    languageLabel: 'Idioma de la reunión',
    changeLanguage: 'Cambiar el idioma de la reunión',
    save: 'Guardar',
    saveLanguage: 'Guardar el idioma',
    cancel: 'Cancelar',
    recordingEyebrow: 'Grabación',
    nothingRecorded: 'Todavía no se ha grabado nada',
    recordLead:
      'La sala y la llamada se captarán en pistas separadas, en este dispositivo. Puede detenerlo cuando termine la reunión.',
    recordThisMeeting: 'Grabar esta reunión',
    sourceImport: 'Importación del origen',
    originalUnchanged: 'Su original sigue intacto',
    sourceReady: 'Origen listo',
    readyToTranscribe: 'Listo para transcribir',
    managedSource: 'Origen gestionado',
    language: 'Idioma',
    languageHint: 'Ajuste de la reunión · cámbielo arriba antes de transcribir',
    preset: 'Preajuste',
    globalDefault: 'Valor por omisión',
    notSelected: 'Sin seleccionar',
    peopleSpeaking: 'Personas que hablan',
    doNotSeparate: 'No distinguir a los intervinientes',
    separateAndCount: 'Distinguirlos y averiguar cuántos son',
    prepareSpeakers: 'Preparar la separación de intervinientes',
    prepareSpeakersDetail:
      'LocaLog necesita dos archivos de modelo locales verificados antes de poder añadir nombres provisionales. Su grabación se queda en este dispositivo.',
    preparing: (percent: number) => `Preparando ${percent} %`,
    prepare: 'Preparar',
    prepareWithSize: (size: string) => `Preparar (${size})`,
    speakerRuntimeMissing:
      'El entorno de separación de intervinientes no está disponible en esta instalación. La transcripción puede continuar, pero usará nombres genéricos editables.',
    reviewAndTrim: 'Revisar y recortar la grabación primero',
    trimDetail:
      '— quite la espera antes de empezar y todo lo que la reunión no necesite. Su grabación nunca se modifica.',
    gettingReady: 'Preparando la transcripción…',
    useJobControls: 'Use los controles de arriba',
    prepareSpeakersFirst: 'Prepare antes la separación de intervinientes',
    transcribe: 'Transcribir',
    transcriptionFailedToStart: 'No se ha podido iniciar la transcripción. Inténtelo de nuevo.',
    transcriptReady: 'Transcripción lista',
    reviewBeforeGeneration: 'Revisar antes de generar',
    transcriptReadyDetail:
      'La transcripción con marcas de tiempo está lista para correcciones y para asignar los intervinientes.',
    reviewTranscript: 'Revisar la transcripción',
    protocolAvailable: 'Acta disponible',
    continueInEditor: 'Continuar en el editor',
    protocolDetail: 'La transcripción sigue disponible junto a la versión actual del acta.',
    openProtocol: 'Abrir el acta',
  },

  newMeeting: {
    meetingOverride: 'Ajuste propio de esta reunión',
    preparing: 'Preparando…',
    bringingRecordingIn: 'Trayendo la grabación…',
    noPerMeetingOverrides:
      'Los ajustes propios de cada reunión y la elección de nombres y términos reunión a reunión todavía no están disponibles.',
    chosenOnceNote:
      'La calidad de la transcripción y el modelo que redacta el acta se eligen una vez, en los Ajustes, y valen para todas las reuniones.',
    titleRecording: 'Grabación',
    titleImport: 'Importación estructurada',
    heading: 'Nueva reunión',
    leadRecording:
      'Ponga nombre a la reunión y elija su proyecto. La grabación empieza en la pantalla siguiente.',
    leadImport: 'Elija la grabación, confirme los datos y LocaLog se encarga del resto.',
    context: 'Marco',
    chooseProject: 'Elegir un proyecto',
    project: 'Proyecto',
    newProject: 'Nuevo proyecto',
    noInbox:
      'Cada origen pertenece a una reunión, y cada reunión a un proyecto. No hay bandeja de entrada.',
    source: 'Origen',
    importRecording: 'Importar una grabación',
    originalStays: 'Su original se queda donde está',
    readyToCopy: 'Listo para copiarse cuando confirme esta reunión',
    letGoToImport: 'Suelte para importar',
    originalStaysShort: 'El original se queda donde está.',
    dropHere: 'Suelte aquí una grabación, o haga clic para elegir una',
    dropDetail:
      'MP3, M4A, WAV, MP4, MOV y otros. El original queda intacto: LocaLog lo copia a su propio almacenamiento.',
    readyToAssign: 'Listo para asignarse a esta reunión',
    chooseFile: 'Elegir un archivo de audio o vídeo',
    previewNote: 'La vista previa del navegador muestra el flujo sin guardar el archivo.',
    useDemoRecording: 'Usar la grabación de demostración',
    essentials: 'Lo esencial',
    meetingInformation: 'Datos de la reunión',
    title: 'Título',
    titlePlaceholder: 'Se toma del archivo si se deja vacío',
    date: 'Fecha',
    language: 'Idioma de la reunión',
    protocolStyle: 'Estilo del acta',
    projectDefault: 'Valor por omisión del proyecto',
    qualityNote:
      'La calidad de la transcripción se elige una vez en los Ajustes y vale para todas las reuniones.',
    advanced: 'Opciones avanzadas de procesamiento',
    cancel: 'Cancelar',
    createAndRecord: 'Crear la reunión y grabar',
    createAndImport: 'Crear la reunión e importar',
  },

  recordingReview: {
    lead: 'Corte lo que la reunión no necesita antes de transcribirla. Su grabación nunca se modifica: todo esto se puede deshacer.',
    noPreparedAudio:
      'Esta reunión todavía no tiene audio preparado que revisar. Estará disponible cuando se confirme la importación.',
    dragToSelect:
      'Arrastre por la grabación para seleccionar un tramo, o use las flechas manteniendo Mayúsculas.',
    selectedRange: (from: string, to: string) => `Seleccionado de ${from} a ${to}.`,
    eyebrow: 'Grabación',
    heading: 'Revisar la grabación',
    noAudio: 'Todavía no hay audio de trabajo',
    waveformLabel: 'La grabación. Muévase con las flechas y mantenga Mayúsculas para seleccionar.',
    keptOf: (kept: string, whole: string) => `${kept} de ${whole} conservados`,
    startsAt: (time: string) => `Empieza en ${time}`,
    endsAt: (time: string) => `Termina en ${time}`,
    removedSpan: (from: string, to: string) => `Quitado de ${from} a ${to}`,
    startHere: 'Empezar aquí',
    removeSelection: 'Quitar la selección',
    endHere: 'Terminar aquí',
    edits: 'Cortes',
    nothingRemoved: 'No se ha quitado nada. Se transcribirá la grabación entera.',
    undo: 'Deshacer',
    putEverythingBack: 'Devolverlo todo',
    untouchedNote: 'La grabación en sí queda intacta. Esto son indicaciones sobre qué usar.',
    undoStartTrim: 'Deshacer el recorte del principio',
    undoEndTrim: 'Deshacer el recorte del final',
    putStretchBack: 'Devolver este tramo',
    next: 'Siguiente',
    continueToTranscription: 'Pasar a la transcripción',
    backToMeeting: 'Volver a la reunión',
  },

  transcript: {
    heardAs: (heard: string) => `Oído como «${heard}»`,
    askAboutTheRest: 'Revisar el resto',
    askingAboutTheRest: 'Leyendo las frases…',
    askAboutTheRestNote:
      'Unas pocas palabras se oyen mal de forma distinta cada vez, así que corregir una grafía no las encuentra. Esto lee cada una en su propia frase y propone un nombre de la lista de este proyecto: no puede proponer otra cosa, y no cambia nada hasta que usted lo diga.',
    proposedNothing: 'No se ha reconocido nada más.',
    proposedNothingNote:
      'Que es la respuesta habitual, y una buena: solo puede proponer un nombre que este proyecto ya tenga, así que prefiere callarse antes que inventarse uno.',
    proposalsHeading: (count: number) => (count === 1 ? '1 propuesta' : `${count} propuestas`),
    proposalSuggests: (heard: string, suggested: string) => `${heard} → ${suggested}`,
    spellingsToCheck: (count: number) =>
      count === 1 ? '1 grafía que conviene revisar' : `${count} grafías que conviene revisar`,
    questionedByProtocol: 'el acta no ha reconocido esta palabra',
    autosaveFailed: 'El guardado automático ha fallado: su último estado guardado está intacto',
    correctCount: (count: number) => `Corregir ${count}`,
    audioCouldNotLoad: 'No se ha podido cargar el audio de trabajo de esta reunión.',
    pauseAudio: 'Pausar',
    playAudio: 'Reproducir',
    saving: 'Guardando…',
    editsSaved: 'Cambios guardados',
    revisionSaved: 'Versión de la transcripción guardada',
    separationUnavailableHere:
      'La separación de intervinientes todavía no está disponible en esta instalación. Puede continuar poniendo los nombres a mano.',
    rerunForSeparation:
      'Vuelva a ejecutar esta transcripción para obtener un resultado de separación actual.',
    separationUnavailableForRun:
      'La separación de intervinientes no estaba disponible en esta ejecución. Puede continuar poniendo los nombres a mano.',
    nothingChangedYet: 'Todavía no ha cambiado nada',
    readingOpening: 'Leyendo el comienzo…',
    readWhoIsHere: 'Leer quién participa en esta reunión',
    correcting: 'Corrigiendo…',
    durationPending: 'Duración por determinar',
    introducedThemselves: (count: number) => `${count} se han presentado`,
    noNamesYet: (project: string) => `Todavía no hay nombres en ${project}`,
    speltAsHeard:
      'Escritos como los oyó la transcripción. Corrija los que estén mal: se arreglarán aquí y se recordarán para este proyecto.',
    openingNote:
      'Una reunión suele empezar con gente diciendo quién es. Leer ese pasaje le da a este proyecto sus nombres, que es lo que una transcripción no puede adivinar.',
    foundInPlaces: (count: number) =>
      `Encontrado en ${count} ${count === 1 ? 'sitio' : 'sitios'}. Desmarque los que deban quedarse como están.`,
    noneMisheardEveryTime: (count: number) =>
      `Ninguna palabra se oyó mal todas las veces que apareció. Quedan ${count} pasajes marcados como poco claros por otros motivos.`,
    nothingFlaggedNote:
      'No se ha marcado nada como poco claro. Una transcripción hecha antes de que esto existiera tampoco muestra nada aquí, así que conviene releer una antigua en vez de fiarse de ella.',
    workingAudioLater:
      'El audio de trabajo estará disponible cuando se haya transcrito esta reunión.',
    recordingEndsNote:
      'Si la reunión siguió más allá, la grabación no lo captó y el acta no lo contendrá.',
    heading: 'Revisión de la transcripción',
    exportTranscript: 'Exportar la transcripción…',
    exportLabel: 'Exportar esta transcripción',
    asMarkdown: 'En Markdown',
    asPlainText: 'En texto sin formato',
    reviewDetails: 'Detalles de la revisión',
    sourceContext: 'Contexto del origen',
    seekAudio: 'Moverse por el audio',
    follow: 'Seguir',
    followLabel: 'Desplazar la transcripción hasta el fragmento que se está reproduciendo',
    searchTranscript: 'Buscar en la transcripción',
    editableTranscript: 'Transcripción editable',
    removeLine: 'Quitar esta línea de la transcripción',
    nothingFlagged: 'Nada marcado como poco claro',
    show: 'Mostrar',
    showing: 'Mostrando',
    onePassage: '1 pasaje poco claro',
    manyPassages: (count: number) => `${count} pasajes poco claros`,
    speakerHint:
      'Los nombres de los intervinientes son un punto de partida: cámbielos por las personas que hablaron.',
    generateProtocol: 'Generar el acta',
    review: 'Revisión',
    detailsLabel: 'Detalles de la revisión de la transcripción',
    closeInspector: 'Cerrar el panel',
    speakers: 'Intervinientes',
    whereRecordingStops: 'Donde se detiene la grabación',
    transcriptionInput: 'Entrada de la transcripción',
    language: 'Idioma',
    meetingLanguage: 'Idioma de la reunión',
    saveLanguage: 'Guardar el idioma',
    cancel: 'Cancelar',
    changeLanguage: 'Cambiar el idioma',
    rerunNote:
      'Úselo tras cambiar el idioma o los ajustes de transcripción. La nueva ejecución se guarda como una versión aparte.',
    rerun: 'Volver a transcribir',
    rerunPreparing: 'Preparando una transcripción nueva…',
    rerunConfirm: (language: string) =>
      `¿Volver a transcribir en ${language}? La transcripción actual seguirá hasta que se confirme el resultado nuevo, y entonces se sustituirá esta transcripción de trabajo.`,
    whoIsHere: 'Quién participa en esta reunión',
    close: 'Cerrar',
    aboutAMinute: 'Alrededor de un minuto. No puede ejecutarse nada más mientras tanto.',
    unsureNames: 'Nombres que merecen una segunda mirada',
    whatShouldItSay: '¿Cómo debería escribirse?',
    rememberForProject:
      'Recordarlo para este proyecto, para que la próxima reunión lo escriba bien',
    areAnyNames: '¿Alguno es un nombre? Corregir uno arregla esta transcripción y se recuerda.',
    nothingToCheck: 'Nada que revisar',
    correctSpelling: 'Corregir la grafía',
    checkWording: 'Revisar la redacción',
    checkWords: (words: string) => `Compruebe ${words}`,
    textAt: (time: string) => `Texto de la transcripción en ${time}`,
    jumpTo: (time: string) => `Ir a ${time}`,
    removeLineAt: (time: string) => `Quitar la línea en ${time}`,
    renameSpeaker: (speaker: string) => `Cambiar el nombre de ${speaker}`,
    nameHeardAs: (heard: string) => `Nombre oído como ${heard}`,
    protocolStyle: 'Estilo del acta',
    audioUnplayable: 'No se ha podido reproducir el audio de trabajo de esta reunión.',
    speakersResolved:
      'Los turnos de palabra se han resuelto localmente. Los nombres son provisionales: cámbielos solo cuando sepa de quién se trata.',
    speakersFailed:
      'La separación de intervinientes no ha producido turnos utilizables en esta ejecución. La transcripción está intacta y usa nombres neutros; puede continuar poniéndolos a mano.',
    speakersUnavailable:
      'La separación de intervinientes no estaba disponible en esta ejecución. La transcripción está intacta y usa un nombre neutro; puede cambiarlo a mano.',
    speakersUnknown:
      'Esta transcripción antigua no registra si se ejecutó la separación de intervinientes. Sus nombres neutros no prueban que hablara una sola persona.',
  },

  library: {
    remove: 'Quitar',
    edit: 'Editar',
    keep: 'Conservar',
    notInUseSuffix: ' · sin usar',
    /** Véase la nota en en.ts: solo mientras un estilo no se haya renombrado. */
    shippedStyle: {
      'style-formal': {
        name: 'Acta formal',
        description: 'Registro estructurado de la discusión, las decisiones y las acciones.',
      },
      'style-working-note': {
        name: 'Nota de trabajo interna',
        description: 'Registro de trabajo conciso para un equipo de proyecto interno.',
      },
      'style-decision-log': {
        name: 'Registro de decisiones técnicas',
        description: 'Destaca las alternativas, las restricciones y las decisiones explícitas.',
      },
    },
    copyOf: (name: string) => `${name} (copia)`,
    enterATerm: 'Escriba un término.',
    reading: 'Leyendo…',
    editTerm: 'Editar el término',
    inUse: 'En uso',
    notInUse: 'Sin usar',
    instructionsGiven:
      'Estas son las instrucciones que se le dan al modelo, en el orden en que se le dan',
    asShipped: ', exactamente como venía este estilo',
    invariantsNote:
      'No forman parte de este estilo y no se pueden editar aquí: no se guardan con ningún estilo. Se añaden a cada acta mientras se redacta, porque un documento que recoge una decisión que nadie tomó no es un acta de otro estilo, sino un acta falsa.',
    whichTermsHelp:
      'Los nombres, las empresas y las siglas son lo que más ayuda. La terminología profesional corriente suele transcribirse bien sin necesidad de listarla.',
    termsLeadLong:
      'Añada los nombres, empresas y siglas que usa este trabajo para que se transcriban bien. En una reunión real de ochenta minutos, esto llevó el nombre del propio proyecto de no escribirse nunca bien a escribirse siempre bien.',
    eyebrow: 'Biblioteca',
    protocolStyles: 'Estilos de acta',
    namesAndTerms: 'Nombres y términos',
    stylesLead:
      'Qué dice un acta, y en qué orden. No cómo se compone: eso es la presentación, y vive en el editor junto al documento que describe.',
    termsLead:
      'Los nombres que una transcripción no puede adivinar: su proyecto, las empresas, las personas. Medido en una reunión real, valen más que cualquier otro ajuste de aquí.',
    addTerm: 'Añadir un término',
    saveTerm: 'Guardar el término',
    stylesUnreadable: 'Aquí no se pueden leer los estilos.',
    length: 'Extensión',
    name: 'Nombre',
    description: 'Descripción',
    whatItAsksFor: 'Qué pide este estilo',
    addInstruction: 'Añadir una instrucción',
    removeInstruction: 'Quitar esta instrucción',
    checkedOnProtocol: 'Comprobado sobre el acta terminada',
    alwaysEveryStyle: 'Siempre, en todos los estilos',
    saveStyle: 'Guardar el estilo',
    cancel: 'Cancelar',
    delete: 'Eliminar',
    editThisStyle: 'Editar este estilo',
    duplicate: 'Duplicar',
    duplicateToEdit: 'Duplicar para editar',
    shippedStyleNote:
      'Un estilo que viene con la aplicación se queda como está, para que un acta redactada el año pasado pueda redactarse igual hoy. Cópielo para hacer el suyo.',
    ownershipAutomatic: 'La asignación es automática.',
    termsScopeNote:
      'Los nombres y términos de un proyecto valen para sus reuniones sin tener que elegirlos cada vez.',
    term: 'Término',
    spellingAsShown: 'Grafía tal como debe aparecer',
    category: 'Categoría',
    appliesTo: 'Se aplica a',
    everyProject: 'Todos los proyectos',
    unknownProject: 'Proyecto desconocido',
    noTerms: 'Todavía no hay nombres ni términos',
    deleteThisTerm: '¿Eliminar este término?',
    densityFull: 'Prosa completa',
    densityPlain: 'Enunciados simples',
    densityLine: 'Una línea por punto',
    densityFullMeaning: 'Prosa completa. Quien no estuvo puede seguir la discusión.',
    densityPlainMeaning: 'Enunciados simples. Lo que se dijo, sin el relato.',
    densityLineMeaning: 'Una línea por punto. El registro, y nada alrededor.',
    categoryPerson: 'Persona',
    categoryOrganisation: 'Empresa',
    categoryProject: 'Proyecto',
    categoryAbbreviation: 'Sigla',
    categoryTechnicalTerm: 'Término técnico',
    categoryOther: 'Otro',
  },

  furniture: {
    header: 'Encabezado',
    footer: 'Pie de página',
    left: 'Izquierda',
    centre: 'Centro',
    insertInto: (where: string) => `Insertar un valor en ${where}`,
    right: 'Derecha',
    insert: 'Insertar…',
    lineHint:
      'Escriba la línea tal como debe leerse, y ponga un valor donde quiera uno: «Página », el número, « de 12». Un valor es un solo objeto: se selecciona y se borra entero.',
    appliesTo: (project: string) =>
      `Se aplica a todas las actas de ${project}. Se repite en la página impresa y no forma parte del documento que está editando.`,
  },

  shell: {
    breadcrumbMeeting: 'Reunión',
    breadcrumbRecording: 'Grabación',
    breadcrumbReview: 'Revisión',
    skipToWorkspace: 'Ir al espacio de trabajo',
    workspace: 'Espacio de trabajo',
    workspaceFailed: 'No se ha podido abrir el espacio de trabajo',
    workspaceFailedDetail: 'Sus archivos existentes no se han modificado.',
    tryAgain: 'Volver a intentarlo',
    preparingWorkspace: 'Preparando el espacio de trabajo local…',
    openNavigation: 'Abrir la navegación',

    notSelected: 'Sin seleccionar',

    jobNeedsDecision: 'Necesita su decisión',
    jobReadyToContinue: 'Listo para continuar',
    jobCancelling: 'Cancelando con seguridad',

    formatWordDocument: 'Documento de Word',
    formatPlainText: 'Texto sin formato',
    exportSaved: (format: string) => `Exportación a ${format} guardada`,
    exportFailed: (format: string, why: string) => `Ha fallado la exportación a ${format}: ${why}`,
    exportPrepared: (format: string) => `Exportación a ${format} preparada`,
    exportNeedsDesktop: (format: string) =>
      `La exportación a ${format} necesita la aplicación de escritorio.`,

    meetingArchived: 'Reunión archivada. Está en Ajustes, en Almacenamiento.',
    projectArchived: 'Proyecto archivado. Está en Ajustes, en Almacenamiento.',
    transcriptExported: 'Transcripción exportada',
  },

  protocol: {
    undo: 'Deshacer',
    redo: 'Rehacer',
    next: 'Siguiente',
    blockParagraph: 'Párrafo',
    blockHeading1: 'Título 1',
    blockHeading2: 'Título 2',
    blockHeading3: 'Título 3',
    figuresMissingFromRewrite: (count: number) =>
      `Faltan en esta reformulación ${count} cifras que el pasaje sí daba`,
    markdownView: 'Vista Markdown',
    documentView: 'Vista de documento',
    looking: 'Buscando…',
    replaceAll: 'Reemplazar todo',
    rewrite: 'Reformular',
    rewriting: 'Reformulando',
    figureMissingFromRewrite: 'Falta en esta reformulación una cifra que el pasaje sí daba',
    reviewedRevisionPreserved:
      'La versión revisada se conserva. Estos cambios de trabajo no se han revisado.',
    thisRevisionReviewed: 'Esta versión exacta e inmutable se marcó como revisada.',
    generatedStaysEditable: 'El contenido generado sigue siendo revisable y editable.',
    notFound: 'No encontrado',
    matchCount: (count: number) => `${count} ${count === 1 ? 'coincidencia' : 'coincidencias'}`,
    replacedCount: (count: number) => ` · ${count} reemplazadas`,
    changesNotYetMade: (count: number) =>
      `${count} ${count === 1 ? 'cambio' : 'cambios'}, aún sin aplicar`,
    compoundNote:
      'Un nombre con mayúscula se busca también dentro de las palabras compuestas, que es donde un reemplazo simple lo pierde. Léalos y luego consérvelos o déjelos.',
    andMore: (count: number) => `y ${count} más, todas de las dos mismas formas.`,
    passageGoesAlone:
      'El pasaje va solo a su modelo local. Los números, los nombres y las fechas deben volver sin cambios: compruébelos y deshaga si no es así.',
    nothingChangedYet:
      'Todavía no se ha cambiado nada. Léalo y luego consérvelo o déjelo: un modelo local reformula bien y no hay que creerle sin más.',
    secondPassNote:
      'Se le ha preguntado a su propio modelo, y se equivoca en los dos sentidos: se le escapan cambios y señala redacciones que están bien. Merece una mirada, no un veredicto.',
    pageEdgesNote:
      'Dónde terminarían las páginas, medido como las compone la hoja de estilo de impresión: un título o una tabla bajan enteros en vez de partirse; la prosa no. La impresora decide la última línea o dos, así que tómelo con un margen de una línea y no al milímetro.',
    transcriptSourceNote:
      'Redactado a partir de la transcripción revisada de esta reunión. Nada registra qué pasaje produjo qué frase, así que lo que sigue busca las palabras en lugar de afirmar que lo sabe: una paráfrasis no encontrará nada, que es la respuesta honesta.',
    noWordsTogether:
      'Estas palabras no aparecen juntas en la transcripción. Eso suele significar que el borrador lo ha dicho con sus propias palabras, cosa que puede hacer: la grabación es donde comprobarlo.',
    revisionNote:
      'Lo que escribe se guarda como cambios de trabajo y no crea una versión. Se crea una versión cuando se genera un borrador, cuando usted la pide, cuando marca un acta como revisada y cuando se restaura una anterior, para que esta lista siga siendo legible.',
    nothingRewrites:
      'Aquí no hay nada que reescriba su texto por usted. El borrador es suyo, y todas las versiones se conservan.',
    figuresKept: (kept: number, stated: number) => `${kept} de ${stated} cifras conservadas`,
    figuresNote: (stated: number, kept: number) =>
      `La reunión dio ${stated} cifras y este borrador repite ${kept}. Cuántas deben estar aquí depende del estilo que haya elegido, así que esto es algo que mirar y no una nota.`,
    figuresInvented: (count: number) =>
      count === 1
        ? 'Aquí aparece una cifra que la reunión no dio'
        : `Aquí aparecen ${count} cifras que la reunión no dio`,
    confirmAgainstRecording: '. Conviene comprobarlo con la grabación.',
    tasksUnowned: (count: number) =>
      count === 1
        ? 'Aquí hay una tarea sin nadie al lado'
        : `Aquí hay ${count} tareas sin nadie al lado`,
    unownedNote:
      '. El borrador prefiere dejar fuera al responsable antes que adivinarlo, así que puede que sea exactamente lo que se decidió en la reunión, y sale mucho más barato ponerle nombre ahora que en la reunión siguiente.',
    editor: 'Editor de actas',
    markdownBacked: 'basado en Markdown',
    noteMissingTableHeading: 'Sin tabla de próximos pasos',
    noteMissingTableBody:
      'Esta acta se redactó tres veces y ninguna de las versiones terminó con una tabla de las tareas acordadas y sus responsables. Las acciones que la reunión acordó se describen en las secciones de arriba, pero no están recogidas aquí.',
    noteGapsHeading: 'No cubierto por esta acta',
    noteOneGap:
      'Un tramo de la grabación no se ha podido leer, y nada de lo anterior lo describe. La grabación en sí está completa y todavía se puede escuchar.',
    noteSeveralGaps:
      'Varios tramos de la grabación no se han podido leer, y nada de lo anterior los describe. La grabación en sí está completa y esos tramos todavía se pueden escuchar.',
    documentType: 'Acta',
    statusDraft: 'Borrador',
    statusReviewed: 'Revisada',
    statusChanged: 'Modificada desde la revisión',
    fieldProjectName: 'Nombre del proyecto',
    fieldMeetingTitle: 'Título de la reunión',
    fieldMeetingDate: 'Fecha de la reunión',
    fieldDocumentType: 'Tipo de documento',
    fieldProtocolStatus: 'Estado',
    fieldPageNumber: 'Número de página',
    fieldPageOfCount: 'Página n de m',
    fieldText: 'Texto libre',
    showPageBreaks: 'Mostrar los saltos de página',
    hidePageBreaks: 'Ocultar los saltos de página',
    saving: 'Guardando…',
    autosaveFailed: 'Ha fallado el guardado automático',
    workingEditsSaved: 'Cambios de trabajo guardados',
    revisionSaved: 'Versión guardada',
    editorTools: 'Herramientas',
    find: 'Buscar',
    findInProtocol: 'Buscar en el acta',
    replaceWith: 'Reemplazar por',
    makeChanges: 'Aplicar estos cambios',
    leaveIt: 'Dejarlo',
    zoomOut: 'Reducir',
    zoomIn: 'Ampliar',
    insertTable: 'Insertar una tabla',
    insertDivider: 'Insertar un separador',
    documentMenu: 'Menú del documento',
    clearFormatting: 'Quitar el formato',
    table: 'Tabla',
    blockType: 'Tipo de bloque',
    addColumnLeft: 'Añadir una columna a la izquierda',
    addColumnRight: 'Añadir una columna a la derecha',
    deleteColumn: 'Eliminar esta columna',
    addRowAbove: 'Añadir una fila encima',
    addRowBelow: 'Añadir una fila debajo',
    deleteRow: 'Eliminar esta fila',
    formatting: 'Formato',
    bold: 'Negrita',
    italic: 'Cursiva',
    bulletedList: 'Lista con viñetas',
    numberedList: 'Lista numerada',
    quotation: 'Cita',
    askModel: 'Pedir al modelo que lo diga de otro modo',
    customInstruction: 'Instrucción propia…',
    whatShouldChange: '¿Qué hay que cambiar?',
    proposedChange: 'Cambio propuesto',
    proposedReplacement: 'Reemplazo propuesto',
    proposedRewrite: 'Reformulación propuesta',
    unchanged: 'El modelo ha devuelto el pasaje sin cambios.',
    factsMoved: 'Una segunda pasada cree que estos datos se han movido',
    noFactMoved: 'Una segunda pasada no ha visto moverse ningún dato. Se le escapan cosas.',
    useThis: 'Usar esto',
    improveClarity: 'Aclarar',
    improveClarityInstruction: 'Haz que esto se lea con más claridad.',
    makeFormal: 'Hacer más formal',
    makeFormalInstruction: 'Usa un registro más formal, como se redactaría un acta profesional.',
    makePlainer: 'Hacer más directo',
    makePlainerInstruction: 'Haz la redacción más sencilla y directa, sin perder precisión.',
    shorten: 'Acortar',
    shortenInstruction: 'Di esto con menos palabras.',
    rewriteUnavailable: 'Aquí no está disponible la reformulación.',
    replaceUnavailable: 'Aquí no está disponible el reemplazo de un nombre.',
    nameNotFound: 'Ese nombre no está en esta acta.',
    protocolMarkdown: 'Markdown del acta',
    protocolLabel: 'Acta',
    protocolDetails: 'Detalles del acta',
    documentDetails: 'Detalles del documento',
    closeInspector: 'Cerrar el panel',
    tabDocument: 'Documento',
    tabTranscript: 'Transcripción',
    tabHistory: 'Historial',
    status: 'Estado',
    createRevision: 'Crear una versión',
    lineNumber: (line: number) => `Línea ${line}`,
    pageNumber: (page: number) => `Página ${page}`,
    revisionNumber: (ordinal: number) => `Versión ${ordinal}`,
    markReviewed: 'Marcar como revisada',
    style: 'Estilo',
    sections: 'Secciones',
    newSection: 'Sección nueva',
    appearance: 'Presentación',
    editAppearance: 'Editar la presentación',
    headerFooter: 'Encabezado y pie de página',
    editHeaderFooter: 'Editar el encabezado y el pie de página',
    nothingRepeated: 'No se repite nada en la página',
    presets: 'Ajustes guardados',
    useOrSavePreset: 'Usar o guardar un ajuste',
    noneSaved: 'Todavía no hay ninguno guardado',
    savedCount: (count: number) => `${count} guardados`,
    use: 'Usar',
    remove: 'Quitar',
    nameThisPreset: 'Poner nombre a este ajuste',
    nameForPreset: 'Nombre de este ajuste',
    save: 'Guardar',
    cancel: 'Cancelar',
    saveAsPreset: 'Guardar esta presentación y este encabezado como ajuste',
    export: 'Exportar',
    exportPdf: 'Exportar a PDF',
    exportWord: 'Exportar a Word',
    exportMarkdown: 'Exportar a Markdown',
    exportPlainText: 'Exportar a texto sin formato',
    exportNote:
      'El PDF se imprime desde el documento que está leyendo, compuesto como este proyecto compone sus actas: elija «Guardar como PDF» en el diálogo de impresión.',
    source: 'Origen',
    findSelectedPassage: 'Encontrar el pasaje seleccionado',
    lookingFor: 'Buscando:',
    openReviewedTranscript: 'Abrir la transcripción revisada',
    whatToCheck: 'Qué conviene comprobar',
    revisions: 'Versiones',
    current: 'Actual',
    restore: 'Restaurar',
  },

  sidebar: {
    projects: 'Proyectos',
    newProject: 'Nuevo proyecto',
    createProject: 'Crear el proyecto',
    library: 'Biblioteca',
    protocolStyles: 'Estilos de acta',
    namesAndTerms: 'Nombres y términos',
    settings: 'Ajustes',
    recording: 'Grabación',
    primaryNavigation: 'Navegación principal',
    closeNavigation: 'Cerrar la navegación',
    openNavigation: 'Abrir la navegación',
    themeFollowingSystem: 'Sigue el tema del sistema. Cambiar a siempre claro.',
    themeAlwaysLight: 'Siempre claro. Cambiar a siempre oscuro.',
    themeAlwaysDark: 'Siempre oscuro. Volver al tema del sistema.',
    themeFollowingShort: 'Sigue al sistema',
    sidebarWidth: (width: number) => `${width} píxeles`,
    resizeSidebar:
      'Redimensionar el panel. Use las flechas para ajustarlo, o Intro para restablecerlo.',
    themeAlwaysLightShort: 'Siempre claro',
    themeAlwaysDarkShort: 'Siempre oscuro',

    importNeedsDecision: 'La importación necesita su decisión',
    needsAttention: 'Necesita su atención',
    importingRecording: 'Importando la grabación',
    transcribing: 'Transcribiendo',
    writingProtocol: 'Redactando el acta',
    working: 'En curso',
    workingEllipsis: 'En curso…',
    separatingSpeakers: 'Separando a los intervinientes',
    openMeetingNeedingAttention: 'Abrir la reunión que necesita atención',
    openThisMeeting: 'Abrir esta reunión',
  },

  start: {
    eyebrow: 'IA local para actas de reunión confidenciales',
    title: 'Empezar una reunión',
    lead: 'Importe un archivo de audio o vídeo. Revise cada paso antes de que se convierta en un acta.',
    importTitle: 'Importar una grabación',
    importDetail: 'Elija un proyecto y mantenga todo en su contexto',
    recordTitle: 'Grabar una reunión',
    recordDetail: 'Capte la sala y la llamada en este dispositivo, en pistas separadas',
    promiseTitle: 'Su trabajo de reuniones se queda en este dispositivo.',
    promiseDetail: 'Sin cuenta de LocaLog, sin servicio en la nube, sin telemetría.',

    setupProviderTitle: 'Una cosa más antes de la primera acta',
    setupProviderBody:
      'Transcribir ya funciona. Redactar el acta necesita además un modelo de lenguaje en este dispositivo, que se configura en los ajustes. Puede importar y transcribir una grabación antes de hacerlo.',
    setupProviderAction: 'Configurarlo en los ajustes',
    setupTitle: 'Una descarga antes de la primera transcripción',
    setupBody: (quality: string, size: string) =>
      `LocaLog transcribe en este dispositivo, así que el modelo tiene que estar aquí. La calidad ${quality} ocupa ${size} y se descarga una sola vez. Puede importar antes una grabación: el modelo hace falta cuando empieza la transcripción, no antes.`,
    setupDownload: (size: string) => `Descargarlo ahora (${size})`,
    setupCancel: 'Cancelar',
    setupAside: 'Las demás calidades, y la separación de intervinientes, están en los Ajustes.',
  },
};
