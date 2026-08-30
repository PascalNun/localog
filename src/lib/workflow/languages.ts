import { strings } from '../i18n';

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
export function detectLanguageLabel(): string {
  return strings().dialog.detectFromRecording;
}

/**
 * What to show for a meeting's language.
 *
 * The stored value is the identifier — `German`, which is what the transcription
 * runtime is handed — and this is the label for it. Anything not in the list is
 * shown as it was typed, because the field accepts a language nobody listed and
 * hiding what somebody wrote would be worse than showing it untranslated.
 */
export function meetingLanguageLabel(language: string | null | undefined): string {
  const named = (language ?? '').trim();
  if (named === '') return detectLanguageLabel();
  const names = strings().meetingLanguages as Record<string, string | undefined>;
  return names[named] ?? named;
}

/**
 * The label to put in an editable field, where empty must stay empty so that the
 * placeholder can say what an empty field means.
 */
export function meetingLanguageField(stored: string | null | undefined): string {
  const named = (stored ?? '').trim();
  return named === '' ? '' : meetingLanguageLabel(named);
}

/**
 * The identifier to store for what somebody chose or typed.
 *
 * The picker offers the language named in the reader's language and stores the
 * English identifier, because that value is handed to the transcription runtime
 * and written into the database — a French user choosing *Allemand* must not
 * produce a meeting whose language is the string `Allemand`.
 *
 * Case-insensitive, because the field is free text and somebody typing a language
 * rather than picking it should not have to match the capitalisation. Anything
 * that matches no label is stored as typed, which is what makes the list a
 * convenience rather than a restriction.
 */
export function meetingLanguageValue(typed: string): string {
  const said = typed.trim();
  if (said === '') return '';
  const names = strings().meetingLanguages as Record<string, string>;
  const found = Object.entries(names).find(
    ([, label]) => label.toLocaleLowerCase() === said.toLocaleLowerCase(),
  );
  return found ? found[0] : said;
}
