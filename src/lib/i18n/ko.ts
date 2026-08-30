/**
 * Every word the application says, in Korean.
 *
 * Typed against English, so this file cannot be missing a key or inventing one.
 *
 * ## Decisions taken once, here, so the whole application reads as one voice
 *
 * **회의록, never a transliteration of "protocol".** 회의록 is the established
 * Korean object with an expected shape — participants, decisions, action items
 * with owners — which is precisely what this product makes. 프로토콜 would name a
 * communications protocol and nothing else.
 *
 * **전사 for transcription, 전사본 for the transcript itself.** 녹취록 is the more
 * everyday word for a transcript of a recording and was the alternative; 전사 and
 * 전사본 are taken together so that the process and its result share a stem, which
 * matters in an interface where both appear on the same screen. 받아쓰기 is
 * dictation and would read as schoolwork.
 *
 * **합니다체 for statements, 하십시오체 for instructions.** The formal business
 * register, matching the German *Sie* and the French *vous*. Apple's Korean leans
 * on the softer 해요체, but this is written for offices keeping the formal record
 * of meetings.
 *
 * **회의 for a meeting, 녹음 for a recording, 트랙 for a track, 화자 for a speaker.**
 *
 * Korean has no plural, so the count functions collapse to one form with a
 * counter — 건, 곳, 명 — chosen for what is being counted.
 */

import type { Strings } from './en';

const failures = {
  missingProject: '선택한 프로젝트가 더 이상 존재하지 않습니다.',
  missingMeeting: '선택한 회의가 더 이상 존재하지 않습니다.',
  missingJob: '해당 가져오기 작업을 더 이상 사용할 수 없습니다.',
  importBusy: '다른 녹음을 이미 가져오는 중입니다. 먼저 완료하거나 취소하십시오.',
  unsupportedSchema: (version: string) =>
    `이 LocaLog 데이터는 더 새롭고 지원되지 않는 버전(${version})에서 만들어졌습니다.`,
  storageUnavailable: 'LocaLog가 로컬 작업 공간에 접근하지 못했습니다.',

  styleMissing: '해당 스타일이 더 이상 존재하지 않습니다.',
  styleNameRequired: '스타일에 이름을 지정하십시오.',
  styleNotSaved: '스타일을 저장하지 못했습니다.',
  styleUnavailable: '선택한 회의록 스타일을 사용할 수 없습니다.',
  styleUsedByMeeting: '이 스타일을 쓰는 회의가 있습니다. 먼저 그 회의를 바꾸십시오.',
  styleUsedByProject: '이 스타일을 기본값으로 쓰는 프로젝트가 있습니다. 먼저 그것을 바꾸십시오.',

  presetNameRequired: '프리셋에 이름을 지정하십시오.',
  presetNotSaved: '프리셋을 저장하지 못했습니다.',
  presetBuiltInUndeletable: 'LocaLog에 포함된 프리셋은 삭제할 수 없습니다.',

  transcriptInvalid: '저장된 전사본이 올바르지 않습니다.',
  transcriptSegmentMissing: '해당 전사 구간이 더 이상 존재하지 않습니다.',
  transcriptTextRequired: '올바른 전사 텍스트를 입력하십시오.',
  transcriptNeedsSegment: '전사본에는 구간이 최소한 하나 필요합니다.',
  transcriptSpeakerRequired: '올바른 화자 이름을 입력하십시오.',
  transcriptNotSaved: '전사본을 저장하지 못했습니다.',
  transcriptNotCommitted: '전사본을 확정하지 못했습니다.',
  spellingRequired: '올바른 표기를 입력하십시오.',

  protocolTextRequired: '올바른 회의록 텍스트를 입력하십시오.',
  protocolRevisionMissing: '선택한 회의록 판이 더 이상 존재하지 않습니다.',
  protocolNeededBeforeExport: '내보내기 전에 회의록을 생성하십시오.',
  protocolNeededBeforeSetAside: '섹션을 빼두기 전에 회의록을 생성하십시오.',
  sectionNotSetAside: '해당 섹션을 빼두지 못했습니다.',
  reviewBeforeGeneration: '생성하기 전에 전사본을 검토하십시오.',
  vocabularyUnresolved: '이름과 용어를 확인하지 못했습니다.',

  selectionRequired: '바꿀 텍스트를 선택하십시오.',
  selectionTooLong: '한 번에 바꾸기에는 너무 깁니다. 문서 전체가 아니라 섹션을 선택하십시오.',
  passageNotRewritten: '해당 대목을 고쳐 쓰지 못했습니다.',
  openingNotRead: '회의 도입부를 읽지 못했습니다.',
  providerNeededForPassage: '대목을 고쳐 쓰기 전에 사용 중인 Ollama를 실행하십시오.',
  providerNeededForOpening: '자기소개를 읽기 전에 사용 중인 Ollama를 실행하십시오.',
  providerNeededForCorrections: '이 표기를 확인하기 전에 사용 중인 Ollama를 실행하십시오.',
  providerModelRequired: '설정 → 회의록 생성에서 설치된 Ollama 모델을 선택하십시오.',

  styleNotMigrated: '스타일을 이전하지 못했습니다.',
  termMissing: '해당 용어가 더 이상 존재하지 않습니다.',
  exportFormatInvalid: '올바른 내보내기 형식을 선택하십시오.',
  meetingDateInvalid: '올바른 회의 날짜를 선택하십시오.',
  scopeInvalid: '올바른 적용 범위를 선택하십시오.',
  sourceFileInvalid: '올바른 원본 파일을 선택하십시오.',
  workspaceViewInvalid: '올바른 작업 공간 보기를 선택하십시오.',
  recordingUnreadable: '해당 녹음을 읽지 못했습니다.',
  appearanceNotSaved: '체재를 저장하지 못했습니다.',
  furnitureNotSaved: '머리글과 바닥글을 저장하지 못했습니다.',
  documentOperationFailed: '로컬 문서 작업을 끝내지 못했습니다.',
  providerConfigNotSaved: '회의록 공급자 설정을 저장하지 못했습니다.',
  runtimeConfigNotSaved: '전사 런타임 설정을 저장하지 못했습니다.',
  recorderNotStarted: '녹음기를 시작하지 못했습니다.',
  tracksNotCombined: '녹음의 트랙을 합치지 못했습니다.',
  protocolInvalid: '저장된 회의록이 올바르지 않습니다.',
  protocolNotUtf8: '저장된 회의록이 올바른 UTF-8이 아닙니다.',
  editsNotRecorded: '해당 편집은 기록할 수 없습니다.',

  recordingAlreadyRunning: '이미 다른 회의를 녹음하고 있습니다.',
  presetUnknown: '알려진 전사 품질을 선택하십시오.',
  providerModelNotInstalled: 'Ollama에 이미 설치된 모델을 선택하십시오.',
  diariserPathInvalid: '존재하는 화자 분리 프로그램을 선택하십시오.',
  whisperPathInvalid: '존재하는 whisper.cpp 실행 파일을 선택하십시오.',
  nothingRecording: '녹음 중인 것이 없습니다.',
  revealOnlyOnMac: '폴더 열기는 macOS에서만 연결되어 있습니다. 위의 경로는 정확합니다.',
  privacySettingsOnlyOnMac: '개인 정보 보호 설정 열기는 macOS에서만 연결되어 있습니다.',
  providerNeededForModel: '모델을 선택하기 전에 사용 중인 Ollama를 실행하십시오.',
  settingsNotOpened: '시스템 설정을 열지 못했습니다.',
  presetMissing: '해당 내보내기 서식을 더 이상 사용할 수 없습니다.',
  downloadStopped: '내려받기가 중단되었습니다.',
  coordinatorUnavailable: '가져오기 조정자를 사용할 수 없습니다.',
  taskStopped: '로컬 취소 작업이 중단되었습니다.',
  recorderPermissionsUnknown: '녹음기에 권한 상태를 묻지 못했습니다.',
  recorderStateUnknown: '녹음기가 알 수 없는 상태입니다. LocaLog를 다시 시작하십시오.',
  recordingNotFinished: '녹음을 마치지 못했습니다.',
  replacementNotPrepared: '대체를 준비하지 못했습니다.',
  workspaceNotOpened: '작업 공간 폴더를 열지 못했습니다.',
  settingsPaneUnknown: '그런 설정 화면은 없습니다.',
  meetingBusy: '이 회의는 아직 처리 중입니다. 먼저 그 작업을 취소하십시오.',
  printDialogUnavailable: '이 창에서 인쇄 대화상자를 열지 못했습니다.',

  backupNameUnsafe: '해당 백업 이름은 폴더 이름으로 쓸 수 없습니다.',
  notABackup: '해당 폴더는 LocaLog 백업이 아닙니다. manifest.json이 없습니다.',
  backupPathOutside: (path: string) =>
    `이 백업은 자기 폴더 바깥의 파일(${path})을 가리키고 있어 복원하지 않았습니다.`,
  backupFormatUnknown: (format: string) =>
    `이 백업은 ${format} 형식으로 쓰여 있으며, 이 버전의 LocaLog는 읽을 수 없습니다. 더 새로운 버전이라면 읽을 수 있습니다.`,
  backupDamaged: (what: string) =>
    `이 백업은 불완전하거나 손상되었습니다(${what}). 따라서 아무것도 바꾸지 않았습니다. 현재 작업은 그대로입니다.`,
  backupNameTaken: (name: string) => `해당 폴더에는 이미 「${name}」이(가) 있습니다.`,
  backupIoFailed: (what: string) => `백업을 쓰거나 읽지 못했습니다: ${what}`,
  backupDatabaseFailed: (what: string) => `데이터베이스를 복사하지 못했습니다: ${what}`,

  categoryRequired: '분류를 선택하십시오.',
  meetingLanguageRequired: '회의 언어를 선택하십시오.',
  meetingLanguageInvalid: '올바른 회의 언어를 선택하십시오.',
  meetingInvalid: '올바른 회의를 선택하십시오.',
  projectInvalid: '올바른 프로젝트를 선택하십시오.',
  styleInvalid: '올바른 회의록 스타일을 선택하십시오.',
  sourceRecordingInvalid: '올바른 원본 녹음을 선택하십시오.',
  meetingTitleRequired: '회의 제목을 입력하십시오.',
  projectNameRequired: '프로젝트 이름을 입력하십시오.',
  termRequired: '용어를 입력하십시오.',
  meetingTitleTooLong: '회의 제목이 너무 깁니다.',
  speakerPassCannotRead: (what: string) => `화자 처리가 작업용 오디오를 읽지 못했습니다: ${what}`,
  speakerPassCannotWrite: (what: string) => `화자 처리가 오디오를 쓰지 못했습니다: ${what}`,
  recordingNotStored: (what: string) => `녹음을 저장하지 못했습니다: ${what}`,
  recordingNotRead: (what: string) => `녹음을 읽지 못했습니다: ${what}`,
  modelNotDownloaded: (what: string) => `모델을 내려받지 못했습니다: ${what}`,
  modelNotSaved: (what: string) => `모델을 저장하지 못했습니다: ${what}`,
  ollamaRequestFailed: (what: string) => `Ollama가 로컬 요청을 끝내지 못했습니다: ${what}`,
  recorderStartFailed: (what: string) => `녹음기를 시작하지 못했습니다: ${what}`,

  embeddingsUnrecognisable: '화자 처리가 알아볼 수 있는 음성 특징을 만들지 못했습니다.',
  embeddingsNoDimensions: '해당 음성 특징에는 차원 정보가 없습니다.',
  embeddingsTruncated: '해당 음성 특징이 명시된 길이보다 짧습니다.',
  probeInvalid: '미디어 분석이 올바르지 않은 메타데이터를 반환했습니다.',
  cachePathInvalid: '정규화 캐시 경로가 올바르지 않습니다.',
  normalizerNoOutput: '미디어 준비 과정이 오디오 파일을 만들지 못했습니다.',
  speakerPassNoAudio: '화자 처리가 들을 것이 없습니다.',
  speakerPassTooMuchAudio: '화자 처리가 감당할 수 있는 양보다 많은 오디오를 계획했습니다.',
  recordingEmpty: '녹음이 빈 파일로 저장되었습니다.',
  editsLeaveNothing: '이 편집으로는 녹음이 하나도 남지 않습니다.',
  workingAudioUnreadable: '작업용 오디오가 읽을 수 있는 WAV 파일이 아닙니다.',
  workingAudioNotWav: '작업용 오디오가 WAV 파일이 아닙니다.',
  workingAudioSilent: '작업용 오디오에 소리가 들어 있지 않습니다.',
  workingAudioFormatUnreadable: '작업용 오디오의 형식을 읽을 수 없습니다.',
  workingAudioNoFormat: '작업용 오디오에 형식 정보가 없습니다.',
  condensedAudioTooLarge: '압축한 오디오가 너무 커서 쓸 수 없습니다.',
  combinedPathInvalid: '합친 녹음의 경로가 올바르지 않습니다.',
  modelUnknown: '해당 전사 모델을 알 수 없습니다.',
  downloadCancelled: '내려받기를 취소했습니다.',
  downloadCorrupt: '내려받은 파일이 불완전하거나 손상되어 버렸습니다.',
  ollamaModelGone:
    '선택한 Ollama 모델이 더 이상 설치되어 있지 않습니다. 다른 모델을 선택하고 다시 시도하십시오.',
  ollamaModelChanged:
    '이 작업을 대기열에 넣은 뒤 선택한 Ollama 모델이 바뀌었습니다. 다시 시도해 확인하십시오.',
  ollamaRuntimeChanged:
    '이 작업을 대기열에 넣은 뒤 Ollama 런타임이 바뀌었습니다. 다시 시도해 확인하십시오.',
  responseTooLarge: '로컬 모델의 응답이 안전 한도를 넘어 반영하지 않았습니다.',
  responseIncomplete: '로컬 모델이 완전한 회의록을 내놓기 전에 멈췄습니다.',
  protocolWouldNotFit: (sizes: string) => {
    const [expected, ceiling] = sizes.split('/').map((n) => Number(n).toLocaleString('ko-KR'));
    return `이 회의는 길어서, 그 회의록은 약 ${expected}자가 되며 한 번의 응답(약 ${ceiling}자)에 담기지 않습니다. 이는 응답의 운이 아니라 계산의 문제여서 다시 해도 같은 결과가 나오므로 아무것도 시도하지 않았습니다. 더 간결한 회의록 스타일을 선택하시거나 녹음을 나누십시오.`;
  },
  generationConfigUnreadable:
    '이 작업은 이전 버전의 LocaLog가 준비한 것이어서 읽을 수 없습니다. 아무것도 반영되지 않았고 전사본은 그대로입니다. 생성을 다시 시작하십시오.',
  ollamaUnchecked: 'Ollama는 아직 확인되지 않았습니다.',
  responseUnusable:
    '로컬 모델이 LocaLog가 회의록으로 쓸 수 없는 응답을 내놓았습니다. 아무것도 반영되지 않았고 전사본은 그대로입니다. 모델은 매번 다르게 답하므로 다시 시도하면 되는 경우가 많습니다.',
  recorderMissing: '설치된 녹음기가 없습니다. LocaLog에 포함되어 있으나 이 빌드가 찾지 못했습니다.',
  recorderSilentAboutPermissions: '녹음기가 무엇이 허용되었는지 밝히지 않았습니다.',
  recorderCannotReportPermissions: '이 녹음기는 무엇이 허용되었는지 알릴 수 없습니다.',
  runtimePathsMustBeAbsolute: 'whisper.cpp 실행 파일과 모델에는 절대 경로를 지정하십시오.',
  whisperExecutableMissing: '선택한 whisper.cpp 실행 파일을 찾지 못했습니다.',
  whisperModelMissing: '선택한 whisper.cpp 모델을 찾지 못했습니다.',
  embeddingsVersion: (version: string) =>
    `이 음성 특징은 버전 ${version}이며, 이 빌드는 읽지 못합니다.`,
  recordingTooSmall: (what: string) => `저장된 녹음이 길이에 비해 너무 작습니다(${what}).`,
  workingAudioFormatWrong: (what: string) =>
    `화자 처리에는 16kHz 모노 16비트 오디오가 필요하지만 이것은 ${what}입니다.`,
  notEnoughSpace: (what: string) => `이 모델을 받기에 공간이 부족합니다(${what}).`,

  // en.ts의 설명을 참고하십시오. Rust 쪽이 아직 직접 쓰고 있던 문장들입니다.
  settingInvalid: '해당 실행 설정은 저장할 수 없습니다.',
  meetingTitleRequiredToRecord: '회의에 제목을 붙이십시오. 제목을 가져올 파일이 없습니다.',
  importSourceGone: '이 가져오기를 다시 시도하기 전에 원본 파일을 다시 선택하십시오.',
  termProjectRequired: '이 용어가 속한 프로젝트를 선택하십시오.',
  termAlreadyPresent: '해당 용어는 이미 여기에 있습니다.',
  sourceRecordingRequired: '원본 녹음을 다시 선택하십시오.',
  managedPathInvalid: '저장된 해당 파일의 경로가 올바르지 않습니다.',
  documentChecksumFailed: '저장된 문서가 로컬 무결성 검사를 통과하지 못했습니다.',
  transcriptOutputInvalid: '전사 결과를 LocaLog가 전사본으로 읽을 수 없습니다.',
  speakerCountOutOfRange: '예상 화자 수는 2에서 64 사이여야 합니다.',
  sourceNotCommitted: '전사하기 전에 회의 원본을 확정하십시오.',
  providerNeededForGeneration: '회의록을 생성하기 전에 사용 중인 Ollama를 실행하십시오.',
  exportDestinationInvalid: '올바른 내보내기 위치를 선택하십시오.',
  exportFileExists: '다른 파일 이름을 선택하십시오. 기존 파일을 묻지 않고 덮어쓰지는 않습니다.',
  exportFolderMissing: '선택한 내보내기 폴더를 사용할 수 없습니다.',
  processingBusy: '다른 로컬 작업이 이미 실행 중입니다. 끝날 때까지 기다리거나 먼저 취소하십시오.',
  ffmpegMissingForRecording: '녹음을 마무리하려면 FFmpeg가 필요하지만 찾지 못했습니다.',

  // 설정의 Ollama 행. en.ts의 설명을 참고하십시오.
  ollamaNotRunning: (detail: string) =>
    `사용 중인 Ollama를 실행한 다음 새로 고치십시오.${detail ? ` ${detail}` : ''}`,
  ollamaModelsUnreadable: (detail: string) =>
    `Ollama는 실행 중이지만 어떤 모델이 설치되어 있는지 알려 주지 않았습니다.${detail ? ` ${detail}` : ''}`,
  ollamaReadyNoModel:
    'Ollama가 준비되었습니다. 회의록을 생성할 모델을 설치된 것 중에서 선택하십시오.',
  ollamaModelReady: '선택한 로컬 모델이 준비되었습니다.',
  ollamaSelectedModelMissing:
    '선택한 모델이 설치되어 있지 않습니다. 이미 설치된 다른 모델을 선택하십시오.',
};

