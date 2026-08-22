import { describe, expect, it } from 'vitest';
import { resolveRow } from './furniture';
import type { FurnitureField } from '../workflow/types';

const facts = {
  projectName: 'Neubau Halle 4',
  meetingTitle: 'Jour fixe',
  meetingDate: '29.07.2026',
  documentType: 'Protokoll',
  protocolStatus: 'Entwurf',
};

const pages = { number: '3', ofCount: '3 von 12' };
const text = (value: string): FurnitureField => ({ kind: 'text', value });

describe('a slot is a line somebody writes', () => {
  it('puts a value in the middle of a sentence', () => {
    expect(resolveRow([text('Projekt: '), { kind: 'projectName' }], facts, null)).toBe(
      'Projekt: Neubau Halle 4',
    );
  });

  it('keeps the spacing the person typed', () => {
    const line: FurnitureField[] = [text('Seite '), { kind: 'pageNumber' }, text(' von 12')];
    expect(resolveRow(line, facts, pages)).toBe('Seite 3 von 12');
  });

  it('imposes no separator between two values', () => {
    const line: FurnitureField[] = [{ kind: 'documentType' }, text(' · '), { kind: 'meetingDate' }];
    expect(resolveRow(line, facts, null)).toBe('Protokoll · 29.07.2026');
  });
});

describe('what an output cannot answer', () => {
  it('leaves out the whole line rather than a fragment of it', () => {
    // The natural footer, in a PDF the browser cannot paginate. It used to print
    // the bare word "Seite" on every page.
    const line: FurnitureField[] = [text('Seite '), { kind: 'pageNumber' }];
    expect(resolveRow(line, facts, null)).toBe('');
  });

  it('prints that same line where the pages can be counted', () => {
    const line: FurnitureField[] = [text('Seite '), { kind: 'pageNumber' }];
    expect(resolveRow(line, facts, pages)).toBe('Seite 3');
  });

  it('keeps a line whose value is merely empty', () => {
    // A meeting with no date is not the same as an output that cannot count pages.
    const line: FurnitureField[] = [text('Datum: '), { kind: 'meetingDate' }];
    expect(resolveRow(line, { ...facts, meetingDate: '' }, null)).toBe('Datum: ');
  });
});
