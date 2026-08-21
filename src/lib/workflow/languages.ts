/**
 * Common meeting-language choices shown in the calm, normal workflow.
 *
 * The list is a convenience, not a restriction: people can still type a
 * language that is not listed here. Meeting language is deliberately kept
 * separate from the application's interface language.
 */
export const COMMON_MEETING_LANGUAGES = [
  'English',
  'German',
  'French',
  'Spanish',
  'Italian',
  'Dutch',
  'Portuguese',
  'Polish',
  'Danish',
  'Swedish',
  'Norwegian',
  'Finnish',
  'Czech',
  'Turkish',
  'Japanese',
  'Korean',
  'Chinese',
  'Arabic',
  'Ukrainian',
] as const;

/**
 * What an unset meeting language means, and how to say it.
 *
 * The default used to be "English", which is a guess made from nothing: a German
 * recording was transcribed in English because a field nobody had looked at
 * asserted a language the application had no evidence for. Leaving it unset is
 * the honest state, and the transcription runtime already reads an empty
 * language as "work it out from the audio".
 *
 * Detection fills a gap; it never overrides a choice. Where somebody has named a
 * language, that is what runs.
 */
export const DETECT_LANGUAGE_LABEL = 'Detect from the recording';

export function meetingLanguageLabel(language: string | null | undefined): string {
  const named = (language ?? '').trim();
  return named === '' ? DETECT_LANGUAGE_LABEL : named;
}
