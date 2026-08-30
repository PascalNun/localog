/**
 * Every word the application says, in Japanese.
 *
 * Typed against English, so this file cannot be missing a key or inventing one.
 *
 * ## Decisions taken once, here, so the whole application reads as one voice
 *
 * **議事録, never a transliteration of "protocol".** 議事録 is the established
 * Japanese object with an expected shape — participants, decisions, action items
 * with owners — which is precisely what this product makes. プロトコル would name
 * a communications protocol and nothing else.
 *
 * **文字起こし for transcription, not 転写.** 転写 is copying in the biological or
 * mechanical sense. 文字起こし is what a Japanese office calls turning speech into
 * text, and it is the word somebody would actually search for.
 *
 * **ですます調 throughout**, matching the German *Sie* and the French *vous*.
 * Written for offices keeping the formal record of meetings. Not 尊敬語 or
 * 謙譲語 beyond what politeness needs — over-formality reads as a machine trying
 * too hard.
 *
 * **会議 for a meeting.** 打ち合わせ is what a design office often says for a
 * working session and would be warmer, but 会議 is the word that belongs beside
 * 議事録 and covers the formal cases too.
 *
 * **録音 for a recording, トラック for a track, 話者 for a speaker.**
 *
 * Japanese has no plural and no article, so the count functions collapse to one
 * form with a counter — 件, 箇所, 個 — chosen for what is being counted.
 */

import type { Strings } from './en';

const failures = {
  missingProject: '選択されたプロジェクトはもう存在しません。',
  missingMeeting: '選択された会議はもう存在しません。',
  missingJob: 'その取り込み処理はもう利用できません。',
  importBusy: '別の録音を取り込み中です。先に完了するか中止してください。',
  unsupportedSchema: (version: string) =>
    `このLocaLogのデータは、より新しい未対応のバージョン（${version}）で作成されています。`,
  storageUnavailable: 'LocaLogはローカルの作業領域にアクセスできませんでした。',

  styleMissing: 'そのスタイルはもう存在しません。',
  styleNameRequired: 'スタイルに名前を付けてください。',
  styleNotSaved: 'スタイルを保存できませんでした。',
  styleUnavailable: '選択された議事録スタイルは利用できません。',
  styleUsedByMeeting: 'このスタイルを使っている会議があります。先にそちらを変更してください。',
  styleUsedByProject:
    'このスタイルを既定にしているプロジェクトがあります。先にそちらを変更してください。',

  presetNameRequired: 'プリセットに名前を付けてください。',
  presetNotSaved: 'プリセットを保存できませんでした。',
  presetBuiltInUndeletable: 'LocaLogに同梱されているプリセットは削除できません。',

  transcriptInvalid: '保存されている文字起こしが不正です。',
  transcriptSegmentMissing: 'その文字起こしの区間はもう存在しません。',
  transcriptTextRequired: '有効な文字起こしのテキストを入力してください。',
  transcriptNeedsSegment: '文字起こしには少なくとも1つの区間が必要です。',
  transcriptSpeakerRequired: '有効な話者名を入力してください。',
  transcriptNotSaved: '文字起こしを保存できませんでした。',
  transcriptNotCommitted: '文字起こしを確定できませんでした。',
  spellingRequired: '有効な表記を入力してください。',

  protocolTextRequired: '有効な議事録のテキストを入力してください。',
  protocolRevisionMissing: '選択された議事録の版はもう存在しません。',
  protocolNeededBeforeExport: '書き出す前に議事録を生成してください。',
  protocolNeededBeforeSetAside: 'セクションを外す前に議事録を生成してください。',
  sectionNotSetAside: 'そのセクションを外せませんでした。',
  reviewBeforeGeneration: '生成の前に文字起こしを確認してください。',
  vocabularyUnresolved: '名前と用語を解決できませんでした。',

  selectionRequired: '変更したいテキストを選択してください。',
  selectionTooLong:
    '一度に変更するには長すぎます。文書全体ではなく、セクションを選択してください。',
  passageNotRewritten: 'その箇所を書き直せませんでした。',
  openingNotRead: '会議の冒頭を読み取れませんでした。',
  providerNeededForPassage: '箇所を書き直す前に、お使いのOllamaを起動してください。',
  providerNeededForOpening: '自己紹介を読み取る前に、お使いのOllamaを起動してください。',
  providerNeededForCorrections: 'これらの表記を確かめる前に、お使いの Ollama を起動してください。',
  providerModelRequired: '設定 → 議事録の生成で、インストール済みのOllamaモデルを選んでください。',

  styleNotMigrated: 'スタイルを移行できませんでした。',
  termMissing: 'その用語はもう存在しません。',
  exportFormatInvalid: '有効な書き出し形式を選んでください。',
  meetingDateInvalid: '有効な会議の日付を選んでください。',
  scopeInvalid: '有効な適用範囲を選んでください。',
  sourceFileInvalid: '有効な元ファイルを選んでください。',
  workspaceViewInvalid: '有効な作業領域の表示を選んでください。',
  recordingUnreadable: 'その録音を読み取れませんでした。',
  appearanceNotSaved: '体裁を保存できませんでした。',
  furnitureNotSaved: 'ヘッダーとフッターを保存できませんでした。',
  documentOperationFailed: 'ローカルでの文書処理を完了できませんでした。',
  providerConfigNotSaved: '議事録プロバイダーの設定を保存できませんでした。',
  runtimeConfigNotSaved: '文字起こしランタイムの設定を保存できませんでした。',
  recorderNotStarted: '録音機能を開始できませんでした。',
  tracksNotCombined: '録音のトラックを結合できませんでした。',
  protocolInvalid: '保存されている議事録が不正です。',
  protocolNotUtf8: '保存されている議事録が正しいUTF-8ではありません。',
  editsNotRecorded: 'それらの編集は記録できません。',

  recordingAlreadyRunning: 'すでに別の会議を録音中です。',
  presetUnknown: '既知の文字起こし品質を選んでください。',
  providerModelNotInstalled: 'Ollamaにインストール済みのモデルを選んでください。',
  diariserPathInvalid: '存在する話者分離プログラムを選んでください。',
  whisperPathInvalid: '存在するwhisper.cppの実行ファイルを選んでください。',
  nothingRecording: '録音は行われていません。',
  revealOnlyOnMac: 'フォルダを開く機能はmacOSのみに対応しています。上のパスは正しいものです。',
  privacySettingsOnlyOnMac: 'プライバシー設定を開く機能はmacOSのみに対応しています。',
  providerNeededForModel: 'モデルを選ぶ前に、お使いのOllamaを起動してください。',
  settingsNotOpened: 'システム設定を開けませんでした。',
  presetMissing: 'その書き出しテンプレートはもう利用できません。',
  downloadStopped: 'ダウンロードが中断しました。',
  coordinatorUnavailable: '取り込みの調整役が利用できません。',
  taskStopped: 'ローカルの中止処理が停止しました。',
  recorderPermissionsUnknown: '録音機能に許可の状態を問い合わせられませんでした。',
  recorderStateUnknown: '録音機能の状態が不明です。LocaLogを再起動してください。',
  recordingNotFinished: '録音を終了できませんでした。',
  replacementNotPrepared: '置き換えを準備できませんでした。',
  workspaceNotOpened: '作業領域のフォルダを開けませんでした。',
  settingsPaneUnknown: 'そのような設定画面はありません。',
  meetingBusy: 'この会議はまだ処理中です。先にそちらを中止してください。',
  printDialogUnavailable: 'このウインドウでは印刷ダイアログを開けませんでした。',

  backupNameUnsafe: 'そのバックアップ名はフォルダ名として使えません。',
  notABackup: 'そのフォルダはLocaLogのバックアップではありません。manifest.jsonがありません。',
  backupPathOutside: (path: string) =>
    `このバックアップは自身のフォルダの外にあるファイル（${path}）を参照しているため、復元しませんでした。`,
  backupFormatUnknown: (format: string) =>
    `このバックアップは形式${format}で書かれており、このバージョンのLocaLogでは読めません。より新しいバージョンなら読めます。`,
  backupDamaged: (what: string) =>
    `このバックアップは不完全または破損しています（${what}）。そのため何も変更していません。現在の作業はそのままです。`,
  backupNameTaken: (name: string) => `そのフォルダにはすでに「${name}」があります。`,
  backupIoFailed: (what: string) => `バックアップを書き込み・読み取りできませんでした：${what}`,
  backupDatabaseFailed: (what: string) => `データベースを複製できませんでした：${what}`,

  categoryRequired: '分類を選んでください。',
  meetingLanguageRequired: '会議の言語を選んでください。',
  meetingLanguageInvalid: '有効な会議の言語を選んでください。',
  meetingInvalid: '有効な会議を選んでください。',
  projectInvalid: '有効なプロジェクトを選んでください。',
  styleInvalid: '有効な議事録スタイルを選んでください。',
  sourceRecordingInvalid: '有効な元の録音を選んでください。',
  meetingTitleRequired: '会議の題名を入力してください。',
  projectNameRequired: 'プロジェクト名を入力してください。',
  termRequired: '用語を入力してください。',
  meetingTitleTooLong: '会議の題名が長すぎます。',
  speakerPassCannotRead: (what: string) => `話者処理が作業用音声を読み取れませんでした：${what}`,
  speakerPassCannotWrite: (what: string) => `話者処理が音声を書き出せませんでした：${what}`,
  recordingNotStored: (what: string) => `録音を保存できませんでした：${what}`,
  recordingNotRead: (what: string) => `録音を読み取れませんでした：${what}`,
  modelNotDownloaded: (what: string) => `モデルをダウンロードできませんでした：${what}`,
  modelNotSaved: (what: string) => `モデルを保存できませんでした：${what}`,
  ollamaRequestFailed: (what: string) => `Ollamaがローカルの要求を完了できませんでした：${what}`,
  recorderStartFailed: (what: string) => `録音機能を開始できませんでした：${what}`,

  embeddingsUnrecognisable: '話者処理は認識できる声の特徴量を生成しませんでした。',
  embeddingsNoDimensions: 'その声の特徴量には次元の記述がありません。',
  embeddingsTruncated: 'その声の特徴量は宣言された長さより短いです。',
  probeInvalid: 'メディアの解析が不正なメタデータを返しました。',
  cachePathInvalid: '正規化キャッシュのパスが不正です。',
  normalizerNoOutput: 'メディアの準備処理は音声ファイルを生成しませんでした。',
  speakerPassNoAudio: '話者処理が聴くものがありません。',
  speakerPassTooMuchAudio: '話者処理が扱える量を超える音声を計画しました。',
  recordingEmpty: '録音は空のファイルとして保存されました。',
  editsLeaveNothing: 'この編集では録音が何も残りません。',
  workingAudioUnreadable: '作業用音声は読み取れるWAVファイルではありません。',
  workingAudioNotWav: '作業用音声はWAVファイルではありません。',
  workingAudioSilent: '作業用音声に音が入っていません。',
  workingAudioFormatUnreadable: '作業用音声の形式が読み取れません。',
  workingAudioNoFormat: '作業用音声に形式の記述がありません。',
  condensedAudioTooLarge: '圧縮した音声が大きすぎて書き出せません。',
  combinedPathInvalid: '結合した録音のパスが不正です。',
  modelUnknown: 'その文字起こしモデルは認識できません。',
  downloadCancelled: 'ダウンロードを中止しました。',
  downloadCorrupt: 'ダウンロードが不完全または破損していたため破棄しました。',
  ollamaModelGone:
    '選択されたOllamaモデルはもうインストールされていません。別のモデルを選んでやり直してください。',
  ollamaModelChanged:
    'この処理を待ち行列に入れたあとで、選択されたOllamaモデルが変わりました。やり直して解決してください。',
  ollamaRuntimeChanged:
    'この処理を待ち行列に入れたあとで、Ollamaランタイムが変わりました。やり直して解決してください。',
  responseTooLarge: 'ローカルモデルの応答が安全上の上限を超えたため、取り込みませんでした。',
  responseIncomplete: 'ローカルモデルは完全な議事録を返す前に停止しました。',
  protocolWouldNotFit: (sizes: string) => {
    const [expected, ceiling] = sizes.split('/').map((n) => Number(n).toLocaleString('ja-JP'));
    return `この会議は長く、その議事録はおよそ${expected}文字になるため、1回の応答（およそ${ceiling}文字）には収まりません。これは応答の当たり外れではなく計算上の問題で、やり直しても同じ結果になるため、何も試していません。より簡潔な議事録スタイルを選ぶか、録音を分割してください。`;
  },
  generationConfigUnreadable:
    'この処理は以前のバージョンのLocaLogが用意したもので、読み取れません。何も取り込んでおらず、文字起こしはそのままです。生成をやり直してください。',
  ollamaUnchecked: 'Ollamaはまだ確認されていません。',
  responseUnusable:
    'ローカルモデルは、LocaLogが議事録として使えない応答を返しました。何も取り込んでおらず、文字起こしはそのままです。モデルは毎回異なる答えを返すため、やり直すとうまくいくことがよくあります。',
  generationRuntimeReady: 'この端末で議事録を書く準備ができています。',
  generationModelNotDownloaded: '議事録を書くためのモデルは、まだダウンロードされていません。',
  generationRuntimeMissing:
    '議事録を書くための実行環境が見つかりません。LocaLog に同梱されていますが、このビルドからは見つけられません。',
  generationRuntimeNoPort:
    'ローカルの実行環境に割り当てられる空きポートがありませんでした。いくつかアプリを閉じて、もう一度お試しください。',
  generationRuntimeNotStarted: 'ローカルの実行環境を起動できませんでした。',
  generationRuntimeNeverReady:
    'ローカルの実行環境は起動しましたが、準備が整いませんでした。とても大きなモデルを遅いディスクから読み込むと時間がかかります。再び起きる場合は、小さいモデルを選んでください。',
  generationRuntimeStopped:
    'ローカルの実行環境が起動中に停止しました。モデルのファイルが不完全かもしれません。削除してから、もう一度ダウンロードしてください。',
  recorderMissing:
    '録音機能がインストールされていません。LocaLogには同梱されていますが、このビルドでは見つかりません。',
  recorderSilentAboutPermissions: '録音機能は何が許可されているかを答えませんでした。',
  recorderCannotReportPermissions: 'この録音機能は何が許可されているかを報告できません。',
  runtimePathsMustBeAbsolute: 'whisper.cppの実行ファイルとモデルには絶対パスを指定してください。',
  whisperExecutableMissing: '選択されたwhisper.cppの実行ファイルが見つかりませんでした。',
  whisperModelMissing: '選択されたwhisper.cppのモデルが見つかりませんでした。',
  embeddingsVersion: (version: string) =>
    `この声の特徴量はバージョン${version}で、このビルドでは読めません。`,
  recordingTooSmall: (what: string) => `保存された録音は長さのわりに小さすぎます（${what}）。`,
  workingAudioFormatWrong: (what: string) =>
    `話者処理には16kHzモノラル16ビットの音声が必要ですが、これは${what}です。`,
  notEnoughSpace: (what: string) => `このモデルには空き容量が足りません（${what}）。`,

  // en.ts の注記を参照。Rust 側がまだ自分で書いていた文です。
  settingInvalid: 'その実行時の設定は保存できません。',
  meetingTitleRequiredToRecord:
    '会議にタイトルを付けてください。タイトルを取れるファイルがありません。',
  importSourceGone: 'この読み込みをやり直す前に、元のファイルをもう一度選んでください。',
  termProjectRequired: 'この用語が属するプロジェクトを選んでください。',
  termAlreadyPresent: 'その用語はすでにここにあります。',
  sourceRecordingRequired: '元の録音をもう一度選んでください。',
  managedPathInvalid: '保存されているそのファイルへのパスが不正です。',
  documentChecksumFailed: '保存された文書がローカルの整合性チェックを通りませんでした。',
  transcriptOutputInvalid: '文字起こしの結果を、LocaLog は文字起こしとして読み取れませんでした。',
  speakerCountOutOfRange: '想定する話者の人数は 2 〜 64 の範囲で指定してください。',
  sourceNotCommitted: '文字起こしの前に、会議の元データを確定してください。',
  providerNeededForGeneration: '議事録を生成する前に、お使いの Ollama を起動してください。',
  exportDestinationInvalid: '有効な書き出し先を選んでください。',
  exportFileExists:
    '別のファイル名を選んでください。既存のファイルを断りなく上書きすることはありません。',
  exportFolderMissing: '選ばれた書き出し先のフォルダが利用できません。',
  processingBusy: '別のローカル処理がすでに実行中です。終わるまで待つか、先に中止してください。',
  ffmpegMissingForRecording: '録音を仕上げるには FFmpeg が必要ですが、見つかりませんでした。',

  // 設定の Ollama の行。en.ts の注記を参照。
  ollamaNotRunning: (detail: string) =>
    `お使いの Ollama を起動してから、再確認してください。${detail ? ` ${detail}` : ''}`,
  ollamaModelsUnreadable: (detail: string) =>
    `Ollama は動いていますが、どのモデルが入っているかを返しませんでした。${detail ? ` ${detail}` : ''}`,
  ollamaReadyNoModel:
    'Ollama の準備ができました。議事録を生成するモデルを、インストール済みのものから選んでください。',
  ollamaModelReady: '選ばれているローカルモデルの準備ができています。',
  ollamaSelectedModelMissing:
    '選ばれているモデルはインストールされていません。すでに入っている別のモデルを選んでください。',
};

