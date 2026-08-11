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

export type CommonMeetingLanguage = (typeof COMMON_MEETING_LANGUAGES)[number];