export const ko: Strings = {
  locale: 'ko-KR',

  failures,

  /** en.ts의 설명을 참고하십시오. 키는 저장되는 값입니다. */
  meetingLanguages: {
    English: '영어',
    German: '독일어',
    French: '프랑스어',
    Spanish: '스페인어',
    Italian: '이탈리아어',
    Dutch: '네덜란드어',
    Portuguese: '포르투갈어',
    Polish: '폴란드어',
    Danish: '덴마크어',
    Swedish: '스웨덴어',
    Norwegian: '노르웨이어',
    Finnish: '핀란드어',
    Czech: '체코어',
    Turkish: '튀르키예어',
    Japanese: '일본어',
    Korean: '한국어',
    Chinese: '중국어',
    Arabic: '아랍어',
    Ukrainian: '우크라이나어',
  },
  dialog: {
    detectFromRecording: '녹음에서 자동 판별',
    chooseRecording: '회의 녹음 선택',
    audioAndVideo: '오디오와 동영상',
    plainText: '일반 텍스트',
    exportTitle: (title: string) => `${title} 내보내기`,
  },

  settings: {
    memoryReported: (gb: number) => `메모리 ${gb}GB 확인됨`,
    themeAutomatic: '자동',
    themeLight: '밝게',
    themeDark: '어둡게',
    modelSelected: '선택됨',
    useThisModel: '이 모델 사용',
    useModel: '모델 사용',
    catalogueNote:
      '이 목록은 의도적으로 좁게 두었습니다. LocaLog는 몰래 모델을 내려받지 않으며, 임의의 모델 장터를 보여 주지도 않습니다. 런타임, 라이선스, 메모리 사용량, 독일어와 영어 품질을 확인한 뒤에야 선택할 수 있게 됩니다.',
    managedCopiesNote:
      'LocaLog는 가져온 녹음, 준비된 오디오, 전사본, 회의록, 내려받은 모델의 관리용 사본을 응용 프로그램 데이터 폴더에 보관합니다. 내보내기는 지정하신 위치에만 기록됩니다.',
    discoveredRuntime: (path: string) => `찾은 런타임: ${path}`,
    runtimeVersion: (version: string) => `런타임 버전: ${version}`,
    evaluatedIn: (languages: string) => `${languages}에서 평가됨`,
    evaluationPending: '품질 평가는 아직 진행되지 않았습니다',
    otherModelNote:
      '이것은 어떤 로컬 모델을 써 볼지 이미 알고 계신 분을 위한 것입니다. LocaLog가 평가하거나 권하는 모델이 아니며, 런타임과 메모리의 제약은 똑같이 적용됩니다.',
    qualityLead:
      '원하시는 품질을 선택하십시오. LocaLog는 처음에 필요한 것을 내려받아 이 기기에 보관합니다.',
    speakerDiscovery:
      'LocaLog는 포함된 자원이나 시스템 경로에서 화자 분리 런타임을 스스로 찾습니다. 선택 사항이며 전사를 막는 일은 없습니다.',
    noSpeakerRuntime: '이 기기에서는 아직 호환되는 화자 분리 런타임을 찾지 못했습니다.',
    readinessNote:
      '확인에는 시간이 제한된 실행 시험이 포함되므로, 호환되지 않거나 손상된 실행 파일이 사용 가능한 것처럼 표시되지 않습니다.',
    restoreSummary: (name: string, projects: number, meetings: number, version: string) =>
      `${name}에는 프로젝트 ${projects}건과 회의 ${meetings}건이 들어 있습니다(LocaLog ${version}에서 백업).`,
    restoreWarning:
      '복원하면 이 작업 공간의 프로젝트와 회의가 그것들로 바뀝니다. 삭제되는 것은 없으며 지금 있는 것은 옆 폴더에 남지만, LocaLog는 복원된 작업을 보여 주게 되고 종료했다가 다시 열어야 합니다.',
    interfaceLanguage: '화면 언어',
    interfaceLanguageDetail: 'LocaLog 자체의 언어입니다. 각 회의의 언어와는 별개입니다.',
    application: '응용 프로그램',
    title: '설정',
    lead: '실무에 필요한 것을 먼저. 기술적인 세부 사항은 접어 두었습니다.',
    sectionsLabel: '설정 항목',
    sectionGeneral: '일반',
    sectionModels: '모델',
    sectionTranscription: '전사',
    sectionStorage: '저장 위치',
    sectionAppearance: '체재',
    sectionAdvanced: '고급',
    defaultExport: '기본 내보내기',
    defaultExportDetail: '편집기가 먼저 제안하는 형식입니다. 나머지도 한 번에 고를 수 있습니다.',
    defaultExportLabel: '기본 내보내기 형식',
    formatPdf: 'PDF',
    formatWord: 'Word',
    formatMarkdown: 'Markdown',
    formatPlainText: '일반 텍스트',
    defaultForProtocols: '회의록의 기본값',
    chooseOnce: '한 번만 고르시면 계속 작업하실 수 있습니다',
    modelLead:
      'LocaLog는 바꾸시기 전까지 이 모델로 로컬 회의록 초안을 만듭니다. 보통의 흐름에서는 회의마다 모델을 고르실 필요가 없습니다.',
    recommendedForMachine: '이 기기에 권장',
    notInstalledYet: '아직 설치되지 않음',
    baseline: '기준',
    european: '유럽산',
    checkInstalled: '설치된 모델 확인',
    curatedModels: '선별된 회의록 모델',
    downloadModel: (size: string) => `내려받기(${size})`,
    prepareSpeakerSeparation: '화자 구분 준비',
    restoredBackup: (projects: number, meetings: number, previous: string) =>
      `프로젝트 ${projects}개와 회의 ${meetings}개를 복원했습니다. 여기에 있던 것은 삭제하지 않고 ${previous}(으)로 옮겼습니다. 복원한 작업 공간을 쓰려면 LocaLog를 종료했다가 다시 여십시오.`,
    /** en.ts의 설명을 참고하십시오. */
    transcriptionPreset: {
      fast: { name: '빠름', detail: '빠른 초안, 메모리를 가장 적게 씁니다' },
      balanced: { name: '균형', detail: '일상적인 회의에' },
      accurate: { name: '정확', detail: '품질이 가장 좋고 가장 느립니다' },
    },
    downloadingPreset: (name: string) => `${name} 내려받는 중`,
    /** en.ts의 설명을 참고하십시오. */
    modelDescription: {
      'gemma4-12b':
        '측정한 모델 가운데 가장 정확하고 가장 안정적입니다. 세 차례 실행에서 회의에 나온 수치 35개 가운데 27~31개를 지켰고, 그다음 모델은 6개까지 떨어졌습니다. 다만 느립니다. 80분짜리 회의에 약 14분이 걸립니다.',
      'ministral-8b':
        '독일어 회의에서 세 가지 설정으로 측정했고, 그중 하나에서 쓸 만한 회의록을 썼습니다. 나머지는 두 줄짜리 초안과, 마크다운을 요구한 자리에 JSON 문서를 내놓았습니다. 유럽산 후보로 남겨 두었을 뿐, 아직 기준을 대신할 것은 아닙니다.',
      'qwen3.5-4b':
        '측정한 모델 가운데 가장 빠릅니다. 80분짜리 회의에 약 5분이 걸리며, 메모리가 부족할 때의 선택지입니다. 다만 정식 서식이 요구하는 다음 단계 표를 한 번도 만들어 내지 못했습니다.',
      'ministral-3b': '지원하는 Mac 가운데 가장 사양이 낮은 기기를 위한 첫 유럽산 후보입니다.',
      'granite4.1-8b':
        '독일어 회의에서 세 가지 설정으로 측정했으며, 같은 입력에 대해 언급된 수치 35개 가운데 22개, 19개, 6개를 남겼습니다. 말한 내용의 6분의 5를 잃는 실행은 기록을 남기는 도구가 아니므로 권하지 않습니다.',
      'llama-8b': '검증된 Llama 판본이 나올 때를 위한 비교용 자리입니다.',
    },
    modelOrigin: {
      international: '국제 공개 모델',
      european: '유럽산 모델',
    },
    modelLicence: {
      apache2: 'Apache 2.0',
      gemma: 'Gemma 이용 약관',
      modelSpecific: '모델별 조건',
    },
    modelLanguage: {
      de: '독일어',
      en: '영어',
      ja: '일본어',
      more: '그 밖에 다수',
    },
    modelStatus: {
      installed: '설치됨',
      notInstalled: '설치되지 않음',
      plannedCandidate: '예정된 후보',
    },
    modelSizeInstalled: (gb: string) => `설치 시 약 ${gb} GB`,
    modelSizeSmall: '기기에서 돌리는 작은 모델',
    modelSizeLarger: '더 큰 로컬 모델',
    useAnotherModel: '다른 설치된 모델 사용',
    installedModel: '설치된 모델',
    chooseInstalledModel: '설치된 모델 선택',
    useInstalledModel: '설치된 모델 사용',
    conservativeBaseline: '보수적인 8GB 기준을 사용 중',
    transcriptionQuality: '전사 품질',
    cancel: '취소',
    ready: '준비됨',
    remove: '삭제',
    advancedDetails: '고급 설정',
    modelsStoredNote: '모델은 LocaLog의 응용 프로그램 데이터 폴더에 보관되며 쓰기 전에 검증됩니다.',
    whisperExecutable: 'whisper-cli 실행 파일',
    whisperExecutablePlaceholder: '/whisper-cli 까지의 경로',
    chooseFile: '파일 선택',
    whisperNote: 'whisper-server가 아니라 명령줄 전사 바이너리를 선택하십시오.',
    saveRuntime: '런타임 저장',
    detected: (version: string) => `확인됨: ${version}`,
    chooseWhisper: 'whisper-cli 실행 파일 선택',
    speakerDifferentiation: '화자 구분',
    speakerLead:
      '발언 차례를 나누면 누가 언제 말했는지가 기록됩니다. 선택 사항이며 전사를 막지 않고, 이름은 검토 중에 언제든 고칠 수 있습니다.',
    runtimeUnavailable: '이 설치본에서는 런타임을 쓸 수 없습니다',
    optional: '선택 사항',
    checkReadiness: '준비 상태 확인',
    downloadingSpeakerModels: '화자 분리 모델을 내려받는 중',
    speakerRuntimeMissing: '모델은 준비되었으나 이 설치본에는 호환되는 런타임이 없습니다.',
    whereWorkIsKept: '작업이 보관되는 곳',
    workspaceNote:
      'LocaLog는 안의 경로가 계속 유효하도록 이 폴더를 관리하지만, 폴더는 사용자의 것이며 언제든 들여다보실 수 있습니다.',
    showInFinder: 'Finder에서 보기',
    backup: '백업',
    backupLead:
      '모든 것이 이 기기에 남는다는 말은, 기기와 함께 사라질 수도 있다는 뜻입니다. 백업은 평범한 폴더이므로 외장 드라이브 등 안전한 곳에 두십시오.',
    backUpNow: '지금 백업',
    working: '진행 중…',
    backupContents:
      '모든 프로젝트, 회의, 전사본, 회의록과 녹음 자체가 들어갑니다. 두 가지는 일부러 뺐습니다. 사용자의 작업이 아니고 필요할 때 다시 만들 수 있기 때문입니다. 내려받은 모델과 각 녹음의 준비된 사본입니다. 실제 작업 공간에서 재어 보니 그 준비된 오디오만으로 백업의 4분의 3이었습니다.',
    restore: '복원',
    restoreLead:
      '백업을 되돌립니다. 먼저 전체를 검증하며, 지금 있는 것은 삭제하지 않고 옆으로 옮깁니다.',
    chooseBackup: '백업 선택…',
    chooseBackupTitle: 'LocaLog 백업 선택',
    whereToKeepBackup: '백업을 둘 곳',
    replaceWorkspace: '이 작업 공간 바꾸기',
    restoring: '복원 중…',
    archived: '보관됨',
    archivedLead:
      '한쪽으로 치워 둔 프로젝트와 회의입니다. 삭제된 것은 없습니다. 그 아래의 회의, 전사본, 회의록은 모두 여기에 있고 모든 백업에도 들어 있습니다.',
    show: '보기',
    hide: '숨기기',
    nothingArchived: '보관된 것이 없습니다.',
    project: '프로젝트',
    meeting: '회의',
    bringBack: '되돌리기',
    theme: '테마',
    themeFollowing: (theme: string) => `이 Mac의 설정(${theme})을 따릅니다.`,
    themeSetHere: '이 Mac의 설정과 상관없이 여기에서 지정합니다.',
    nextFakeJob: '다음 모의 작업',
    nextFakeJobDetail: '개발 전용 조작입니다. 실패와 재시도 상태를 살펴보기 위한 것입니다.',
    completeNormally: '정상적으로 끝남',
    failOnce: '한 번 실패한 뒤 재시도 허용',
    syntheticNote: '이것은 메모리 안의 합성 런타임에만 영향을 줍니다.',
  },

  project: {
    deleteMeeting: (title: string) => `${title} 삭제`,
    deleteWarning:
      '회의를 삭제하면 그 녹음과 전사본, 모든 회의록 판이 이 기기에서 사라집니다. 되돌릴 수 없습니다.',
    eyebrow: '프로젝트',
    archiveProject: '프로젝트 보관',
    newMeeting: '새 회의',
    meetings: '회의',
    newestFirst: '최신순',
    columnDate: '날짜',
    columnMeeting: '회의',
    columnDuration: '길이',
    columnStatus: '상태',
    archive: '보관',
    delete: '삭제',
    keep: '남기기',
    noMeetings: '아직 회의가 없습니다',
    noMeetingsDetail: '첫 녹음을 가져와 이 프로젝트의 기록을 시작하십시오.',
    importRecording: '녹음 가져오기',
  },

  lifecycle: {
    draft: '초안',
    sourceReady: '전사 준비됨',
    transcriptReady: '전사 완료',
    protocolDraft: '회의록 초안',
    reviewed: '검토됨',
    archived: '보관됨',
  },

  sections: {
    noHeadings: '이 회의록에는 아직 제목이 없어 나열할 것이 없습니다.',
    setAside: '빼두기',
    addSection: '섹션 추가',
    dragHint: '끌거나 화살표 키를 쓰십시오',
    setThisAside: '이 섹션 빼두기',
    putThisBack: '이 섹션 되돌리기',
    moveSection: (title: string) => `${title}을(를) 옮깁니다. 화살표 키를 쓰십시오.`,
    setAsideNamed: (title: string) => `${title} 빼두기`,
    putBackNamed: (title: string) => `${title} 되돌리기`,
    setAsideNote:
      '빼둔 섹션은 문서에서 빠지므로, 읽고 계신 것이 곧 내보내지는 것입니다. 여기에 보관되며 언제든 되돌릴 수 있습니다.',
  },

  jobErrors: {
    interrupted: {
      title: '가져오기가 중단되었습니다',
      detail:
        '관리용 사본이 확정되기 전에 LocaLog가 멈췄습니다. 외부 원본은 그대로이며 안전하게 다시 시도하실 수 있습니다.',
    },
    permission_denied: {
      title: 'LocaLog가 녹음을 읽거나 저장하지 못했습니다',
      detail:
        '선택한 파일과 LocaLog의 로컬 데이터 위치에 대한 접근 권한을 확인한 뒤 다시 시도하십시오. 외부 원본은 바뀌지 않았습니다.',
    },
    insufficient_space: {
      title: '로컬 저장 공간이 부족합니다',
      detail:
        '공간을 확보하고 다시 시도하십시오. 일부만 저장된 녹음을 완료된 것으로 내놓은 적은 없습니다.',
    },
    source_missing: {
      title: '선택한 녹음을 더 이상 찾을 수 없습니다',
      detail:
        '파일을 원래 위치에 되돌리시거나 새로 가져오십시오. 회의는 초안 상태로 안전하게 남아 있습니다.',
    },
    source_reselection_required: {
      title: '녹음을 다시 선택하십시오',
      detail:
        '이 회의는 원본 위치를 보관하지 않던 이전 개발 빌드에서 만들어졌습니다. 계속하시려면 녹음을 다시 선택하십시오. 회의 자체는 보존되었습니다.',
    },
    unsupported_media: {
      title: '이 미디어 형식은 아직 지원되지 않습니다',
      detail: '흔히 쓰는 오디오나 비디오 파일을 선택하십시오. 외부 원본은 바뀌지 않았습니다.',
    },
    empty_source: {
      title: '선택한 녹음이 비어 있습니다',
      detail:
        '오디오나 비디오 데이터가 들어 있는 녹음을 선택하십시오. 비어 있던 외부 파일은 바뀌지 않았습니다.',
    },
    synthetic_failure: {
      title: '개발용 어댑터가 요청대로 멈췄습니다',
      detail:
        '의도된 실패는 판이 확정되기 전에 일어났습니다. 원본과 가장 최근의 안정된 작업은 안전하며 다시 시도하실 수 있습니다.',
    },
    invalid_adapter_output: {
      title: '로컬 출력을 검증하지 못했습니다',
      detail:
        'LocaLog는 그 불완전한 결과를 반영하지 않았습니다. 가장 최근의 안정된 원본과 문서 판은 안전합니다.',
    },
    runtime_missing: {
      title: '로컬 전사 런타임을 선택하십시오',
      detail:
        '설정 → 전사에서 설치된 whisper.cpp 실행 파일을 선택하십시오. LocaLog는 런타임을 내려받지 않습니다.',
    },
    model_missing: {
      title: '로컬 전사 모델을 선택하십시오',
      detail:
        '설정 → 전사에서 이미 사용할 수 있는 whisper.cpp 모델을 선택하십시오. 모델을 내려받거나 바꾸지 않았습니다.',
    },
    runtime_changed: {
      title: '전사 런타임이 바뀌었습니다',
      detail:
        '대기열의 작업이 실행되지 않았습니다. whisper.cpp 실행 파일이 기록된 런타임과 더 이상 맞지 않기 때문입니다. 다시 시도해 현재 런타임을 확인하십시오.',
    },
    model_changed: {
      title: '전사 모델이 바뀌었습니다',
      detail:
        '대기열의 작업이 실행되지 않았습니다. 모델이 기록된 체크섬과 더 이상 맞지 않기 때문입니다. 다시 시도해 현재 모델을 확인하십시오.',
    },
    media_probe_failed: {
      title: '녹음을 살펴보지 못했습니다',
      detail:
        'FFprobe가 설치되어 있는지, 가져온 원본을 아직 읽을 수 있는지 확인하십시오. 원본은 그대로입니다.',
    },
    normalization_failed: {
      title: '녹음을 준비하지 못했습니다',
      detail:
        'FFmpeg가 설치되어 있는지 확인하고 다시 시도하십시오. 준비된 사본은 다시 만들 수 있고 원본은 그대로입니다.',
    },
    transcription_failed: {
      title: '로컬 전사를 끝내지 못했습니다',
      detail:
        'whisper.cpp 런타임이 전사본의 판이 확정되기 전에 멈췄습니다. 그 모델을 확인하고 다시 시도하십시오.',
    },
    transcription_timeout: {
      title: '로컬 전사가 너무 오래 걸렸습니다',
      detail:
        '감시 중이던 전사 과정이 판이 확정되기 전에 중단되었습니다. 녹음과 런타임을 확인한 뒤 다시 시도하십시오.',
    },
    provider_model_missing: {
      title: '선택한 로컬 모델을 사용할 수 없습니다',
      detail:
        '선택한 Ollama 모델이 더 이상 설치되어 있지 않습니다. 설정 → 회의록 생성에서 설치된 모델을 선택하고 다시 시도하십시오.',
    },
    provider_model_changed: {
      title: '로컬 모델이 바뀌었습니다',
      detail:
        '이 작업을 대기열에 넣은 뒤 모델의 체크섬이 바뀌었습니다. 다시 시도해 현재 설치된 모델을 반영하십시오.',
    },
    provider_runtime_changed: {
      title: '로컬 공급자가 바뀌었습니다',
      detail:
        '이 작업을 대기열에 넣은 뒤 Ollama 런타임의 버전이 바뀌었습니다. 다시 시도해 현재 런타임을 반영하십시오.',
    },
    provider_unavailable: {
      title: '로컬 회의록 생성이 연결하지 못했습니다',
      detail:
        '사용 중인 Ollama를 실행하고 다시 시도하십시오. LocaLog는 런타임을 시작하거나 내려받지 않습니다.',
    },
    provider_invalid_output: {
      title: '로컬 모델의 출력을 검증하지 못했습니다',
      detail:
        'LocaLog는 불완전하거나 형식이 어긋난 회의록을 반영하지 않았습니다. 전사본은 안전하며 다시 시도하실 수 있습니다.',
    },
    provider_incomplete_output: {
      title: '로컬 모델의 출력을 검증하지 못했습니다',
      detail:
        'LocaLog는 불완전하거나 형식이 어긋난 회의록을 반영하지 않았습니다. 전사본은 안전하며 다시 시도하실 수 있습니다.',
    },
    provider_response_too_large: {
      title: '로컬 모델의 응답이 너무 컸습니다',
      detail:
        '응답이 LocaLog의 안전 한도를 넘어 반영하지 않았습니다. 더 짧은 전사본이나 다른 로컬 모델로 시도해 보십시오.',
    },
    invalid_transcript_output: {
      title: '전사 출력을 검증하지 못했습니다',
      detail:
        '런타임의 출력이 불완전하거나 형식이 어긋나 LocaLog가 반영하지 않았습니다. 원본은 안전합니다.',
    },
    processing_failed: {
      title: '로컬 처리를 끝내지 못했습니다',
      detail:
        '불완전한 전사본이나 회의록을 완성된 것으로 내놓은 적은 없습니다. 가장 최근의 안정된 작업은 그대로 있으며 다시 시도하실 수 있습니다.',
    },
    unknown: {
      title: '가져오기를 끝내지 못했습니다',
      detail:
        '회의는 초안 상태이고 외부 원본은 바뀌지 않았습니다. 안전하게 다시 시도하실 수 있습니다.',
    },
  },

  jobStages: {
    transcriptSaved: '전사본을 저장했습니다',
    protocolSaved: '회의록을 저장했습니다',
    importComplete: '가져오기 완료 — 원본 그대로',
    processingCancelled: '로컬 처리를 취소했습니다 — 안정된 상태 유지',
    processingInterrupted: '로컬 처리가 중단되었습니다 — 안정된 상태 유지',
    processingFailed: '로컬 처리를 끝내지 못했습니다 — 안정된 상태 유지',

    ready_to_import: '녹음을 가져올 준비가 되었습니다',
    copying: '녹음을 가져오는 중',
    stoppingSafely: '안전하게 멈추는 중',
    temporary_complete: '거의 다 되었습니다',
    finalizing: '녹음을 안전하게 보관하는 중',
    duplicate_confirmation: '이 녹음은 이미 있을 수 있습니다',
    completed: '녹음이 들어왔습니다',
    cancelled: '가져오기 취소됨 — 원본 그대로',
    interrupted: '가져오기 중단됨 — 원본 그대로',
    failed: '가져오기를 끝내지 못했습니다 — 원본 그대로',
    probing_media: '녹음을 살펴보는 중',
    normalizing_audio: '오디오를 준비하는 중',
    output_staged: '안전하게 저장하는 중',

    transcription_queued: '전사할 준비가 되었습니다',
    checking_source: '녹음을 확인하는 중',
    loading_transcription_model: '모델을 불러오는 중',
    transcribing_audio: '전사하는 중',
    separating_speakers: '화자를 가려내는 중',
    validating_transcript: '전사본을 저장하는 중',
    preparing_fake_transcriber: '준비하는 중',
    transcribing_synthetic_segments: '전사 구간을 만드는 중',

    generation_queued: '회의록을 쓸 준비가 되었습니다',
    checking_transcript: '전사본을 확인하는 중',
    resolving_protocol_inputs: '스타일과 용어를 모으는 중',
    condensing_transcript: '회의를 통독하는 중',
    generating_protocol: '회의록 초안을 쓰는 중',
    validating_protocol: '회의록을 저장하는 중',
    reading_introductions: '자기소개를 읽는 중',

    protocol_would_not_fit: '이 회의는 한 번에 담기에는 긴 회의입니다',
    segments_no_subject_claimed: '회의의 일부가 어느 주제에도 들어가지 않았습니다',
    sections_over_their_length: '일부 섹션이 요청보다 길게 나왔습니다',

    finding_subjects: (detail: string) =>
      detail ? `무엇을 이야기했는지 찾는 중 — 구간 ${detail}` : '무엇을 이야기했는지 찾는 중',
    writing_section: (detail: string) =>
      detail ? `${detail}을(를) 쓰는 중` : '회의록을 섹션별로 쓰는 중',
    joining_subjects: (detail: string) =>
      detail ? `관련된 주제를 묶는 중 — ${detail}건` : '관련된 주제를 묶는 중',
    joined_subjects: (detail: string) =>
      detail ? `주제를 묶었습니다 — ${detail}` : '주제를 묶었습니다',
    joining_failed: (detail: string) =>
      detail ? `주제를 묶지 못했습니다 — ${detail}` : '주제를 묶지 못했습니다',

    working: '진행 중',
  },

  stages: {
    label: '회의 단계',
    source: '원본',
    transcript: '전사본',
    protocol: '회의록',
  },

  progress: {
    needsAttention: '확인이 필요합니다',
    backgroundWork: '백그라운드 작업',
    cancellingSafely: '안전하게 취소하는 중…',
    cancel: '취소',
    speakerPassNote:
      '이 과정은 발언 차례를 견주기 위해 녹음 전체를 읽습니다. 긴 녹음은 몇 분이 걸릴 수 있으며 언제든 안전하게 취소하실 수 있습니다.',
    latestRetained: '가장 최근의 안정된 작업 유지',
    originalUnchanged: ' · 외부 원본 그대로',
    retry: '다시 시도',
    importing: '녹음을 가져오는 중',
    transcribing: '전사하는 중',
    generating: '회의록을 생성하는 중',
    separatingSpeakers: '화자를 분리하는 중',
    working: '진행 중…',
    duplicateNote: '같은 내용이 이미 LocaLog에 있습니다. 합치거나 버린 것은 없습니다.',
    cancelImport: '가져오기 취소',
    importAnotherCopy: '다른 사본으로 가져오기',
    chooseSourceAgain: '원본 다시 선택',
    continueImport: '가져오기 계속',
    transcribeAgain: '전사 다시 시작',
    generateAgain: '생성 다시 시작',
  },

  newProject: {
    namesHeading: '이름과 용어',
    namesLead:
      '전사는 한 번도 들어 본 적 없는 이름을 짐작할 수 없습니다. 지금 알려 주시는 것이 이 프로젝트에 쓸 수 있는 가장 값진 1분입니다. 잘못 들린 이름은 그 녹음으로 만드는 모든 회의록에 그대로 되풀이되며, 이후의 어떤 단계로도 되살릴 수 없습니다.',
    namesPeople: '사람',
    namesPeopleHint: '자리에 있을 만한 분, 회의에서 이름이 나올 만한 분.',
    namesOrganisations: '회사와 발주처',
    namesOrganisationsHint: '발주처, 다른 설계자, 납품 업체.',
    namesProject: '이 프로젝트',
    namesProjectHint: '프로젝트, 대지, 건물의 이름.',
    namesTerms: '그 밖에 바르게 적고 싶은 말',
    namesTermsHint: '일반적인 전사가 알지 못할, 이 일에서 쓰는 말.',
    namesNote:
      '쉼표로 구분하십시오. 모두 선택 사항이며 확정된 것은 없습니다. 「이름과 용어」에서 언제든 더하고 고치실 수 있고, 전사본을 검토하며 하신 수정도 여기에 남습니다.',
    creating: '만드는 중…',
    createAndContinue: '만들고 계속',
    afterCreated:
      '회의록 스타일과 이 일에서 쓰는 이름·용어는 프로젝트를 만든 뒤에 설정하실 수 있습니다. 이름에는 1분을 들일 값어치가 있습니다. 전사가 짐작하지 못하는 것이 바로 그것이기 때문입니다.',
    eyebrow: '프로젝트',
    title: '새 프로젝트',
    lead: '회의와 원본이 속하는 실무상의 틀을 만듭니다.',
    defaults: '프로젝트 기본값',
    name: '프로젝트 이름',
    namePlaceholder: '예: 주민 회관 조사',
    description: '설명',
    descriptionOptional: '선택 사항',
    descriptionPlaceholder: '간결한 내부용 설명',
    defaultLanguage: '회의의 기본 언어',
    defaultLanguageDetail: '화면 언어와는 별개입니다.',
    cancel: '취소',
  },

  appearance: {
    font: '글꼴',
    appliesToProject: (project: string) =>
      `${project}의 모든 회의록에 적용되어, 사무소의 문서가 서로 닮아 보이게 합니다. 바뀌는 것은 회의록의 짜임이지 내용이 아닙니다. 내용은 위의 스타일입니다.`,
    bodySize: '본문 크기',
    headingScale: '제목 비율',
    lineSpacing: '줄 간격',
    pageWidth: '페이지 너비',
  },

  record: {
    recordingNow: '녹음 중',
    recordThisMeeting: '이 회의 녹음',
    lead: '실내와 통화를 이 기기에서 서로 다른 트랙에 담습니다. 참석하신 분들의 동의를 받는 일은 사용자의 몫이며, LocaLog가 알 수 있는 것이 아닙니다.',
    notRecording: '녹음하지 않음',
    microphone: '마이크',
    theCall: '통화',
    trackRecording: '녹음 중',
    trackSilent: '아직 무음',
    trackListening: '듣는 중…',
    stopRecording: '녹음 중지',
    finishing: '마무리하는 중…',
    startRecording: '녹음 시작',
    starting: '시작하는 중…',
    backToMeeting: '회의로 돌아가기',
    noRecorder: '이 빌드에는 녹음기가 없습니다. 대신 파일을 가져오십시오.',
    openTheSetting: '설정 열기',
    grantedInSettings: '시스템 설정에서 허용하시면 돌아오시는 즉시 여기에 반영됩니다.',
    callWouldNotRecordTitle: '통화는 녹음되지 않습니다.',
    callWouldNotRecordBody:
      'macOS가 LocaLog에 화면 및 시스템 오디오 녹음을 허용하지 않았습니다. 허용이 없으면 통화 녹음은 오류가 아니라 무음이 되므로, 나중에 알게 되기보다 지금 허용해 두시는 편이 낫습니다. 실내 마이크는 그대로 담깁니다.',
    roomWouldNotRecordTitle: '실내는 녹음되지 않습니다.',
    roomWouldNotRecordBody:
      'LocaLog에 마이크 사용이 거부되었습니다. 위 설정이 허용한다면 통화는 담깁니다.',
    recorderNotesTitle: '녹음기가 요청받은 모든 것을 하지는 못했습니다.',
    stoppedOnItsOwn: '녹음기가 스스로 멈췄습니다. 그때까지 담은 것은 보관되었습니다.',
    quietCall: (seconds: number) =>
      `통화에서 ${seconds}초 동안 아무것도 들어오지 않았습니다. macOS는 화면 및 시스템 오디오 녹음 권한이 없는 응용 프로그램에 오류가 아니라 무음을 줍니다. 회의가 끝난 뒤보다 지금 확인해 두시는 편이 낫습니다.`,
    quietMicrophone: (seconds: number) =>
      `마이크에서 ${seconds}초 동안 아무것도 들어오지 않았습니다. 올바른 입력이 선택되었는지, 다른 프로그램이 쓰고 있지는 않은지 확인하십시오.`,
  },

  meeting: {
    browserPreview: '브라우저 미리보기',
    speakersEstimateNote:
      'LocaLog가 들은 목소리를 묶어 셉니다. 어디까지나 추정이므로, 맞지 않아 보이면 숫자로 바꾸실 수 있습니다.',
    speakersCountNote:
      '대략의 짐작이면 충분합니다. LocaLog가 찾을 목소리의 수입니다. 너무 많으면 한 사람이 둘로 갈리고, 너무 적으면 두 사람이 하나로 묶일 수 있습니다.',
    speakersTogetherNote: '전사본은 화자 이름을 하나로 유지합니다.',
    importInterrupted:
      '관리용 사본이 확정되기 전에 LocaLog가 닫혔습니다. 회의는 초안 상태이며 가져오기를 안전하게 다시 시도하실 수 있습니다.',
    importCancelled:
      '관리용 사본이 취소되었습니다. 회의는 초안 상태이며 외부 파일은 바뀌지 않았습니다.',
    importFailed:
      '관리용 사본을 확정하지 못했습니다. 회의는 초안 상태이며 외부 파일은 바뀌지 않았습니다.',
    importRunning:
      'LocaLog가 이 원본을 자체 관리 저장소로 복사하고 있습니다. 복사가 검증되고 확정된 뒤에야 준비됩니다.',
    sourceStored: '이(가) 이 회의와 함께 안전하게 보관되었습니다. 외부 원본은 바뀌지 않았습니다.',
    sourceSynthetic:
      '이(가) 이 브라우저용 예시 회의에 지정되었습니다. 실제 파일은 복사되지 않았습니다.',
    syntheticFixture: '예시 자료',
    eyebrow: '회의',
    titleLabel: '회의 제목',
    editTitle: '회의 제목 편집',
    languageLabel: '회의 언어',
    changeLanguage: '회의 언어 변경',
    save: '저장',
    saveLanguage: '언어 저장',
    cancel: '취소',
    recordingEyebrow: '녹음',
    nothingRecorded: '아직 녹음된 것이 없습니다',
    recordLead:
      '실내와 통화를 이 기기에서 서로 다른 트랙에 담습니다. 회의가 끝나면 언제든 멈추실 수 있습니다.',
    recordThisMeeting: '이 회의 녹음',
    sourceImport: '원본 가져오기',
    originalUnchanged: '원본은 그대로입니다',
    sourceReady: '원본 준비됨',
    readyToTranscribe: '전사 준비됨',
    managedSource: '관리 중인 원본',
    language: '언어',
    languageHint: '회의별 설정 · 전사 전에 위에서 바꾸십시오',
    preset: '프리셋',
    globalDefault: '전체 기본값',
    notSelected: '선택되지 않음',
    peopleSpeaking: '말하는 사람 수',
    doNotSeparate: '화자를 구분하지 않음',
    separateAndCount: '구분하고, 몇 명인지도 알아내기',
    prepareSpeakers: '화자 분리 준비',
    prepareSpeakersDetail:
      '잠정적인 화자 이름을 붙이려면 검증된 로컬 모델 파일 두 개가 필요합니다. 녹음은 이 기기를 떠나지 않습니다.',
    preparing: (percent: number) => `준비 중 ${percent}%`,
    prepare: '준비',
    prepareWithSize: (size: string) => `준비(${size})`,
    speakerRuntimeMissing:
      '이 설치본에서는 화자 분리 런타임을 쓸 수 없습니다. 전사는 계속할 수 있지만, 고칠 수 있는 일반적인 화자 이름이 쓰입니다.',
    reviewAndTrim: '먼저 녹음을 검토하고 잘라내기',
    trimDetail:
      '— 회의가 시작되기 전의 기다림과 필요 없는 부분을 덜어내십시오. 녹음 자체는 바뀌지 않습니다.',
    gettingReady: '전사를 준비하는 중…',
    useJobControls: '위의 조작을 쓰십시오',
    prepareSpeakersFirst: '먼저 화자 분리를 준비하십시오',
    transcribe: '전사',
    transcriptionFailedToStart: '전사를 시작하지 못했습니다. 다시 시도해 주십시오.',
    transcriptReady: '전사 완료',
    reviewBeforeGeneration: '생성 전에 검토',
    transcriptReadyDetail: '시각이 붙은 전사본이 준비되어 수정과 화자 지정을 하실 수 있습니다.',
    reviewTranscript: '전사본 검토',
    protocolAvailable: '회의록 있음',
    continueInEditor: '편집기에서 계속',
    protocolDetail: '전사본은 현재 회의록 판과 나란히 남아 있습니다.',
    openProtocol: '회의록 열기',
  },

  newMeeting: {
    meetingOverride: '이 회의에만 적용',
    preparing: '준비 중…',
    bringingRecordingIn: '녹음을 가져오는 중…',
    noPerMeetingOverrides: '회의별 개별 설정과 회의마다 이름·용어를 고르는 기능은 아직 없습니다.',
    chosenOnceNote: '전사 품질과 회의록을 쓰는 모델은 설정에서 한 번 고르면 모든 회의에 쓰입니다.',
    titleRecording: '녹음',
    titleImport: '구조화된 가져오기',
    heading: '새 회의',
    leadRecording: '회의에 이름을 붙이고 프로젝트를 고르십시오. 녹음은 다음 화면에서 시작됩니다.',
    leadImport: '녹음을 고르고 내용을 확인하시면, 나머지는 LocaLog가 진행합니다.',
    context: '맥락',
    chooseProject: '프로젝트 선택',
    project: '프로젝트',
    newProject: '새 프로젝트',
    noInbox: '모든 원본은 회의에 속하고, 모든 회의는 프로젝트에 속합니다. 받은 편지함은 없습니다.',
    source: '원본',
    importRecording: '녹음 가져오기',
    originalStays: '원본은 있던 자리에 그대로 남습니다',
    readyToCopy: '이 회의를 확정하시면 복사됩니다',
    letGoToImport: '놓으면 가져옵니다',
    originalStaysShort: '원본은 있던 자리에 그대로 남습니다.',
    dropHere: '여기에 녹음을 놓거나, 눌러서 고르십시오',
    dropDetail:
      'MP3, M4A, WAV, MP4, MOV 등. 원본에는 손대지 않습니다. LocaLog가 자체 저장소로 복사합니다.',
    readyToAssign: '이 회의에 지정할 준비가 되었습니다',
    chooseFile: '오디오나 비디오 파일 선택',
    previewNote: '브라우저 미리보기는 파일을 저장하지 않고 흐름만 보여 줍니다.',
    useDemoRecording: '예시 녹음 사용',
    essentials: '기본 사항',
    meetingInformation: '회의 정보',
    title: '제목',
    titlePlaceholder: '비워 두시면 파일 이름에서 가져옵니다',
    date: '날짜',
    language: '회의 언어',
    protocolStyle: '회의록 스타일',
    projectDefault: '프로젝트 기본값',
    qualityNote: '전사 품질은 설정에서 한 번 고르면 모든 회의에 적용됩니다.',
    advanced: '고급 처리 옵션',
    cancel: '취소',
    createAndRecord: '회의를 만들고 녹음',
    createAndImport: '회의를 만들고 가져오기',
  },

  recordingReview: {
    lead: '전사하기 전에 회의에 필요 없는 부분을 잘라내십시오. 녹음 자체는 바뀌지 않으며, 여기의 모든 조작은 되돌릴 수 있습니다.',
    noPreparedAudio:
      '이 회의에는 아직 검토할 준비된 오디오가 없습니다. 가져오기가 확정되면 쓸 수 있게 됩니다.',
    dragToSelect: '녹음 위를 끌어 구간을 고르시거나, Shift를 누른 채 화살표 키를 쓰십시오.',
    selectedRange: (from: string, to: string) => `${from}부터 ${to}까지 선택했습니다.`,
    eyebrow: '녹음',
    heading: '녹음 검토',
    noAudio: '아직 작업용 오디오가 없습니다',
    waveformLabel: '녹음입니다. 화살표 키로 이동하고, Shift를 누르면 구간을 고를 수 있습니다.',
    keptOf: (kept: string, whole: string) => `${whole} 가운데 ${kept} 유지`,
    startsAt: (time: string) => `${time}에서 시작`,
    endsAt: (time: string) => `${time}에서 끝남`,
    removedSpan: (from: string, to: string) => `${from}부터 ${to}까지 삭제함`,
    startHere: '여기서 시작',
    removeSelection: '선택 구간 제거',
    endHere: '여기서 끝',
    edits: '편집',
    nothingRemoved: '제거한 것이 없습니다. 녹음 전체를 전사합니다.',
    undo: '실행 취소',
    putEverythingBack: '모두 되돌리기',
    untouchedNote: '녹음 자체는 그대로입니다. 이것은 무엇을 쓸지에 대한 지시입니다.',
    undoStartTrim: '앞부분 잘라내기 취소',
    undoEndTrim: '뒷부분 잘라내기 취소',
    putStretchBack: '이 구간 되돌리기',
    next: '다음',
    continueToTranscription: '전사로 넘어가기',
    backToMeeting: '회의로 돌아가기',
  },

  transcript: {
    heardAs: (heard: string) => `「${heard}」(으)로 들렸습니다`,
    askAboutTheRest: '나머지 살펴보기',
    askingAboutTheRest: '문장을 읽는 중…',
    askAboutTheRestNote:
      '몇몇 낱말은 나올 때마다 다르게 잘못 들리므로, 표기를 고쳐도 찾아지지 않습니다. 여기서는 낱말마다 그 문장 안에서 읽고, 이 프로젝트 목록에 있는 이름을 제안합니다. 그 밖의 것은 제안할 수 없으며, 말씀하시기 전까지 아무것도 바꾸지 않습니다.',
    proposedNothing: '더 알아낸 것이 없습니다.',
    proposedNothingNote:
      '보통의 답이고, 좋은 답이기도 합니다. 이 프로젝트에 이미 있는 이름만 제안할 수 있으므로, 지어내기보다 잠자코 있는 것입니다.',
    proposalsHeading: (count: number) => `제안 ${count}건`,
    proposalSuggests: (heard: string, suggested: string) => `${heard} → ${suggested}`,
    spellingsToCheck: (count: number) => `살펴볼 표기 ${count}건`,
    questionedByProtocol: '회의록이 이 낱말을 알아보지 못했습니다',
    autosaveFailed: '자동 저장에 실패했습니다 — 마지막으로 저장된 상태는 그대로입니다',
    correctCount: (count: number) => `${count}건 수정`,
    audioCouldNotLoad: '이 회의의 작업용 오디오를 불러오지 못했습니다.',
    pauseAudio: '일시정지',
    playAudio: '재생',
    saving: '저장하는 중…',
    editsSaved: '편집을 저장했습니다',
    revisionSaved: '전사본의 판을 저장했습니다',
    separationUnavailableHere:
      '이 설치본에서는 화자 분리를 아직 쓸 수 없습니다. 이름을 손수 붙이며 계속하실 수 있습니다.',
    rerunForSeparation: '현재의 화자 분리 결과를 남기려면 이 전사를 다시 실행하십시오.',
    separationUnavailableForRun:
      '이번 실행에서는 화자 분리를 쓸 수 없었습니다. 이름을 손수 붙이며 계속하실 수 있습니다.',
    nothingChangedYet: '아직 바뀐 것이 없습니다',
    readingOpening: '도입부를 읽는 중…',
    readWhoIsHere: '이 회의에 누가 있는지 읽기',
    correcting: '수정하는 중…',
    durationPending: '길이 미정',
    introducedThemselves: (count: number) => `${count}명이 자기소개를 했습니다`,
    noNamesYet: (project: string) => `${project}에는 아직 이름이 없습니다`,
    speltAsHeard:
      '전사가 들은 대로 적은 표기입니다. 잘못된 것을 고쳐 주십시오. 여기에서 고쳐지고 이 프로젝트에 기억됩니다.',
    openingNote:
      '회의는 보통 각자 누구인지 밝히며 시작합니다. 그 대목을 읽으면 이 프로젝트의 이름을 얻게 되는데, 전사가 짐작하지 못하는 것이 바로 그것입니다.',
    foundInPlaces: (count: number) =>
      `${count}곳에서 찾았습니다. 그대로 두어야 할 것은 체크를 해제하십시오.`,
    noneMisheardEveryTime: (count: number) =>
      `나올 때마다 매번 잘못 들린 낱말은 없었습니다. 다른 이유로 불명확하다고 표시된 대목이 ${count}건 남아 있습니다.`,
    nothingFlaggedNote:
      '불명확하다고 표시된 것이 없습니다. 이 기능이 생기기 전에 만든 전사본도 여기에 아무것도 보이지 않으므로, 오래된 전사본은 믿기보다 다시 읽어 보시는 편이 낫습니다.',
    workingAudioLater: '작업용 오디오는 이 회의를 전사한 뒤에 쓸 수 있게 됩니다.',
    recordingEndsNote:
      '회의가 이 뒤로도 이어졌다면 녹음이 그것을 담지 못했고, 회의록에도 들어가지 않습니다.',
    heading: '전사본 검토',
    exportTranscript: '전사본 내보내기…',
    exportLabel: '이 전사본 내보내기',
    asMarkdown: 'Markdown으로',
    asPlainText: '일반 텍스트로',
    reviewDetails: '검토 세부 정보',
    sourceContext: '원본의 맥락',
    seekAudio: '오디오 이동',
    follow: '따라가기',
    followLabel: '재생 중인 대목까지 전사본을 스크롤합니다',
    searchTranscript: '전사본 검색',
    editableTranscript: '편집할 수 있는 전사본',
    removeLine: '이 줄을 전사본에서 제거',
    nothingFlagged: '불명확하다고 표시된 것이 없습니다',
    show: '보기',
    showing: '보는 중',
    onePassage: '불명확한 대목 1건',
    manyPassages: (count: number) => `불명확한 대목 ${count}건`,
    speakerHint: '화자 이름은 출발점입니다. 실제로 말한 분의 이름으로 바꾸십시오.',
    generateProtocol: '회의록 생성',
    review: '검토',
    detailsLabel: '전사본 검토 세부 정보',
    closeInspector: '패널 닫기',
    speakers: '화자',
    whereRecordingStops: '녹음이 끝나는 곳',
    transcriptionInput: '전사 입력',
    language: '언어',
    meetingLanguage: '회의 언어',
    saveLanguage: '언어 저장',
    cancel: '취소',
    changeLanguage: '언어 변경',
    rerunNote: '언어나 전사 설정을 바꾸신 뒤에 쓰십시오. 새 실행은 별도의 판으로 기록됩니다.',
    rerun: '전사 다시 실행',
    rerunPreparing: '새 전사본을 준비하는 중…',
    rerunConfirm: (language: string) =>
      `${language}(으)로 전사를 다시 실행할까요? 새 결과가 확정될 때까지 현재 전사본이 남고, 그 뒤에 이 작업 중인 전사본이 바뀝니다.`,
    whoIsHere: '이 회의에 있는 사람',
    close: '닫기',
    aboutAMinute: '1분쯤 걸립니다. 그동안 다른 작업은 실행할 수 없습니다.',
    unsureNames: '한 번 더 볼 만한 이름',
    whatShouldItSay: '어떻게 적어야 합니까?',
    rememberForProject: '이 프로젝트에 기억해 두어, 다음 회의에서 바르게 적히도록 합니다',
    areAnyNames: '이 가운데 이름이 있습니까? 하나를 고치면 이 전사본이 고쳐지고 기억됩니다.',
    nothingToCheck: '확인할 것이 없습니다',
    correctSpelling: '표기 수정',
    checkWording: '표현 확인',
    checkWords: (words: string) => `${words} 확인`,
    textAt: (time: string) => `${time}의 전사 텍스트`,
    jumpTo: (time: string) => `${time}(으)로 이동`,
    removeLineAt: (time: string) => `${time} 위치의 줄 삭제`,
    renameSpeaker: (speaker: string) => `${speaker} 이름 바꾸기`,
    nameHeardAs: (heard: string) => `${heard}(으)로 들린 이름`,
    protocolStyle: '회의록 스타일',
    audioUnplayable: '이 회의의 작업용 오디오를 재생하지 못했습니다.',
    speakersResolved:
      '발언 차례는 이 기기에서 가려냈습니다. 이름은 잠정적이므로, 누구인지 아실 때에만 바꾸십시오.',
    speakersFailed:
      '이번 실행에서는 화자 분리가 쓸 만한 차례를 내놓지 못했습니다. 전사본은 온전하며 일반적인 이름을 씁니다. 손수 붙이며 계속하실 수 있습니다.',
    speakersUnavailable:
      '이번 실행에서는 화자 분리를 쓸 수 없었습니다. 전사본은 온전하며 일반적인 이름 하나를 씁니다. 손수 바꾸실 수 있습니다.',
    speakersUnknown:
      '이 오래된 전사본에는 화자 분리가 실행되었는지 기록이 없습니다. 일반적인 이름이 붙어 있다는 것이 화자가 한 명이었다는 증거는 아닙니다.',
  },

  library: {
    remove: '제거',
    edit: '편집',
    keep: '남기기',
    notInUseSuffix: ' · 쓰지 않음',
    /** en.ts의 설명을 참고하십시오. 이름을 바꾸기 전까지만 쓰입니다. */
    shippedStyle: {
      'style-formal': {
        name: '정식 회의록',
        description: '논의와 결정, 실행 항목을 짜임새 있게 남기는 형식입니다.',
      },
      'style-working-note': {
        name: '내부 업무 메모',
        description: '내부 프로젝트 팀을 위한 간결한 작업 기록입니다.',
      },
      'style-decision-log': {
        name: '기술 결정 기록',
        description: '대안과 제약, 명시된 결정을 앞세웁니다.',
      },
    },
    copyOf: (name: string) => `${name} (사본)`,
    enterATerm: '용어를 입력하십시오.',
    reading: '읽는 중…',
    editTerm: '용어 편집',
    inUse: '사용 중',
    notInUse: '쓰지 않음',
    instructionsGiven: '모델에 주어지는 지시이며, 주어지는 순서대로 놓여 있습니다',
    asShipped: '(이 스타일이 포함되었을 때 그대로입니다)',
    invariantsNote:
      '이것들은 이 스타일의 일부가 아니며 여기서 편집할 수 없습니다. 애초에 어떤 스타일과도 함께 저장되지 않기 때문입니다. 회의록을 쓸 때마다 모든 회의록에 더해집니다. 아무도 내리지 않은 결정을 적은 문서는 스타일이 다른 회의록이 아니라 잘못된 회의록이기 때문입니다.',
    whichTermsHelp:
      '사람 이름, 회사 이름, 약어가 가장 도움이 됩니다. 흔한 전문 용어는 적어 두지 않아도 대개 바르게 전사됩니다.',
    termsLeadLong:
      '이 일에서 쓰는 사람 이름, 회사 이름, 약어를 더해 두면 바르게 전사됩니다. 실제 80분 회의에서는 프로젝트 이름 자체가 「한 번도 바르지 않음」에서 「언제나 바름」으로 바뀌었습니다.',
    eyebrow: '라이브러리',
    protocolStyles: '회의록 스타일',
    namesAndTerms: '이름과 용어',
    stylesLead:
      '회의록이 무엇을, 어떤 차례로 말하는지입니다. 어떻게 짜이는지가 아닙니다. 그것은 체재이며, 그것이 설명하는 문서 옆 편집기 안에 있습니다.',
    termsLead:
      '전사가 짐작하지 못하는 이름들입니다. 프로젝트, 회사, 사람. 실제 회의에서 재어 보니 여기의 어떤 설정보다도 값어치가 있었습니다.',
    addTerm: '용어 추가',
    saveTerm: '용어 저장',
    stylesUnreadable: '여기서는 스타일을 읽을 수 없습니다.',
    length: '길이',
    name: '이름',
    description: '설명',
    whatItAsksFor: '이 스타일이 요구하는 것',
    addInstruction: '지시 추가',
    removeInstruction: '이 지시 제거',
    checkedOnProtocol: '완성된 회의록에서 확인됩니다',
    alwaysEveryStyle: '언제나, 모든 스타일에서',
    saveStyle: '스타일 저장',
    cancel: '취소',
    delete: '삭제',
    editThisStyle: '이 스타일 편집',
    duplicate: '복제',
    duplicateToEdit: '복제해서 편집',
    shippedStyleNote:
      '함께 제공된 스타일은 그대로 남습니다. 지난해 쓴 회의록을 오늘도 같은 방식으로 쓸 수 있게 하기 위해서입니다. 자신의 것으로 만들려면 복제하십시오.',
    ownershipAutomatic: '지정은 자동입니다.',
    termsScopeNote: '프로젝트의 이름과 용어는 매번 고르지 않아도 그 회의에 적용됩니다.',
    term: '용어',
    spellingAsShown: '나타나야 할 표기',
    category: '분류',
    appliesTo: '적용 범위',
    everyProject: '모든 프로젝트',
    unknownProject: '알 수 없는 프로젝트',
    noTerms: '아직 이름도 용어도 없습니다',
    deleteThisTerm: '이 용어를 삭제할까요?',
    densityFull: '충실한 산문',
    densityPlain: '간결한 서술',
    densityLine: '항목당 한 줄',
    densityFullMeaning: '충실한 산문. 자리에 없던 사람도 논의를 따라갈 수 있습니다.',
    densityPlainMeaning: '간결한 서술. 말해진 것만, 다시 풀어 쓰지 않고.',
    densityLineMeaning: '항목당 한 줄. 기록만 있고 그 주변은 없습니다.',
    categoryPerson: '사람',
    categoryOrganisation: '회사',
    categoryProject: '프로젝트',
    categoryAbbreviation: '약어',
    categoryTechnicalTerm: '전문 용어',
    categoryOther: '기타',
  },

  furniture: {
    header: '머리글',
    footer: '바닥글',
    left: '왼쪽',
    centre: '가운데',
    insertInto: (where: string) => `${where}에 값 넣기`,
    right: '오른쪽',
    insert: '삽입…',
    lineHint:
      '읽히기를 바라는 대로 줄을 쓰시고, 원하는 자리에 값을 넣으십시오. 「페이지 」, 번호, 「 / 12」처럼. 값은 하나의 덩어리여서 통째로 선택되고 통째로 지워집니다.',
    appliesTo: (project: string) =>
      `${project}의 모든 회의록에 적용됩니다. 인쇄된 페이지마다 되풀이되며, 지금 편집하고 계신 문서의 일부가 아닙니다.`,
  },

  shell: {
    breadcrumbMeeting: '회의',
    breadcrumbRecording: '녹음',
    breadcrumbReview: '검토',
    skipToWorkspace: '작업 공간으로 건너뛰기',
    workspace: '작업 공간',
    workspaceFailed: '작업 공간을 열지 못했습니다',
    workspaceFailedDetail: '기존 파일은 바뀌지 않았습니다.',
    tryAgain: '다시 시도',
    preparingWorkspace: '로컬 작업 공간을 준비하는 중…',
    openNavigation: '탐색 열기',

    notSelected: '선택되지 않음',

    jobNeedsDecision: '결정이 필요합니다',
    jobReadyToContinue: '계속할 준비가 되었습니다',
    jobCancelling: '안전하게 취소하는 중',

    formatWordDocument: 'Word 문서',
    formatPlainText: '일반 텍스트',
    exportSaved: (format: string) => `${format} 내보내기를 저장했습니다`,
    exportFailed: (format: string, why: string) => `${format} 내보내기에 실패했습니다: ${why}`,
    exportPrepared: (format: string) => `${format} 내보내기를 준비했습니다`,
    exportNeedsDesktop: (format: string) =>
      `${format} 내보내기에는 데스크톱 응용 프로그램이 필요합니다.`,

    meetingArchived: '회의를 보관했습니다. 설정의 「저장 위치」에 있습니다.',
    projectArchived: '프로젝트를 보관했습니다. 설정의 「저장 위치」에 있습니다.',
    transcriptExported: '전사본을 내보냈습니다',
  },

  protocol: {
    undo: '실행 취소',
    redo: '다시 실행',
    next: '다음',
    blockParagraph: '본문',
    blockHeading1: '제목 1',
    blockHeading2: '제목 2',
    blockHeading3: '제목 3',
    figuresMissingFromRewrite: (count: number) =>
      `원래 대목에 있던 수치 ${count}건이 이 고쳐 쓴 글에는 없습니다`,
    markdownView: 'Markdown 보기',
    documentView: '문서 보기',
    looking: '찾는 중…',
    replaceAll: '모두 바꾸기',
    rewrite: '고쳐 쓰기',
    rewriting: '고쳐 쓰는 중',
    figureMissingFromRewrite: '원래 대목에 있던 수치가 이 고쳐 쓴 글에는 없습니다',
    reviewedRevisionPreserved:
      '검토된 판은 그대로 남습니다. 여기의 작업 중인 편집은 검토되지 않았습니다.',
    thisRevisionReviewed: '바뀌지 않는 바로 이 판이 검토됨으로 표시되었습니다.',
    generatedStaysEditable: '생성된 내용은 계속 검토하고 편집하실 수 있습니다.',
    notFound: '찾지 못했습니다',
    matchCount: (count: number) => `${count}건`,
    replacedCount: (count: number) => ` · ${count}건 바꿈`,
    changesNotYetMade: (count: number) => `${count}건의 변경, 아직 적용되지 않음`,
    compoundNote:
      '대문자로 시작하는 이름은 복합어 안에서도 찾습니다. 단순한 바꾸기가 놓치는 곳이 거기입니다. 읽어 보신 뒤 남기시거나 두십시오.',
    andMore: (count: number) => `그 밖에 ${count}건, 모두 같은 두 형태입니다.`,
    passageGoesAlone:
      '이 대목만 사용 중인 로컬 모델로 갑니다. 숫자, 이름, 날짜는 그대로 돌아와야 합니다. 확인하시고, 그렇지 않으면 되돌리십시오.',
    nothingChangedYet:
      '아직 아무것도 바뀌지 않았습니다. 읽어 보신 뒤 남기시거나 두십시오. 로컬 모델은 잘 고쳐 쓰지만, 그대로 믿을 것은 아닙니다.',
    secondPassNote:
      '사용 중인 모델 자신에게 물은 결과이며, 양쪽으로 틀립니다. 변경을 놓치기도 하고, 문제없는 표현을 짚기도 합니다. 참고이지 판정은 아닙니다.',
    pageEdgesNote:
      '인쇄용 스타일시트가 짜는 대로 잰, 페이지가 끝나는 자리입니다. 제목이나 표는 쪼개지지 않고 통째로 내려가지만 본문은 나뉩니다. 마지막 한두 줄은 프린터가 정하므로, 한 줄 정도의 폭으로 보십시오.',
    transcriptSourceNote:
      '이 회의의 검토된 전사본에서 쓰였습니다. 어느 대목이 어느 문장이 되었는지는 기록되지 않으므로, 아래는 아는 척하지 않고 낱말을 찾습니다. 바꿔 말한 것이라면 아무것도 찾지 못하는데, 그것이 정직한 답입니다.',
    noWordsTogether:
      '이 낱말들은 전사본에서 함께 나타나지 않습니다. 보통은 초안이 자기 말로 옮겼다는 뜻이고, 그럴 수 있습니다. 확인할 곳은 녹음입니다.',
    revisionNote:
      '입력하신 내용은 작업 중인 편집으로 저장되며 판이 되지는 않습니다. 판은 초안이 생성될 때, 요청하실 때, 회의록을 검토됨으로 표시하실 때, 그리고 예전 판을 복원할 때 만들어집니다. 이 목록이 읽을 만한 길이로 남도록 하기 위해서입니다.',
    nothingRewrites:
      '여기에는 사용자의 글을 대신 고쳐 쓰는 것이 없습니다. 초안은 사용자의 것이고, 모든 판이 남습니다.',
    figuresKept: (kept: number, stated: number) => `수치 ${stated}건 가운데 ${kept}건 유지`,
    figuresNote: (stated: number, kept: number) =>
      `회의에서는 수치가 ${stated}건 언급되었고 이 초안은 그 가운데 ${kept}건을 되풀이합니다. 얼마나 들어가야 하는지는 고르신 스타일에 달렸으므로, 이것은 점수가 아니라 살펴보실 거리입니다.`,
    figuresInvented: (count: number) => `회의에서 언급되지 않은 수치가 ${count}건 있습니다`,
    confirmAgainstRecording: '. 녹음과 견주어 확인해 볼 만합니다.',
    tasksUnowned: (count: number) => `담당자가 없는 작업이 ${count}건 있습니다`,
    unownedNote:
      '. 초안은 담당자를 짐작하지 않고 비워 두므로, 회의에서 정해진 그대로일 수도 있습니다. 이름을 넣는 일은 다음 회의보다 지금이 훨씬 쌉니다.',
    editor: '회의록 편집기',
    markdownBacked: 'Markdown 기반',
    noteMissingTableHeading: '다음 단계 표가 없습니다',
    noteMissingTableBody:
      '이 회의록은 세 번 쓰였지만, 어느 판도 합의된 작업과 담당자의 표로 끝나지 않았습니다. 회의에서 합의된 행동은 위의 섹션들에 적혀 있으나 여기에 모여 있지는 않습니다.',
    noteGapsHeading: '이 회의록에 담기지 않은 부분',
    noteOneGap:
      '녹음의 한 구간을 읽지 못했고, 위의 어디에도 적혀 있지 않습니다. 녹음 자체는 온전하며 다시 들으실 수 있습니다.',
    noteSeveralGaps:
      '녹음의 여러 구간을 읽지 못했고, 위의 어디에도 적혀 있지 않습니다. 녹음 자체는 온전하며 그 구간들도 다시 들으실 수 있습니다.',
    documentType: '회의록',
    statusDraft: '초안',
    statusReviewed: '검토됨',
    statusChanged: '검토 후 변경됨',
    fieldProjectName: '프로젝트 이름',
    fieldMeetingTitle: '회의 제목',
    fieldMeetingDate: '회의 날짜',
    fieldDocumentType: '문서 종류',
    fieldProtocolStatus: '상태',
    fieldPageNumber: '페이지 번호',
    fieldPageOfCount: 'n / m 페이지',
    fieldText: '직접 입력',
    showPageBreaks: '쪽 나눔 보기',
    hidePageBreaks: '쪽 나눔 숨기기',
    saving: '저장하는 중…',
    autosaveFailed: '자동 저장에 실패했습니다',
    workingEditsSaved: '작업 중인 편집을 저장했습니다',
    revisionSaved: '판을 저장했습니다',
    editorTools: '도구',
    find: '찾기',
    findInProtocol: '회의록에서 찾기',
    replaceWith: '바꿀 내용',
    makeChanges: '이 변경 적용',
    leaveIt: '그대로 두기',
    zoomOut: '축소',
    zoomIn: '확대',
    insertTable: '표 삽입',
    insertDivider: '구분선 삽입',
    documentMenu: '문서 메뉴',
    clearFormatting: '서식 지우기',
    table: '표',
    blockType: '블록 종류',
    addColumnLeft: '왼쪽에 열 추가',
    addColumnRight: '오른쪽에 열 추가',
    deleteColumn: '이 열 삭제',
    addRowAbove: '위에 행 추가',
    addRowBelow: '아래에 행 추가',
    deleteRow: '이 행 삭제',
    formatting: '서식',
    bold: '굵게',
    italic: '기울임',
    bulletedList: '글머리 기호 목록',
    numberedList: '번호 매기기 목록',
    quotation: '인용',
    askModel: '모델에 다르게 말해 달라고 하기',
    customInstruction: '직접 지시…',
    whatShouldChange: '무엇을 바꿀까요?',
    proposedChange: '제안된 변경',
    proposedReplacement: '제안된 대체',
    proposedRewrite: '제안된 고쳐 쓰기',
    unchanged: '모델이 대목을 그대로 돌려주었습니다.',
    factsMoved: '두 번째 확인에서는 이 사실들이 달라졌다고 봅니다',
    noFactMoved: '두 번째 확인에서는 달라진 사실을 찾지 못했습니다. 놓치는 것도 있습니다.',
    useThis: '이것 사용',
    improveClarity: '더 또렷하게',
    improveClarityInstruction: '이 대목을 더 읽기 좋게 만들어 주십시오.',
    makeFormal: '더 격식 있게',
    makeFormalInstruction: '전문적인 회의록에 쓰이듯 더 격식 있는 문체로 만들어 주십시오.',
    makePlainer: '더 쉽게',
    makePlainerInstruction: '정확함을 잃지 않으면서 표현을 더 쉽고 곧게 만들어 주십시오.',
    shorten: '줄이기',
    shortenInstruction: '이것을 더 적은 말로 말해 주십시오.',
    rewriteUnavailable: '여기서는 고쳐 쓰기를 쓸 수 없습니다.',
    replaceUnavailable: '여기서는 이름 바꾸기를 쓸 수 없습니다.',
    nameNotFound: '그 이름은 이 회의록에 없습니다.',
    protocolMarkdown: '회의록 Markdown',
    protocolLabel: '회의록',
    protocolDetails: '회의록 세부 정보',
    documentDetails: '문서 세부 정보',
    closeInspector: '패널 닫기',
    tabDocument: '문서',
    tabTranscript: '전사본',
    tabHistory: '기록',
    status: '상태',
    createRevision: '판 만들기',
    lineNumber: (line: number) => `${line}행`,
    pageNumber: (page: number) => `${page}쪽`,
    revisionNumber: (ordinal: number) => `${ordinal}판`,
    markReviewed: '검토됨으로 표시',
    style: '스타일',
    sections: '섹션',
    newSection: '새 섹션',
    appearance: '체재',
    editAppearance: '체재 편집',
    headerFooter: '머리글과 바닥글',
    editHeaderFooter: '머리글과 바닥글 편집',
    nothingRepeated: '페이지에 되풀이되는 것이 없습니다',
    presets: '프리셋',
    useOrSavePreset: '프리셋 사용 또는 저장',
    noneSaved: '아직 저장된 것이 없습니다',
    savedCount: (count: number) => `${count}건 저장됨`,
    use: '사용',
    remove: '제거',
    nameThisPreset: '이 프리셋에 이름 지정',
    nameForPreset: '이 프리셋의 이름',
    save: '저장',
    cancel: '취소',
    saveAsPreset: '이 체재와 머리글을 프리셋으로 저장',
    export: '내보내기',
    exportPdf: 'PDF로 내보내기',
    exportWord: 'Word로 내보내기',
    exportMarkdown: 'Markdown으로 내보내기',
    exportPlainText: '일반 텍스트로 내보내기',
    exportNote:
      'PDF는 지금 읽고 계신 문서를, 이 프로젝트가 회의록을 짜는 대로 인쇄한 것입니다. 인쇄 대화상자에서 「PDF로 저장」을 고르십시오.',
    source: '원본',
    findSelectedPassage: '선택한 대목 찾기',
    lookingFor: '찾는 것:',
    openReviewedTranscript: '검토된 전사본 열기',
    whatToCheck: '확인할 것',
    revisions: '판',
    current: '현재',
    restore: '복원',
  },

  sidebar: {
    projects: '프로젝트',
    newProject: '새 프로젝트',
    createProject: '프로젝트 만들기',
    library: '라이브러리',
    protocolStyles: '회의록 스타일',
    namesAndTerms: '이름과 용어',
    settings: '설정',
    recording: '녹음',
    primaryNavigation: '주 탐색',
    closeNavigation: '탐색 닫기',
    openNavigation: '탐색 열기',
    themeFollowingSystem: '시스템 테마를 따릅니다. 항상 밝게로 바꿉니다.',
    themeAlwaysLight: '항상 밝게입니다. 항상 어둡게로 바꿉니다.',
    themeAlwaysDark: '항상 어둡게입니다. 시스템을 따르도록 되돌립니다.',
    themeFollowingShort: '시스템을 따름',
    sidebarWidth: (width: number) => `${width} 픽셀`,
    resizeSidebar: '사이드바 너비를 바꿉니다. 화살표 키로 조정하고 Enter로 되돌립니다.',
    themeAlwaysLightShort: '항상 밝게',
    themeAlwaysDarkShort: '항상 어둡게',

    importNeedsDecision: '가져오기에 결정이 필요합니다',
    needsAttention: '확인이 필요합니다',
    importingRecording: '녹음을 가져오는 중',
    transcribing: '전사하는 중',
    writingProtocol: '회의록을 쓰는 중',
    working: '진행 중',
    workingEllipsis: '진행 중…',
    separatingSpeakers: '화자를 분리하는 중',
    openMeetingNeedingAttention: '확인이 필요한 회의 열기',
    openThisMeeting: '이 회의 열기',
  },

  start: {
    eyebrow: '비공개 회의록을 위한, 이 기기에서 도는 AI',
    title: '회의 시작',
    lead: '오디오나 비디오 파일을 가져오십시오. 회의록이 되기 전에 단계마다 확인하실 수 있습니다.',
    importTitle: '녹음 가져오기',
    importDetail: '프로젝트를 고르고, 모든 것을 맥락 안에 두십시오',
    recordTitle: '회의 녹음',
    recordDetail: '실내와 통화를 이 기기에서 서로 다른 트랙에 담습니다',
    promiseTitle: '회의 관련 작업은 이 기기를 떠나지 않습니다.',
    promiseDetail: 'LocaLog 계정도, 클라우드 서비스도, 원격 측정도 없습니다.',

    setupProviderTitle: '첫 회의록을 만들기 전에 한 가지 더',
    setupProviderBody:
      '전사는 이제 됩니다. 회의록을 쓰려면 이 기기에 언어 모델도 필요하며, 설정에서 준비합니다. 그 전에 녹음을 가져와 전사하는 것은 가능합니다.',
    setupProviderAction: '설정에서 준비하기',
    setupTitle: '첫 전사 전에 한 번만 내려받습니다',
    setupBody: (quality: string, size: string) =>
      `LocaLog는 이 기기에서 전사하므로 모델이 이 기기에 있어야 합니다. ${quality} 품질은 ${size}이며 한 번만 내려받습니다. 녹음을 먼저 가져오셔도 됩니다. 모델이 필요한 때는 전사가 시작될 때이지 그 전이 아닙니다.`,
    setupDownload: (size: string) => `지금 내려받기(${size})`,
    setupCancel: '취소',
    setupAside: '다른 품질과 화자 분리는 설정에 있습니다.',
  },
};
