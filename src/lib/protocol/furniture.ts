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

export interface DocumentFacts {
  projectName: string;
  meetingTitle: string;
  meetingDate: string;
  documentType: string;
  protocolStatus: string;
}

/** What a field is called where somebody is choosing between them. */
export const FURNITURE_FIELDS: { kind: FurnitureField['kind']; label: string }[] = [
  { kind: 'projectName', label: 'Project name' },
  { kind: 'meetingTitle', label: 'Meeting title' },
  { kind: 'meetingDate', label: 'Meeting date' },
  { kind: 'documentType', label: 'Document type' },
  { kind: 'protocolStatus', label: 'Status' },
  { kind: 'pageNumber', label: 'Page number' },
  { kind: 'pageOfCount', label: 'Page n of m' },
  { kind: 'text', label: 'Custom text' },
];

export function fieldLabel(field: FurnitureField): string {
  if (field.kind === 'text') return field.value || 'Custom text';
  return FURNITURE_FIELDS.find((known) => known.kind === field.kind)?.label ?? field.kind;
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

/** Whether anything in here needs a page count, which not every target can supply. */
export function needsPageNumbers(furniture: PageFurniture): boolean {
  const counted = (row: FurnitureRow) =>
    [...row.left, ...row.centre, ...row.right].some(
      (field) => field.kind === 'pageNumber' || field.kind === 'pageOfCount',
    );
  return counted(furniture.header) || counted(furniture.footer);
}
