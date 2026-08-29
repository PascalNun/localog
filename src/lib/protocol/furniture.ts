/**
 * The header and footer, turned from fields into words.
 *
 * Fields rather than typed text, because "Page 3 of 6" has to be counted and a date
 * has to come from the meeting rather than from whoever last edited the setting.
 * What is typed is one kind of field among several, which is how anything this list
 * does not cover still gets in.
 *
 * Page numbers are the one thing that cannot be resolved here: only the thing doing
 * the paginating knows them. Word is told to count them itself; the print path
 * cannot, and says so rather than printing "Page 1 of 1" on every sheet.
 */

import type { FurnitureField, FurnitureRow, PageFurniture } from '../workflow/types';
import type { Strings } from '../i18n';

export interface DocumentFacts {
  projectName: string;
  meetingTitle: string;
  meetingDate: string;
  documentType: string;
  protocolStatus: string;
}

/**
 * Every kind of value a header or footer line can hold.
 *
 * The identifiers, and only the identifiers. These are stored in a project's
 * furniture and read back from an edited line, so they are the schema and never
 * translate — a header somebody builds in German has to still resolve in English.
 * The words for them live in the dictionary and are looked up separately.
 */
const FIELD_KINDS: FurnitureField['kind'][] = [
  'projectName',
  'meetingTitle',
  'meetingDate',
  'documentType',
  'protocolStatus',
  'pageNumber',
  'pageOfCount',
  'text',
];

function labelFor(kind: FurnitureField['kind'], words: Strings): string {
  const f = words.protocol;
  switch (kind) {
    case 'projectName':
      return f.fieldProjectName;
    case 'meetingTitle':
      return f.fieldMeetingTitle;
    case 'meetingDate':
      return f.fieldMeetingDate;
    case 'documentType':
      return f.fieldDocumentType;
    case 'protocolStatus':
      return f.fieldProtocolStatus;
    case 'pageNumber':
      return f.fieldPageNumber;
    case 'pageOfCount':
      return f.fieldPageOfCount;
    case 'text':
      return f.fieldText;
  }
}

/**
 * What each field is called, where somebody is choosing between them.
 *
 * A function of the dictionary rather than a constant, because a constant reads the
 * words once when the module loads and goes on saying them after the language
 * changes. That is the same fault the lifecycle labels and the band names had.
 */
export function furnitureFields(words: Strings): { kind: FurnitureField['kind']; label: string }[] {
  return FIELD_KINDS.map((kind) => ({ kind, label: labelFor(kind, words) }));
}

export function fieldLabel(field: FurnitureField, words: Strings): string {
  if (field.kind === 'text') return field.value || words.protocol.fieldText;
  return labelFor(field.kind, words);
}

/**
 * One slot's worth of text.
 *
 * `pageMarker` stands in for whatever the target uses to count pages — a field code
 * in Word, nothing at all where pages cannot be counted.
 */
export function resolveRow(
  fields: FurnitureField[],
  facts: DocumentFacts,
  pageMarker: { number: string; ofCount: string } | null,
): string {
  const parts = fields.map((field) => resolveField(field, facts, pageMarker));

  // A slot that mentions something this output cannot answer is left out whole.
  //
  // Dropping only the unanswerable part and keeping its neighbours is what
  // produced the footer somebody would most naturally build: the word "Seite"
  // beside a page number printed "Seite · 3" in Word and, on every page of the
  // PDF, the bare word "Seite". Better to say nothing there than a fragment.
  if (parts.some((part) => part === null)) return '';

  // Joined by nothing, so that a slot is a line somebody writes rather than a
  // list of atoms with a separator imposed between them. The spacing belongs to
  // whoever typed it: "Seite " and " von " are theirs.
  return parts.join('');
}

/**
 * What a field says here, or `null` where this output cannot answer it at all.
 *
 * Empty and unanswerable are different. A meeting with no date resolves to an
 * empty string and its neighbours stand; a page number in a PDF the browser
 * cannot paginate resolves to nothing at all, and the line that mentions it
 * cannot be printed honestly.
 */
function resolveField(
  field: FurnitureField,
  facts: DocumentFacts,
  pageMarker: { number: string; ofCount: string } | null,
): string | null {
  switch (field.kind) {
    case 'projectName':
      return facts.projectName;
    case 'meetingTitle':
      return facts.meetingTitle;
    case 'meetingDate':
      return facts.meetingDate;
    case 'documentType':
      return facts.documentType;
    case 'protocolStatus':
      return facts.protocolStatus;
    case 'pageNumber':
      return pageMarker?.number ?? null;
    case 'pageOfCount':
      return pageMarker?.ofCount ?? null;
    case 'text':
      // Not trimmed: the spaces around a word are how it sits against the value
      // beside it, and taking them away is what made a slot a list rather than a
      // line.
      return field.value;
  }
}

export function rowIsEmpty(row: FurnitureRow): boolean {
  return row.left.length === 0 && row.centre.length === 0 && row.right.length === 0;
}

export function furnitureIsEmpty(furniture: PageFurniture): boolean {
  return rowIsEmpty(furniture.header) && rowIsEmpty(furniture.footer);
}

/**
 * A slot as editable HTML: the text somebody typed, with each value a single
 * object sitting in the middle of it.
 *
 * The value carries `contenteditable="false"` so that it behaves the way the same
 * thing does in Word — selected and deleted whole, never half-edited into a word
 * nobody meant. What it reads as is its label, not a syntax: nobody should have to
 * learn that `{{seite}}` means anything.
 */
export function lineHtml(fields: FurnitureField[], words: Strings): string {
  return fields
    .map((field) =>
      field.kind === 'text'
        ? escapeForLine(field.value)
        : `<span class="furniture-value" contenteditable="false" data-kind="${field.kind}">` +
          `${escapeForLine(fieldLabel(field, words))}</span>`,
    )
    .join('');
}

/** One piece of an edited line: either a value, or the characters around it. */
export interface LinePart {
  /** The kind of value, where this part is one. */
  kind?: string;
  /** The characters, where it is not. */
  text?: string;
}

/**
 * The line read back, after somebody has typed in it.
 *
 * Takes parts rather than nodes so it can be tested without a browser, the way
 * html.ts takes `DomLike`. Adjacent runs of text are joined, because a browser
 * will happily split one into several as the caret moves through it, and an empty
 * run is dropped — but a run of spaces is not, since the spaces around a value are
 * how it sits against the words beside it.
 */
export function fieldsFromLine(parts: LinePart[]): FurnitureField[] {
  const fields: FurnitureField[] = [];
  for (const part of parts) {
    if (part.kind) {
      if (KNOWN_KINDS.has(part.kind)) fields.push({ kind: part.kind } as FurnitureField);
      continue;
    }
    const text = part.text ?? '';
    if (text === '') continue;
    const last = fields[fields.length - 1];
    if (last && last.kind === 'text') last.value += text;
    else fields.push({ kind: 'text', value: text });
  }
  return fields;
}

const KNOWN_KINDS: Set<string> = new Set<string>(FIELD_KINDS);

/** Only the three characters that would otherwise end the run of text. */
function escapeForLine(text: string): string {
  return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}