export const ja: Strings = {
  locale: 'ja-JP',

  failures,

  /** en.ts の注記を参照。キーは保存される値です。 */
  meetingLanguages: {
    English: '英語',
    German: 'ドイツ語',
    French: 'フランス語',
    Spanish: 'スペイン語',
    Italian: 'イタリア語',
    Dutch: 'オランダ語',
    Portuguese: 'ポルトガル語',
    Polish: 'ポーランド語',
    Danish: 'デンマーク語',
    Swedish: 'スウェーデン語',
    Norwegian: 'ノルウェー語',
    Finnish: 'フィンランド語',
    Czech: 'チェコ語',
    Turkish: 'トルコ語',
    Japanese: '日本語',
    Korean: '韓国語',
    Chinese: '中国語',
    Arabic: 'アラビア語',
    Ukrainian: 'ウクライナ語',
  },
  dialog: {
    detectFromRecording: '録音から判定する',
    chooseRecording: '会議の録音を選ぶ',
    audioAndVideo: '音声と動画',
    plainText: 'テキスト',
    exportTitle: (title: string) => `${title} を書き出す`,
  },

  settings: {
    memoryReported: (gb: number) => `メモリ${gb}GBを検出`,
    themeAutomatic: '自動',
    themeLight: 'ライト',
    themeDark: 'ダーク',
    modelSelected: '選択中',
    useThisModel: 'このモデルを使う',
    useModel: 'モデルを使う',
    catalogueNote:
      'この一覧は意図して絞ってあります。LocaLogは黙ってモデルをダウンロードしませんし、任意のモデル市場を並べることもしません。ランタイム、ライセンス、メモリ使用量、ドイツ語と英語での品質を確認したものだけが選べるようになります。',
    managedCopiesNote:
      'LocaLogは、取り込んだ録音、準備した音声、文字起こし、議事録、ダウンロードしたモデルの管理用の複製を、アプリケーションデータのフォルダに保管します。書き出しは指定された場所にのみ書き込まれます。',
    discoveredRuntime: (path: string) => `検出したランタイム：${path}`,
    runtimeVersion: (version: string) => `ランタイムのバージョン：${version}`,
    evaluatedIn: (languages: string) => `${languages}で評価済み`,
    evaluationPending: '品質の評価はまだ行われていません',
    otherModelNote:
      'これは、どのローカルモデルを試したいかすでに分かっている方のためのものです。LocaLogは評価も推奨もしておらず、ランタイムとメモリの制約は同じように当てはまります。',
    qualityLead:
      'ご希望の品質を選んでください。LocaLogは必要なものを初回にダウンロードし、この端末に保管します。',
    speakerDiscovery:
      'LocaLogは同梱のリソースかシステムのパスから、話者分離のランタイムを自動で見つけます。任意の機能であり、文字起こしを妨げることはありません。',
    noSpeakerRuntime: 'この端末では、対応する話者分離のランタイムがまだ見つかっていません。',
    readinessNote:
      '確認には時間を区切った起動テストも含まれるため、互換性のない実行ファイルや壊れた実行ファイルが利用可能として示されることはありません。',
    restoreSummary: (name: string, projects: number, meetings: number, version: string) =>
      `${name}にはプロジェクト${projects}件と会議${meetings}件が入っています（LocaLog ${version}からのバックアップ）。`,
    restoreWarning:
      '復元すると、この作業領域のプロジェクトと会議がそちらに置き換わります。削除は行われず、今あるものは隣のフォルダに残りますが、LocaLogは復元した内容を表示するようになり、いったん終了して開き直す必要があります。',
    interfaceLanguage: '画面の言語',
    interfaceLanguageDetail: 'LocaLog自体の言語です。各会議の言語とは別のものです。',
    application: 'アプリケーション',
    title: '設定',
    lead: 'まず実務上の選択を。技術的な詳細は畳んだままにしてあります。',
    sectionsLabel: '設定の項目',
    sectionGeneral: '一般',
    sectionModels: 'モデル',
    sectionTranscription: '文字起こし',
    sectionStorage: '保存先',
    sectionAppearance: '体裁',
    sectionAdvanced: '詳細',
    defaultExport: '既定の書き出し',
    defaultExportDetail: 'エディタが最初に示す形式です。ほかの形式もワンクリックで選べます。',
    defaultExportLabel: '既定の書き出し形式',
    formatPdf: 'PDF',
    formatWord: 'Word',
    formatMarkdown: 'Markdown',
    formatPlainText: 'テキスト',
    defaultForProtocols: '議事録の既定',
    chooseOnce: '一度選べば、あとは作業を続けられます',
    modelLead:
      'LocaLogは、変更しないかぎりこのモデルでローカルの議事録の下書きを作ります。通常の流れでは、会議ごとにモデルを選ぶ必要はありません。',
    recommendedForMachine: 'この端末におすすめ',
    notInstalledYet: 'まだ未インストール',
    baseline: '基準',
    european: '欧州製',
    checkInstalled: 'インストール済みのモデルを確認',
    curatedModels: '選定済みの議事録モデル',
    downloadModel: (size: string) => `ダウンロード（${size}）`,
    prepareSpeakerSeparation: '話者の分離を準備する',
    restoredBackup: (projects: number, meetings: number, previous: string) =>
      `プロジェクト ${projects} 件と会議 ${meetings} 件を復元しました。ここにあったものは削除せず ${previous} へ移しています。復元した作業領域を使うには、LocaLog を終了してから開き直してください。`,
    /** en.ts の注記を参照。 */
    transcriptionPreset: {
      fast: { name: '高速', detail: '下書きを手早く。メモリの消費が最も少ない' },
      balanced: { name: '標準', detail: '日々の会議に' },
      accurate: { name: '高精度', detail: '品質が最も高く、最も遅い' },
    },
    downloadingPreset: (name: string) => `${name} をダウンロード中`,
    /** en.ts の注記を参照。 */
    modelDescription: {
      'gemma4-12b':
        '測定したモデルのなかで最も正確で、最も安定しています。3 回の実行で、会議に出てきた 35 個の数値のうち 27 〜 31 を残しました。次点は 6 まで落ちています。速度は遅く、80 分の会議でおよそ 14 分かかります。',
      'ministral-8b':
        'ドイツ語の会議で 3 通りの設定を試し、そのうち 1 つで使える議事録を書きました。残りは 2 行の書きかけと、Markdown を求めたところに JSON を返したものです。欧州製の候補として残していますが、基準に代わるものではまだありません。',
      'qwen3.5-4b':
        '測定したなかで最も速く、80 分の会議でおよそ 5 分です。メモリが少ないときの選択肢になります。ただし、形式的なスタイルが求める「次の一手」の表を一度も出しませんでした。',
      'ministral-3b': '対応するなかで最も非力な Mac に向けた、はじめての欧州製の候補です。',
      'granite4.1-8b':
        'ドイツ語の会議で 3 通りの設定を試し、同じ入力に対して、述べられた 35 個の数値のうち 22 個、19 個、6 個を残しました。話された内容の六分の五を落とす実行は記録を残す道具ではないため、おすすめしません。',
      'llama-8b': '検証済みの Llama が出たときのための、比較用の枠です。',
    },
    modelOrigin: {
      international: '国際的なオープンモデル',
      european: '欧州製のモデル',
    },
    modelLicence: {
      apache2: 'Apache 2.0',
      gemma: 'Gemma の利用条件',
      modelSpecific: 'モデルごとの条件',
    },
    modelLanguage: {
      de: 'ドイツ語',
      en: '英語',
      ja: '日本語',
      more: 'ほか多数',
    },
    modelStatus: {
      installed: 'インストール済み',
      notInstalled: '未インストール',
      plannedCandidate: '今後の候補',
    },
    modelSizeInstalled: (gb: string) => `インストール時およそ ${gb} GB`,
    modelSizeSmall: '端末向けの小さなモデル',
    modelSizeLarger: '大きめのローカルモデル',
    useAnotherModel: '別のインストール済みモデルを使う',
    installedModel: 'インストール済みモデル',
    chooseInstalledModel: 'インストール済みモデルを選ぶ',
    useInstalledModel: 'インストール済みモデルを使う',
    conservativeBaseline: '控えめな8GBの基準を使用中',
    transcriptionQuality: '文字起こしの品質',
    cancel: 'キャンセル',
    ready: '準備完了',
    remove: '削除',
    advancedDetails: '詳細',
    modelsStoredNote:
      'モデルはLocaLogのアプリケーションデータのフォルダに保管され、使用前に検証されます。',
    whisperExecutable: 'whisper-cliの実行ファイル',
    whisperExecutablePlaceholder: '/whisper-cli までのパス',
    chooseFile: 'ファイルを選ぶ',
    whisperNote: 'whisper-serverではなく、コマンドラインの文字起こしバイナリを選んでください。',
    saveRuntime: 'ランタイムを保存',
    detected: (version: string) => `検出：${version}`,
    chooseWhisper: 'whisper-cliの実行ファイルを選ぶ',
    speakerDifferentiation: '話者の区別',
    speakerLead:
      '話者の切り替わりを分けると、誰がいつ話したかが記録されます。任意の機能で、文字起こしを妨げることはなく、名前は確認中いつでも編集できます。',
    runtimeUnavailable: 'このインストールではランタイムを利用できません',
    optional: '任意',
    checkReadiness: '準備状況を確認',
    downloadingSpeakerModels: '話者分離モデルをダウンロード中',
    speakerRuntimeMissing:
      'モデルは準備できていますが、このインストールには対応するランタイムがありません。',
    whereWorkIsKept: '作業の保存先',
    workspaceNote:
      'LocaLogは中のパスが有効であり続けるようにこのフォルダを管理しますが、フォルダはあなたのもので、いつでも中を見ていただけます。',
    showInFinder: 'Finderで表示',
    backup: 'バックアップ',
    backupLead:
      'すべてがこの端末に残るということは、端末とともに失われうるということでもあります。バックアップはただのフォルダですので、外付けドライブなど安全な場所に置いてください。',
    backUpNow: '今すぐバックアップ',
    working: '処理中…',
    backupContents:
      'すべてのプロジェクト、会議、文字起こし、議事録、そして録音そのものが含まれます。2つだけ意図して除いてあります。どちらもあなたの作業ではなく、必要になれば作り直せるものだからです。ダウンロードしたモデルと、各録音の準備済みの複製です。実際の作業領域で測ったところ、その準備済み音声だけでバックアップの4分の3を占めていました。',
    restore: '復元',
    restoreLead: 'バックアップを戻します。先に全体を検証し、今あるものは削除せず脇へ移します。',
    chooseBackup: 'バックアップを選ぶ…',
    chooseBackupTitle: 'LocaLogのバックアップを選ぶ',
    whereToKeepBackup: 'バックアップの保存先',
    replaceWorkspace: 'この作業領域を置き換える',
    restoring: '復元中…',
    archived: 'アーカイブ済み',
    archivedLead:
      '脇へ置いたプロジェクトと会議です。何も削除されていません。その下の会議、文字起こし、議事録はすべてここにあり、すべてのバックアップにも入っています。',
    show: '表示',
    hide: '隠す',
    nothingArchived: 'アーカイブされたものはありません。',
    project: 'プロジェクト',
    meeting: '会議',
    bringBack: '戻す',
    theme: 'テーマ',
    themeFollowing: (theme: string) => `このMacの設定（${theme}）に従います。`,
    themeSetHere: 'このMacの設定にかかわらず、ここで指定します。',
    nextFakeJob: '次の疑似処理',
    nextFakeJobDetail: '開発用の操作です。失敗と再試行の表示を確認するためのものです。',
    completeNormally: '正常に完了する',
    failOnce: '一度失敗し、そのあと再試行できる',
    syntheticNote: 'これはメモリ上の合成ランタイムにのみ影響します。',
  },

  project: {
    deleteMeeting: (title: string) => `${title} を削除`,
    deleteWarning:
      '会議を削除すると、その録音、文字起こし、すべての議事録の版がこの端末から消えます。元に戻せません。',
    eyebrow: 'プロジェクト',
    archiveProject: 'プロジェクトをアーカイブ',
    newMeeting: '新しい会議',
    meetings: '会議',
    newestFirst: '新しい順',
    columnDate: '日付',
    columnMeeting: '会議',
    columnDuration: '長さ',
    columnStatus: '状態',
    archive: 'アーカイブ',
    delete: '削除',
    keep: '残す',
    noMeetings: 'まだ会議がありません',
    noMeetingsDetail: '最初の録音を取り込んで、このプロジェクトの記録を始めてください。',
    importRecording: '録音を取り込む',
  },

  lifecycle: {
    draft: '下書き',
    sourceReady: '文字起こし可能',
    transcriptReady: '文字起こし完了',
    protocolDraft: '議事録の下書き',
    reviewed: '確認済み',
    archived: 'アーカイブ済み',
  },

  sections: {
    noHeadings: 'この議事録にはまだ見出しがないため、一覧するものがありません。',
    setAside: '脇へ置く',
    addSection: 'セクションを追加',
    dragHint: 'ドラッグするか、矢印キーを使ってください',
    setThisAside: 'このセクションを脇へ置く',
    putThisBack: 'このセクションを戻す',
    moveSection: (title: string) => `${title}を移動します。矢印キーを使ってください。`,
    setAsideNamed: (title: string) => `${title}を脇へ置く`,
    putBackNamed: (title: string) => `${title}を戻す`,
    setAsideNote:
      '脇へ置いたセクションは文書から外れるため、読んでいるものがそのまま書き出されます。ここに保管され、いつでも戻せます。',
  },

  jobErrors: {
    interrupted: {
      title: '取り込みが中断しました',
      detail:
        '管理用の複製が確定する前にLocaLogが停止しました。外部の原本はそのままで、安全にやり直せます。',
    },
    permission_denied: {
      title: 'LocaLogは録音を読み取り・保存できませんでした',
      detail:
        '選んだファイルと、LocaLogのローカルデータの場所へのアクセス権を確認してからやり直してください。外部の原本は変更されていません。',
    },
    insufficient_space: {
      title: 'ローカルの空き容量が足りません',
      detail:
        '空き容量を確保してやり直してください。途中までの録音を完了として扱ったことはありません。',
    },
    source_missing: {
      title: '選んだ録音はもう見つかりません',
      detail:
        'ファイルを元の場所に戻すか、新しく取り込み直してください。会議は下書きのまま安全に残っています。',
    },
    source_reselection_required: {
      title: '録音をもう一度選んでください',
      detail:
        'この会議は、元ファイルの場所を保持しない以前の開発版で作成されました。続けるには録音をもう一度選んでください。会議自体は残っています。',
    },
    unsupported_media: {
      title: 'この形式にはまだ対応していません',
      detail: '一般的な音声・動画ファイルを選んでください。外部の原本は変更されていません。',
    },
    empty_source: {
      title: '選んだ録音は空です',
      detail:
        '音声または動画のデータが入った録音を選んでください。空の外部ファイルは変更されていません。',
    },
    synthetic_failure: {
      title: '開発用アダプタが指示どおり停止しました',
      detail:
        '意図的な失敗は、版が確定する前に発生しました。元ファイルと直近の安定した作業は無事で、やり直せます。',
    },
    invalid_adapter_output: {
      title: 'ローカルの出力を検証できませんでした',
      detail:
        'LocaLogはその不完全な結果を取り込みませんでした。直近の安定した元ファイルと文書の版は無事です。',
    },
    runtime_missing: {
      title: 'ローカルの文字起こしランタイムを選んでください',
      detail:
        '設定 → 文字起こしで、インストール済みのwhisper.cppの実行ファイルを選んでください。LocaLogはランタイムをダウンロードしません。',
    },
    model_missing: {
      title: 'ローカルの文字起こしモデルを選んでください',
      detail:
        '設定 → 文字起こしで、すでに利用できるwhisper.cppのモデルを選んでください。モデルのダウンロードや変更は行っていません。',
    },
    runtime_changed: {
      title: '文字起こしランタイムが変わりました',
      detail:
        '待ち行列の処理は実行されませんでした。whisper.cppの実行ファイルが記録されたランタイムと一致しないためです。やり直して現在のランタイムを解決してください。',
    },
    model_changed: {
      title: '文字起こしモデルが変わりました',
      detail:
        '待ち行列の処理は実行されませんでした。モデルが記録されたチェックサムと一致しないためです。やり直して現在のモデルを解決してください。',
    },
    media_probe_failed: {
      title: '録音を解析できませんでした',
      detail:
        'FFprobeがインストールされているか、取り込んだ元ファイルがまだ読めるかを確認してください。原本はそのままです。',
    },
    normalization_failed: {
      title: '録音を準備できませんでした',
      detail:
        'FFmpegがインストールされているか確認してやり直してください。準備済みの複製は作り直せますし、原本はそのままです。',
    },
    transcription_failed: {
      title: 'ローカルの文字起こしを完了できませんでした',
      detail:
        'whisper.cppのランタイムは、文字起こしの版が確定する前に停止しました。そのモデルを確認してやり直してください。',
    },
    transcription_timeout: {
      title: 'ローカルの文字起こしに時間がかかりすぎました',
      detail:
        '監視下の文字起こし処理は、版が確定する前に停止されました。録音とランタイムを確認してやり直してください。',
    },
    provider_model_missing: {
      title: '選択されたローカルモデルが利用できません',
      detail:
        '選択されたOllamaモデルはもうインストールされていません。設定 → 議事録の生成でインストール済みのモデルを選び、やり直してください。',
    },
    provider_model_changed: {
      title: 'ローカルモデルが変わりました',
      detail:
        'この処理を待ち行列に入れたあとで、モデルのチェックサムが変わりました。やり直して、現在インストールされているモデルを取り込んでください。',
    },
    provider_runtime_changed: {
      title: 'ローカルのプロバイダーが変わりました',
      detail:
        'この処理を待ち行列に入れたあとで、Ollamaランタイムのバージョンが変わりました。やり直して現在のランタイムを取り込んでください。',
    },
    provider_unavailable: {
      title: 'ローカルの議事録生成が接続できませんでした',
      detail:
        'お使いのOllamaを起動してやり直してください。LocaLogはランタイムの起動もダウンロードも行いません。',
    },
    provider_invalid_output: {
      title: 'ローカルモデルの出力を検証できませんでした',
      detail:
        'LocaLogは不完全または不正な議事録を取り込みませんでした。文字起こしは無事で、やり直せます。',
    },
    provider_incomplete_output: {
      title: 'ローカルモデルの出力を検証できませんでした',
      detail:
        'LocaLogは不完全または不正な議事録を取り込みませんでした。文字起こしは無事で、やり直せます。',
    },
    provider_response_too_large: {
      title: 'ローカルモデルの応答が大きすぎました',
      detail:
        '応答がLocaLogの安全上の上限を超えたため、取り込みませんでした。より短い文字起こしか、別のローカルモデルでお試しください。',
    },
    invalid_transcript_output: {
      title: '文字起こしの出力を検証できませんでした',
      detail:
        'ランタイムの出力が不完全または不正だったため、LocaLogは取り込みませんでした。元ファイルは無事です。',
    },
    processing_failed: {
      title: 'ローカルの処理を完了できませんでした',
      detail:
        '不完全な文字起こしや議事録が完成として示されたことはありません。直近の安定した作業は利用でき、やり直せます。',
    },
    unknown: {
      title: '取り込みを完了できませんでした',
      detail: '会議は下書きのままで、外部の原本は変更されていません。安全にやり直せます。',
    },
  },

  jobStages: {
    transcriptSaved: '文字起こしを保存しました',
    protocolSaved: '議事録を保存しました',
    importComplete: '取り込み完了 — 原本はそのまま',
    processingCancelled: 'ローカルの処理を中止しました — 安定した状態は保持',
    processingInterrupted: 'ローカルの処理が中断しました — 安定した状態は保持',
    processingFailed: 'ローカルの処理を完了できませんでした — 安定した状態は保持',

    ready_to_import: '録音を取り込む準備ができました',
    copying: '録音を取り込んでいます',
    stoppingSafely: '安全に停止しています',
    temporary_complete: 'もう少しです',
    finalizing: '録音を安全に保管しています',
    duplicate_confirmation: 'この録音はすでにあるかもしれません',
    completed: '録音が入りました',
    cancelled: '取り込みを中止しました — 原本はそのまま',
    interrupted: '取り込みが中断しました — 原本はそのまま',
    failed: '取り込みを完了できませんでした — 原本はそのまま',
    probing_media: '録音を調べています',
    normalizing_audio: '音声を準備しています',
    output_staged: '安全に保存しています',

    transcription_queued: '文字起こしの準備ができました',
    checking_source: '録音を確認しています',
    loading_transcription_model: 'モデルを読み込んでいます',
    transcribing_audio: '文字起こし中',
    separating_speakers: '話者を聞き分けています',
    validating_transcript: '文字起こしを保存しています',
    preparing_fake_transcriber: '準備しています',
    transcribing_synthetic_segments: '文字起こしの区間を作っています',

    generation_queued: '議事録を書く準備ができました',
    checking_transcript: '文字起こしを確認しています',
    resolving_protocol_inputs: 'スタイルと用語をまとめています',
    condensing_transcript: '会議を通して読んでいます',
    generating_protocol: '議事録の下書きを書いています',
    validating_protocol: '議事録を保存しています',
    reading_introductions: '自己紹介を読んでいます',

    protocol_would_not_fit: 'この会議は1回の処理に収まらない長さです',
    segments_no_subject_claimed: '会議の一部がどの議題にも入りませんでした',
    sections_over_their_length: '一部のセクションが指定より長くなりました',

    finding_subjects: (detail: string) =>
      detail ? `何が話されたかを探しています — 箇所 ${detail}` : '何が話されたかを探しています',
    writing_section: (detail: string) =>
      detail ? `${detail}を書いています` : '議事録をセクションごとに書いています',
    joining_subjects: (detail: string) =>
      detail ? `関連する議題をまとめています — ${detail}件` : '関連する議題をまとめています',
    joined_subjects: (detail: string) =>
      detail ? `議題をまとめました — ${detail}` : '議題をまとめました',
    joining_failed: (detail: string) =>
      detail ? `議題をまとめられませんでした — ${detail}` : '議題をまとめられませんでした',

    working: '処理中',
  },

  stages: {
    label: '会議の段階',
    source: '元ファイル',
    transcript: '文字起こし',
    protocol: '議事録',
  },

  progress: {
    needsAttention: '確認が必要です',
    backgroundWork: 'バックグラウンドの処理',
    cancellingSafely: '安全に中止しています…',
    cancel: '中止',
    speakerPassNote:
      'この処理は、声の切り替わりを比べるために録音全体を読みます。長い録音では数分かかることがあり、いつでも安全に中止できます。',
    latestRetained: '直近の安定した状態を保持',
    originalUnchanged: ' · 外部の原本はそのまま',
    retry: 'やり直す',
    importing: '録音を取り込み中',
    transcribing: '文字起こし中',
    generating: '議事録を生成中',
    separatingSpeakers: '話者を分離中',
    working: '処理中…',
    duplicateNote: '同じ内容がすでにLocaLogに保存されています。統合も破棄もしていません。',
    cancelImport: '取り込みを中止',
    importAnotherCopy: '別の複製として取り込む',
    chooseSourceAgain: '元ファイルを選び直す',
    continueImport: '取り込みを続ける',
    transcribeAgain: '文字起こしをやり直す',
    generateAgain: '生成をやり直す',
  },

  newProject: {
    namesHeading: '名前と用語',
    namesLead:
      '文字起こしは、一度も聞いたことのない名前を推測できません。今それを教えることが、このプロジェクトに使えるもっとも有益な1分です。聞き違えられた名前は、その録音から作られるすべての議事録に同じまま現れ、あとのどの工程でも取り戻せません。',
    namesPeople: '人',
    namesPeopleHint: '同席しそうな方、会議で名前が挙がりそうな方。',
    namesOrganisations: '会社・発注者',
    namesOrganisationsHint: '発注者、他の設計者、納入業者。',
    namesProject: 'このプロジェクト',
    namesProjectHint: 'プロジェクト、敷地、建物の呼び名。',
    namesTerms: 'ほかに正しく書きたい語',
    namesTermsHint: '一般的な文字起こしでは知らない、この仕事で使う語。',
    namesNote:
      'カンマで区切ってください。すべて任意で、決定的なものではありません。「名前と用語」でいつでも追加・修正できますし、文字起こしの確認中に行った修正もここに残ります。',
    creating: '作成中…',
    createAndContinue: '作成して続ける',
    afterCreated:
      '議事録のスタイルと、この仕事で使う名前や用語は、プロジェクトを作ったあとで設定できます。名前には1分の価値があります。文字起こしが推測できないのは、それだからです。',
    eyebrow: 'プロジェクト',
    title: '新しいプロジェクト',
    lead: '会議と元ファイルが属する、実務上のまとまりを作ります。',
    defaults: 'プロジェクトの既定値',
    name: 'プロジェクト名',
    namePlaceholder: '例：公民館の調査',
    description: '説明',
    descriptionOptional: '任意',
    descriptionPlaceholder: '簡潔な内部向けの説明',
    defaultLanguage: '会議の既定の言語',
    defaultLanguageDetail: '画面の言語とは別のものです。',
    cancel: 'キャンセル',
  },

  appearance: {
    font: '書体',
    appliesToProject: (project: string) =>
      `${project}のすべての議事録に適用され、事務所の文書の見た目がそろいます。変えるのは議事録の組み方であって、内容ではありません。内容は上のスタイルです。`,
    bodySize: '本文の大きさ',
    headingScale: '見出しの比率',
    lineSpacing: '行間',
    pageWidth: 'ページ幅',
  },

  record: {
    recordingNow: '録音中',
    recordThisMeeting: 'この会議を録音',
    lead: '室内と通話を、この端末で別々のトラックに収めます。同席する方の同意を得るのはあなたの役目で、LocaLogには知りようがありません。',
    notRecording: '録音していません',
    microphone: 'マイク',
    theCall: '通話',
    trackRecording: '録音中',
    trackSilent: 'ここまで無音',
    trackListening: '聞いています…',
    stopRecording: '録音を停止',
    finishing: '仕上げています…',
    startRecording: '録音を開始',
    starting: '開始しています…',
    backToMeeting: '会議に戻る',
    noRecorder: 'このビルドには録音機能がありません。代わりにファイルを取り込んでください。',
    openTheSetting: '設定を開く',
    grantedInSettings: 'システム設定で許可すると、戻った時点でここに反映されます。',
    callWouldNotRecordTitle: '通話は録音されません。',
    callWouldNotRecordBody:
      'macOSがLocaLogに画面とシステム音声の収録を許可していません。許可がないと、通話の録音はエラーではなく無音になります。あとで気づくより、今許可しておくほうが確実です。室内のマイクは収録されます。',
    roomWouldNotRecordTitle: '室内は録音されません。',
    roomWouldNotRecordBody:
      'LocaLogはマイクの使用を拒否されています。上の設定が許していれば、通話のほうは収録されます。',
    recorderNotesTitle: '録音機能は求められたことすべてを行えませんでした。',
    stoppedOnItsOwn: '録音機能が自ら停止しました。そこまでに収めたものは保存されています。',
    quietCall: (seconds: number) =>
      `通話から${seconds}秒間なにも届いていません。macOSは、画面とシステム音声の収録を許可されていないアプリケーションにエラーではなく無音を渡します。会議のあとで気づくより、今確認しておくほうが確実です。`,
    quietMicrophone: (seconds: number) =>
      `マイクから${seconds}秒間なにも届いていません。正しい入力が選ばれているか、ほかのアプリが占有していないかを確認してください。`,
  },

  meeting: {
    browserPreview: 'ブラウザのプレビュー',
    speakersEstimateNote:
      'LocaLogは聞こえた声をまとめて数えます。あくまで推定ですので、違うと感じたら数を指定して置き換えられます。',
    speakersCountNote:
      'だいたいの見当で構いません。LocaLogが探す声の数になります。多すぎると1人が2人に分かれ、少なすぎると2人が1人にまとまることがあります。',
    speakersTogetherNote: '文字起こしは話者名を1つのままにします。',
    importInterrupted:
      '管理用の複製が確定する前にLocaLogが閉じられました。会議は下書きのままで、取り込みは安全にやり直せます。',
    importCancelled:
      '管理用の複製を中止しました。会議は下書きのままで、外部のファイルは変更されていません。',
    importFailed:
      '管理用の複製を確定できませんでした。会議は下書きのままで、外部のファイルは変更されていません。',
    importRunning:
      'LocaLogはこの元ファイルを自身の管理領域へ複製しています。複製の検証と確定が終わってから利用できるようになります。',
    sourceStored: 'はこの会議とともに安全に保管されています。外部の原本は変更されていません。',
    sourceSynthetic:
      'はこのブラウザ用のデモ会議に割り当てられています。実際のメディアファイルは複製されていません。',
    syntheticFixture: 'デモ用データ',
    eyebrow: '会議',
    titleLabel: '会議の題名',
    editTitle: '会議の題名を編集',
    languageLabel: '会議の言語',
    changeLanguage: '会議の言語を変更',
    save: '保存',
    saveLanguage: '言語を保存',
    cancel: 'キャンセル',
    recordingEyebrow: '録音',
    nothingRecorded: 'まだ何も録音されていません',
    recordLead:
      '室内と通話を、この端末で別々のトラックに収めます。会議が終わったらいつでも止められます。',
    recordThisMeeting: 'この会議を録音',
    sourceImport: '元ファイルの取り込み',
    originalUnchanged: '原本はそのままです',
    sourceReady: '元ファイルの準備完了',
    readyToTranscribe: '文字起こし可能',
    managedSource: '管理下の元ファイル',
    language: '言語',
    languageHint: '会議ごとの設定 · 文字起こしの前に上で変更してください',
    preset: 'プリセット',
    globalDefault: '全体の既定',
    notSelected: '未選択',
    peopleSpeaking: '話す人数',
    doNotSeparate: '話者を区別しない',
    separateAndCount: '区別して、人数も推定する',
    prepareSpeakers: '話者分離を準備',
    prepareSpeakersDetail:
      '暫定的な話者名を付けるには、検証済みのローカルモデルファイルが2つ必要です。録音はこの端末から出ません。',
    preparing: (percent: number) => `準備中 ${percent}%`,
    prepare: '準備',
    prepareWithSize: (size: string) => `準備（${size}）`,
    speakerRuntimeMissing:
      'このインストールでは話者分離のランタイムを利用できません。文字起こしは続けられますが、編集できる一般的な話者名が使われます。',
    reviewAndTrim: '先に録音を確認して切り詰める',
    trimDetail:
      '— 会議が始まる前の待ち時間や、必要のない部分を取り除けます。録音そのものは変更されません。',
    gettingReady: '文字起こしの準備をしています…',
    useJobControls: '上の操作をお使いください',
    prepareSpeakersFirst: '先に話者分離を準備してください',
    transcribe: '文字起こし',
    transcriptionFailedToStart: '文字起こしを開始できませんでした。もう一度お試しください。',
    transcriptReady: '文字起こし完了',
    reviewBeforeGeneration: '生成の前に確認',
    transcriptReadyDetail: '時刻付きの文字起こしができました。修正と話者の割り当てを行えます。',
    reviewTranscript: '文字起こしを確認',
    protocolAvailable: '議事録があります',
    continueInEditor: 'エディタで続ける',
    protocolDetail: '文字起こしは現在の議事録の版と並んで残ります。',
    openProtocol: '議事録を開く',
  },

  newMeeting: {
    meetingOverride: 'この会議だけの設定',
    preparing: '準備中…',
    bringingRecordingIn: '録音を取り込んでいます…',
    noPerMeetingOverrides:
      '会議ごとの上書き設定と、会議ごとの名前・用語の選択にはまだ対応していません。',
    chosenOnceNote:
      '文字起こしの品質と議事録を書くモデルは設定で一度選べば、すべての会議で使われます。',
    titleRecording: '録音',
    titleImport: '構造化された取り込み',
    heading: '新しい会議',
    leadRecording: '会議に名前を付け、プロジェクトを選んでください。録音は次の画面で始まります。',
    leadImport: '録音を選び、内容を確認すれば、あとはLocaLogが進めます。',
    context: '文脈',
    chooseProject: 'プロジェクトを選ぶ',
    project: 'プロジェクト',
    newProject: '新しいプロジェクト',
    noInbox:
      'すべての元ファイルは会議に属し、すべての会議はプロジェクトに属します。受信箱はありません。',
    source: '元ファイル',
    importRecording: '録音を取り込む',
    originalStays: '原本はそのままの場所に残ります',
    readyToCopy: 'この会議を確定すると複製されます',
    letGoToImport: '離すと取り込みます',
    originalStaysShort: '原本はそのままの場所に残ります。',
    dropHere: 'ここに録音をドロップするか、クリックして選んでください',
    dropDetail:
      'MP3、M4A、WAV、MP4、MOVなど。原本には手を触れません。LocaLogは自身の管理領域へ複製します。',
    readyToAssign: 'この会議に割り当てる準備ができました',
    chooseFile: '音声または動画のファイルを選ぶ',
    previewNote: 'ブラウザのプレビューは、ファイルを保存せずに流れを示します。',
    useDemoRecording: 'デモ用の録音を使う',
    essentials: '基本',
    meetingInformation: '会議の情報',
    title: '題名',
    titlePlaceholder: '空欄ならファイル名から取ります',
    date: '日付',
    language: '会議の言語',
    protocolStyle: '議事録スタイル',
    projectDefault: 'プロジェクトの既定',
    qualityNote: '文字起こしの品質は設定で一度選べば、すべての会議に適用されます。',
    advanced: '詳細な処理オプション',
    cancel: 'キャンセル',
    createAndRecord: '会議を作成して録音',
    createAndImport: '会議を作成して取り込む',
  },

  recordingReview: {
    lead: '文字起こしの前に、会議に必要のない部分を切り落とせます。録音そのものは変更されません。ここでの操作はすべて元に戻せます。',
    noPreparedAudio:
      'この会議には確認できる準備済みの音声がまだありません。取り込みが確定すると使えるようになります。',
    dragToSelect: '録音の上をドラッグして範囲を選ぶか、Shiftを押しながら矢印キーを使ってください。',
    selectedRange: (from: string, to: string) => `${from}から${to}までを選択しました。`,
    eyebrow: '録音',
    heading: '録音を確認',
    noAudio: 'まだ作業用音声がありません',
    waveformLabel: '録音です。矢印キーで移動し、Shiftを押しながらで範囲を選べます。',
    keptOf: (kept: string, whole: string) => `${whole} のうち ${kept} を残しました`,
    startsAt: (time: string) => `${time} から`,
    endsAt: (time: string) => `${time} まで`,
    removedSpan: (from: string, to: string) => `${from} から ${to} を削除`,
    startHere: 'ここから始める',
    removeSelection: '選択範囲を取り除く',
    endHere: 'ここで終える',
    edits: '編集',
    nothingRemoved: '何も取り除いていません。録音全体を文字起こしします。',
    undo: '元に戻す',
    putEverythingBack: 'すべて戻す',
    untouchedNote: '録音そのものはそのままです。これは何を使うかの指示です。',
    undoStartTrim: '冒頭の切り詰めを元に戻す',
    undoEndTrim: '末尾の切り詰めを元に戻す',
    putStretchBack: 'この範囲を戻す',
    next: '次へ',
    continueToTranscription: '文字起こしへ進む',
    backToMeeting: '会議に戻る',
  },

  transcript: {
    heardAs: (heard: string) => `「${heard}」と聞き取られました`,
    askAboutTheRest: '残りを調べる',
    askingAboutTheRest: '文を読んでいます…',
    askAboutTheRestNote:
      'いくつかの語は毎回ちがう形で聞き違えられるため、表記を直しても見つかりません。ここでは一語ずつその文のなかで読み、このプロジェクトの一覧にある名前を提案します。それ以外は提案できませんし、あなたが指示するまで何も変えません。',
    proposedNothing: 'それ以上は認識されませんでした。',
    proposedNothingNote:
      'これが普通の答えで、良い答えでもあります。提案できるのはこのプロジェクトにすでにある名前だけなので、勝手に作り出すより黙っているのです。',
    proposalsHeading: (count: number) => `提案 ${count}件`,
    proposalSuggests: (heard: string, suggested: string) => `${heard} → ${suggested}`,
    spellingsToCheck: (count: number) => `確認したい表記 ${count}件`,
    questionedByProtocol: '議事録はこの語を認識できませんでした',
    autosaveFailed: '自動保存に失敗しました — 最後に保存された状態は無事です',
    correctCount: (count: number) => `${count}件を修正`,
    audioCouldNotLoad: 'この会議の作業用音声を読み込めませんでした。',
    pauseAudio: '一時停止',
    playAudio: '再生',
    saving: '保存中…',
    editsSaved: '編集を保存しました',
    revisionSaved: '文字起こしの版を保存しました',
    separationUnavailableHere:
      'このインストールでは話者分離をまだ利用できません。手作業で名前を付けて進められます。',
    rerunForSeparation: '現在の話者分離の結果を記録するには、この文字起こしをやり直してください。',
    separationUnavailableForRun:
      'この処理では話者分離を利用できませんでした。手作業で名前を付けて進められます。',
    nothingChangedYet: 'まだ何も変えていません',
    readingOpening: '冒頭を読んでいます…',
    readWhoIsHere: 'この会議に誰がいるかを読み取る',
    correcting: '修正中…',
    durationPending: '長さは未確定',
    introducedThemselves: (count: number) => `${count}名が自己紹介しました`,
    noNamesYet: (project: string) => `${project}にはまだ名前がありません`,
    speltAsHeard:
      '文字起こしが聞き取ったとおりの表記です。誤っているものを直してください。ここで修正され、このプロジェクトに記憶されます。',
    openingNote:
      '会議はたいてい、各自が名乗るところから始まります。そこを読むと、このプロジェクトの名前が手に入ります。文字起こしが推測できないのは、それだからです。',
    foundInPlaces: (count: number) =>
      `${count}箇所で見つかりました。そのままにすべきものはチェックを外してください。`,
    noneMisheardEveryTime: (count: number) =>
      `出てくるたびに毎回聞き違えられた語はありませんでした。別の理由で不明瞭とされた箇所が${count}件あります。`,
    nothingFlaggedNote:
      '不明瞭として記録されたものはありません。この機能ができる前に作られた文字起こしでもここには何も出ませんので、古い文字起こしは信用せず読み直すほうが確実です。',
    workingAudioLater: '作業用音声は、この会議の文字起こしが済むと使えるようになります。',
    recordingEndsNote:
      '会議がこの先も続いていた場合、録音はそれを捉えておらず、議事録にも入りません。',
    heading: '文字起こしの確認',
    exportTranscript: '文字起こしを書き出す…',
    exportLabel: 'この文字起こしを書き出す',
    asMarkdown: 'Markdownで',
    asPlainText: 'テキストで',
    reviewDetails: '確認の詳細',
    sourceContext: '元ファイルの文脈',
    seekAudio: '音声を移動',
    follow: '追従',
    followLabel: '再生中の箇所まで文字起こしをスクロールする',
    searchTranscript: '文字起こしを検索',
    editableTranscript: '編集できる文字起こし',
    removeLine: 'この行を文字起こしから取り除く',
    nothingFlagged: '不明瞭とされたものはありません',
    show: '表示',
    showing: '表示中',
    onePassage: '不明瞭な箇所 1件',
    manyPassages: (count: number) => `不明瞭な箇所 ${count}件`,
    speakerHint: '話者名は出発点です。実際に話した方の名前に置き換えてください。',
    generateProtocol: '議事録を生成',
    review: '確認',
    detailsLabel: '文字起こしの確認の詳細',
    closeInspector: 'パネルを閉じる',
    speakers: '話者',
    whereRecordingStops: '録音が終わるところ',
    transcriptionInput: '文字起こしの入力',
    language: '言語',
    meetingLanguage: '会議の言語',
    saveLanguage: '言語を保存',
    cancel: 'キャンセル',
    changeLanguage: '言語を変更',
    rerunNote:
      '言語や文字起こしの設定を変えたあとにお使いください。新しい処理は別の版として記録されます。',
    rerun: '文字起こしをやり直す',
    rerunPreparing: '新しい文字起こしを準備しています…',
    rerunConfirm: (language: string) =>
      `${language}で文字起こしをやり直しますか。新しい結果が確定するまで現在の文字起こしは残り、そのあとこの作業中の文字起こしが置き換わります。`,
    whoIsHere: 'この会議にいる人',
    close: '閉じる',
    aboutAMinute: '1分ほどかかります。その間はほかの処理を実行できません。',
    unsureNames: 'もう一度見ておきたい名前',
    whatShouldItSay: 'どう書くべきですか。',
    rememberForProject: 'このプロジェクトに記憶して、次の会議で正しく書けるようにする',
    areAnyNames:
      'このなかに名前はありますか。1つ直すと、この文字起こしが修正され、記憶もされます。',
    nothingToCheck: '確認するものはありません',
    correctSpelling: '表記を修正',
    checkWording: '言い回しを確認',
    checkWords: (words: string) => `${words} を確認`,
    textAt: (time: string) => `${time} の文字起こしテキスト`,
    jumpTo: (time: string) => `${time} へ移動`,
    removeLineAt: (time: string) => `${time} の行を削除`,
    renameSpeaker: (speaker: string) => `${speaker} の名前を変更`,
    nameHeardAs: (heard: string) => `${heard} と聞こえた名前`,
    protocolStyle: '議事録スタイル',
    audioUnplayable: 'この会議の作業用音声を再生できませんでした。',
    speakersResolved:
      '話者の切り替わりはこの端末で判定されました。名前は暫定です。どなたか分かっている場合にのみ置き換えてください。',
    speakersFailed:
      'この処理では話者分離が使える結果を出せませんでした。文字起こしは無事で、一般的な名前が使われています。手作業で付け直して進められます。',
    speakersUnavailable:
      'この処理では話者分離を利用できませんでした。文字起こしは無事で、一般的な名前が1つ使われています。手作業で置き換えられます。',
    speakersUnknown:
      'この古い文字起こしには、話者分離が行われたかどうかの記録がありません。一般的な名前が付いていることは、話者が1人だった証拠にはなりません。',
  },

  library: {
    remove: '取り除く',
    edit: '編集',
    keep: '残す',
    notInUseSuffix: ' · 未使用',
    /** en.ts の注記を参照。名前が変更されていない間だけ使われます。 */
    shippedStyle: {
      'style-formal': {
        name: '正式な議事録',
        description: '議論・決定事項・対応事項を構造立てて残す形式です。',
      },
      'style-working-note': {
        name: '社内向けの作業メモ',
        description: '社内のプロジェクトチーム向けの、簡潔な作業記録です。',
      },
      'style-decision-log': {
        name: '技術的な決定の記録',
        description: '選択肢・制約・明示された決定を前面に出します。',
      },
    },
    copyOf: (name: string) => `${name}（コピー）`,
    enterATerm: '用語を入力してください。',
    reading: '読み込み中…',
    editTerm: '用語を編集',
    inUse: '使用中',
    notInUse: '未使用',
    instructionsGiven: 'これがモデルに与えられる指示で、与えられる順に並んでいます',
    asShipped: '（このスタイルが同梱されたときのままです）',
    invariantsNote:
      'これらはこのスタイルの一部ではなく、ここでは編集できません。そもそもスタイルとともに保存されていないからです。議事録を書くたびにすべての議事録へ加えられます。誰も下していない決定を記した文書は、別のスタイルの議事録ではなく、誤った議事録だからです。',
    whichTermsHelp:
      '人名、会社名、略語がもっとも効きます。一般的な専門用語は、挙げなくてもたいてい正しく書き起こされます。',
    termsLeadLong:
      'この仕事で使う人名、会社名、略語を加えると、正しく書き起こされます。実際の80分の会議では、プロジェクト名そのものが「一度も正しく書けない」から「常に正しい」に変わりました。',
    eyebrow: 'ライブラリ',
    protocolStyles: '議事録スタイル',
    namesAndTerms: '名前と用語',
    stylesLead:
      '議事録が何を、どの順で述べるか。組み方ではありません。組み方は体裁で、それを説明する文書の隣、エディタのなかにあります。',
    termsLead:
      '文字起こしが推測できない名前です。プロジェクト、会社、人。実際の会議で測ったところ、ここのどの設定よりも効果がありました。',
    addTerm: '用語を追加',
    saveTerm: '用語を保存',
    stylesUnreadable: 'ここではスタイルを読めません。',
    length: '長さ',
    name: '名前',
    description: '説明',
    whatItAsksFor: 'このスタイルが求めるもの',
    addInstruction: '指示を追加',
    removeInstruction: 'この指示を取り除く',
    checkedOnProtocol: '仕上がった議事録で確認されます',
    alwaysEveryStyle: '常に、すべてのスタイルで',
    saveStyle: 'スタイルを保存',
    cancel: 'キャンセル',
    delete: '削除',
    editThisStyle: 'このスタイルを編集',
    duplicate: '複製',
    duplicateToEdit: '複製して編集',
    shippedStyleNote:
      '同梱のスタイルはそのまま残ります。去年書いた議事録を、今日も同じように書けるようにするためです。ご自分のものにするには複製してください。',
    ownershipAutomatic: '割り当ては自動です。',
    termsScopeNote: 'プロジェクトの名前と用語は、その会議に毎回選ばなくても適用されます。',
    term: '用語',
    spellingAsShown: '表示されるべき表記',
    category: '分類',
    appliesTo: '適用範囲',
    everyProject: 'すべてのプロジェクト',
    unknownProject: '不明なプロジェクト',
    noTerms: 'まだ名前も用語もありません',
    deleteThisTerm: 'この用語を削除しますか。',
    densityFull: '文章で詳しく',
    densityPlain: '簡潔な記述',
    densityLine: '1項目1行',
    densityFullMeaning: '文章で詳しく。出席していない人でも議論を追えます。',
    densityPlainMeaning: '簡潔な記述。語られたことだけを、語り直さずに。',
    densityLineMeaning: '1項目1行。記録だけで、周辺はありません。',
    categoryPerson: '人',
    categoryOrganisation: '会社',
    categoryProject: 'プロジェクト',
    categoryAbbreviation: '略語',
    categoryTechnicalTerm: '専門用語',
    categoryOther: 'その他',
  },

  furniture: {
    header: 'ヘッダー',
    footer: 'フッター',
    left: '左',
    centre: '中央',
    insertInto: (where: string) => `${where} に値を差し込む`,
    right: '右',
    insert: '挿入…',
    lineHint:
      '読ませたいとおりに行を書き、必要なところに値を差し込んでください。「ページ 」、番号、「 / 12」のように。値は1つのまとまりで、選択も削除も丸ごと行われます。',
    appliesTo: (project: string) =>
      `${project}のすべての議事録に適用されます。印刷されたページで繰り返され、いま編集している文書の一部ではありません。`,
  },

  shell: {
    breadcrumbMeeting: '会議',
    breadcrumbRecording: '録音',
    breadcrumbReview: '確認',
    skipToWorkspace: '作業領域へ移動',
    workspace: '作業領域',
    workspaceFailed: '作業領域を開けませんでした',
    workspaceFailedDetail: '既存のファイルは変更されていません。',
    tryAgain: 'もう一度試す',
    preparingWorkspace: 'ローカルの作業領域を準備しています…',
    openNavigation: 'ナビゲーションを開く',

    notSelected: '未選択',

    jobNeedsDecision: '判断が必要です',
    jobReadyToContinue: '続ける準備ができました',
    jobCancelling: '安全に中止しています',

    formatWordDocument: 'Word文書',
    formatPlainText: 'テキスト',
    exportSaved: (format: string) => `${format}への書き出しを保存しました`,
    exportFailed: (format: string, why: string) => `${format}への書き出しに失敗しました：${why}`,
    exportPrepared: (format: string) => `${format}への書き出しを準備しました`,
    exportNeedsDesktop: (format: string) =>
      `${format}への書き出しにはデスクトップアプリケーションが必要です。`,

    meetingArchived: '会議をアーカイブしました。設定の「保存先」にあります。',
    projectArchived: 'プロジェクトをアーカイブしました。設定の「保存先」にあります。',
    transcriptExported: '文字起こしを書き出しました',
  },

  protocol: {
    undo: '元に戻す',
    redo: 'やり直す',
    next: '次へ',
    blockParagraph: '本文',
    blockHeading1: '見出し1',
    blockHeading2: '見出し2',
    blockHeading3: '見出し3',
    figuresMissingFromRewrite: (count: number) =>
      `元の箇所にあった数値${count}件が、この書き直しには入っていません`,
    markdownView: 'Markdown表示',
    documentView: '文書表示',
    looking: '探しています…',
    replaceAll: 'すべて置換',
    rewrite: '書き直す',
    rewriting: '書き直し中',
    figureMissingFromRewrite: '元の箇所にあった数値が、この書き直しには入っていません',
    reviewedRevisionPreserved:
      '確認済みの版は残っています。ここでの作業中の編集は確認されていません。',
    thisRevisionReviewed: 'この変更できない版そのものが、確認済みとされています。',
    generatedStaysEditable: '生成された内容は、引き続き確認も編集もできます。',
    notFound: '見つかりません',
    matchCount: (count: number) => `${count}件`,
    replacedCount: (count: number) => ` · ${count}件を置換`,
    changesNotYetMade: (count: number) => `${count}件の変更（未適用）`,
    compoundNote:
      '大文字で始まる名前は複合語の内部でも探します。単純な置換が見落とすのはそこです。目を通してから、採るか採らないかを決めてください。',
    andMore: (count: number) => `ほかに${count}件、いずれも同じ2つの形です。`,
    passageGoesAlone:
      'この箇所だけがお使いのローカルモデルに渡ります。数値、名前、日付はそのまま返るはずです。確認し、そうでなければ元に戻してください。',
    nothingChangedYet:
      'まだ何も変えていません。読んでから、採るか採らないかを決めてください。ローカルモデルの書き直しはよくできていますが、鵜呑みにするものではありません。',
    secondPassNote:
      'お使いのモデル自身に尋ねた結果で、両方向に誤ります。変更を見落としもすれば、問題のない言い回しを咎めもします。参考であって、判定ではありません。',
    pageEdgesNote:
      'ページが切れる位置を、印刷用のスタイルシートが組むとおりに測ったものです。見出しや表は分割されず丸ごと下がり、本文は分かれます。最後の1〜2行はプリンタが決めるため、1行程度の幅を見込んでご覧ください。',
    transcriptSourceNote:
      'この会議の確認済みの文字起こしから書かれています。どの箇所がどの文になったかは記録されていないため、以下は分かっているふりをせず語を探します。言い換えられていれば何も見つかりませんが、それが正直な答えです。',
    noWordsTogether:
      'これらの語は文字起こしのなかで一緒には現れません。たいていは下書きが自分の言葉で述べたということで、それは許されています。確かめる場所は録音です。',
    revisionNote:
      '入力した内容は作業中の編集として保存され、版にはなりません。版が作られるのは、下書きが生成されたとき、あなたが求めたとき、議事録を確認済みとしたとき、そして古い版を復元したときです。この一覧が読める長さに収まるようにするためです。',
    nothingRewrites:
      'ここにはあなたの文章を勝手に書き換えるものはありません。下書きはあなたのもので、すべての版が残ります。',
    figuresKept: (kept: number, stated: number) => `数値 ${stated}件のうち${kept}件を保持`,
    figuresNote: (stated: number, kept: number) =>
      `会議では${stated}件の数値が述べられ、この下書きはそのうち${kept}件を繰り返しています。どれだけ入るべきかは選んだスタイル次第ですので、これは点数ではなく目を通すためのものです。`,
    figuresInvented: (count: number) => `会議で述べられていない数値が${count}件あります`,
    confirmAgainstRecording: '。録音と照らして確かめる価値があります。',
    tasksUnowned: (count: number) => `担当者のない作業が${count}件あります`,
    unownedNote:
      '。下書きは担当者を推測せず空けておくため、会議で決まったとおりかもしれません。名前を入れるなら、次の会議より今のほうがはるかに安上がりです。',
    editor: '議事録エディタ',
    markdownBacked: 'Markdownに基づく',
    noteMissingTableHeading: '次の手順の表がありません',
    noteMissingTableBody:
      'この議事録は3回書かれましたが、いずれも合意された作業と担当者の表で終わりませんでした。会議で合意された行動は上のセクションに記されていますが、ここにはまとめられていません。',
    noteGapsHeading: 'この議事録に含まれていない部分',
    noteOneGap:
      '録音の一部を読み取れず、上のどこにも記されていません。録音そのものは完全で、聴き直すことができます。',
    noteSeveralGaps:
      '録音のいくつかの部分を読み取れず、上のどこにも記されていません。録音そのものは完全で、それらの部分も聴き直すことができます。',
    documentType: '議事録',
    statusDraft: '下書き',
    statusReviewed: '確認済み',
    statusChanged: '確認後に変更あり',
    fieldProjectName: 'プロジェクト名',
    fieldMeetingTitle: '会議の題名',
    fieldMeetingDate: '会議の日付',
    fieldDocumentType: '文書の種類',
    fieldProtocolStatus: '状態',
    fieldPageNumber: 'ページ番号',
    fieldPageOfCount: 'n / m ページ',
    fieldText: '自由入力',
    showPageBreaks: '改ページを表示',
    hidePageBreaks: '改ページを隠す',
    saving: '保存中…',
    autosaveFailed: '自動保存に失敗しました',
    workingEditsSaved: '作業中の編集を保存しました',
    revisionSaved: '版を保存しました',
    editorTools: 'ツール',
    find: '検索',
    findInProtocol: '議事録内を検索',
    replaceWith: '置換後',
    makeChanges: 'この変更を適用',
    leaveIt: 'そのままにする',
    zoomOut: '縮小',
    zoomIn: '拡大',
    insertTable: '表を挿入',
    insertDivider: '区切り線を挿入',
    documentMenu: '文書メニュー',
    clearFormatting: '書式を消す',
    table: '表',
    blockType: 'ブロックの種類',
    addColumnLeft: '左に列を追加',
    addColumnRight: '右に列を追加',
    deleteColumn: 'この列を削除',
    addRowAbove: '上に行を追加',
    addRowBelow: '下に行を追加',
    deleteRow: 'この行を削除',
    formatting: '書式',
    bold: '太字',
    italic: '斜体',
    bulletedList: '箇条書き',
    numberedList: '番号付きリスト',
    quotation: '引用',
    askModel: 'モデルに別の言い方を頼む',
    customInstruction: '自由な指示…',
    whatShouldChange: '何を変えますか。',
    proposedChange: '提案された変更',
    proposedReplacement: '提案された置換',
    proposedRewrite: '提案された書き直し',
    unchanged: 'モデルは箇所をそのまま返しました。',
    factsMoved: '2回目の確認では、これらの事実が動いたと見ています',
    noFactMoved: '2回目の確認では、動いた事実は見つかりませんでした。見落としもあります。',
    useThis: 'これを使う',
    improveClarity: '分かりやすくする',
    improveClarityInstruction: 'この箇所をもっと読みやすくしてください。',
    makeFormal: 'より硬い表現に',
    makeFormalInstruction: '専門的な議事録に書かれるような、より硬い文体にしてください。',
    makePlainer: 'より平易に',
    makePlainerInstruction: '正確さを保ったまま、言い回しをより平易で直接的にしてください。',
    shorten: '短くする',
    shortenInstruction: 'これをより少ない言葉で述べてください。',
    rewriteUnavailable: 'ここでは書き直しを利用できません。',
    replaceUnavailable: 'ここでは名前の置換を利用できません。',
    nameNotFound: 'その名前はこの議事録にありません。',
    protocolMarkdown: '議事録のMarkdown',
    protocolLabel: '議事録',
    protocolDetails: '議事録の詳細',
    documentDetails: '文書の詳細',
    closeInspector: 'パネルを閉じる',
    tabDocument: '文書',
    tabTranscript: '文字起こし',
    tabHistory: '履歴',
    status: '状態',
    createRevision: '版を作る',
    lineNumber: (line: number) => `${line} 行目`,
    pageNumber: (page: number) => `${page} ページ`,
    revisionNumber: (ordinal: number) => `第 ${ordinal} 版`,
    markReviewed: '確認済みにする',
    style: 'スタイル',
    sections: 'セクション',
    newSection: '新しいセクション',
    appearance: '体裁',
    editAppearance: '体裁を編集',
    headerFooter: 'ヘッダーとフッター',
    editHeaderFooter: 'ヘッダーとフッターを編集',
    nothingRepeated: 'ページに繰り返されるものはありません',
    presets: 'プリセット',
    useOrSavePreset: 'プリセットを使う・保存する',
    noneSaved: 'まだ保存されていません',
    savedCount: (count: number) => `${count}件を保存済み`,
    use: '使う',
    remove: '取り除く',
    nameThisPreset: 'このプリセットに名前を付ける',
    nameForPreset: 'このプリセットの名前',
    save: '保存',
    cancel: 'キャンセル',
    saveAsPreset: 'この体裁とヘッダーをプリセットとして保存',
    export: '書き出し',
    exportPdf: 'PDFで書き出す',
    exportWord: 'Wordで書き出す',
    exportMarkdown: 'Markdownで書き出す',
    exportPlainText: 'テキストで書き出す',
    exportNote:
      'PDFは、いま読んでいる文書を、このプロジェクトが議事録を組むとおりに印刷したものです。印刷ダイアログで「PDFとして保存」を選んでください。',
    source: '元ファイル',
    findSelectedPassage: '選択した箇所を探す',
    lookingFor: '探しているもの：',
    openReviewedTranscript: '確認済みの文字起こしを開く',
    whatToCheck: '確認したいこと',
    revisions: '版',
    current: '現在',
    restore: '復元',
  },

  sidebar: {
    projects: 'プロジェクト',
    newProject: '新しいプロジェクト',
    createProject: 'プロジェクトを作成',
    library: 'ライブラリ',
    protocolStyles: '議事録スタイル',
    namesAndTerms: '名前と用語',
    settings: '設定',
    recording: '録音',
    primaryNavigation: '主なナビゲーション',
    closeNavigation: 'ナビゲーションを閉じる',
    openNavigation: 'ナビゲーションを開く',
    themeFollowingSystem: 'システムのテーマに従っています。常にライトへ切り替えます。',
    themeAlwaysLight: '常にライトです。常にダークへ切り替えます。',
    themeAlwaysDark: '常にダークです。システムに従うよう戻します。',
    themeFollowingShort: 'システムに従う',
    sidebarWidth: (width: number) => `${width} ピクセル`,
    resizeSidebar: 'サイドバーの幅を変えます。矢印キーで調整、Enterで元に戻します。',
    themeAlwaysLightShort: '常にライト',
    themeAlwaysDarkShort: '常にダーク',

    importNeedsDecision: '取り込みに判断が必要です',
    needsAttention: '確認が必要です',
    importingRecording: '録音を取り込んでいます',
    transcribing: '文字起こし中',
    writingProtocol: '議事録を書いています',
    working: '処理中',
    workingEllipsis: '処理中…',
    separatingSpeakers: '話者を分離中',
    openMeetingNeedingAttention: '確認が必要な会議を開く',
    openThisMeeting: 'この会議を開く',
  },

  start: {
    eyebrow: '機密の議事録のための、ローカルで動くAI',
    title: '会議を始める',
    lead: '音声または動画のファイルを取り込みます。議事録になる前に、一段階ずつ確認できます。',
    importTitle: '録音を取り込む',
    importDetail: 'プロジェクトを選び、すべてを文脈のなかに保ちます',
    recordTitle: '会議を録音する',
    recordDetail: '室内と通話を、この端末で別々のトラックに収めます',
    promiseTitle: '会議に関する作業は、この端末から出ません。',
    promiseDetail: 'LocaLogのアカウントも、クラウドも、テレメトリもありません。',

    setupProviderTitle: '最初の議事録をつくる前に、もうひとつ',
    setupProviderBody:
      '文字起こしはもう使えます。議事録を書くには、この端末に言語モデルも必要で、設定から用意します。その前に録音の読み込みと文字起こしはできます。',
    setupProviderAction: '設定で用意する',
    setupTitle: '最初の文字起こしの前に、1回だけダウンロードします',
    setupBody: (quality: string, size: string) =>
      `LocaLogはこの端末で文字起こしを行うため、モデルがこの端末にある必要があります。${quality}の品質は${size}で、一度だけダウンロードします。先に録音を取り込むこともできます。モデルが必要になるのは文字起こしを始めるときで、それより前ではありません。`,
    setupDownload: (size: string) => `今すぐダウンロード（${size}）`,
    setupCancel: 'キャンセル',
    setupAside: 'ほかの品質と話者分離は、設定にあります。',
  },
};
