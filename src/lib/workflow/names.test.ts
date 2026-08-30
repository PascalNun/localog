import { describe, expect, it } from 'vitest';
import { namesFromFields, parseNames, type NameKind } from './names';

const fields = (over: Partial<Record<NameKind, string>> = {}): Record<NameKind, string> => ({
  Person: '',
  Organisation: '',
  Project: '',
  'Technical term': '',
  ...over,
});

describe('splitting the names somebody typed', () => {
  it('takes commas, semicolons and line breaks', () => {
    expect(parseNames('Waldt, Rovelli; Solvane\nNorrbo')).toEqual([
      'Waldt',
      'Rovelli',
      'Solvane',
      'Norrbo',
    ]);
  });

  /**
   * The one that would quietly ruin this. A person is "Anna Waldt" and a client is
   * "Falkenstein-Weide"; splitting on spaces would put half-names into the initial
   * prompt, which biases the transcriber towards a fragment.
   */
  it('never splits a name on its spaces', () => {
    expect(parseNames('Anna Waldt, Halle 4, Falkenstein-Weide')).toEqual([
      'Anna Waldt',
      'Halle 4',
      'Falkenstein-Weide',
    ]);
  });

  it('survives the punctuation people actually leave behind', () => {
    expect(parseNames('  AVENTOR ,, Tragwerk ,  ')).toEqual(['AVENTOR', 'Tragwerk']);
    expect(parseNames('')).toEqual([]);
    expect(parseNames('   \n  ')).toEqual([]);
  });

  it('keeps a repeat once, spelled the way it was typed first', () => {
    // Not a mistake to report: somebody who writes a name twice meant it once.
    expect(parseNames('AVENTOR, aventor, Aventor')).toEqual(['AVENTOR']);
  });
});

describe('the names from the whole form', () => {
  it('takes each name’s category from the field it was typed into', () => {
    expect(
      namesFromFields(fields({ Person: 'Waldt', Organisation: 'AVENTOR', Project: 'Halle 4' })),
    ).toEqual([
      { term: 'Waldt', category: 'Person' },
      { term: 'AVENTOR', category: 'Organisation' },
      { term: 'Halle 4', category: 'Project' },
    ]);
  });

  /**
   * The order is the order the fields are asked, and it is not arbitrary: it is the
   * order the storage layer trims in when the list outgrows the transcriber's short
   * prompt, so people and organisations survive and general terms go first.
   */
  it('asks for people first, because that is what survives a trim', () => {
    const all = namesFromFields(
      fields({
        'Technical term': 'Tragwerk',
        Project: 'Halle 4',
        Organisation: 'AVENTOR',
        Person: 'Waldt',
      }),
    );
    expect(all.map((name) => name.category)).toEqual([
      'Person',
      'Organisation',
      'Project',
      'Technical term',
    ]);
  });

  it('stores a name typed into two fields once, under the first of them', () => {
    // Storing it twice would spend part of the transcriber's short prompt saying
    // one thing twice.
    const all = namesFromFields(fields({ Person: 'Solvane', Organisation: 'solvane' }));
    expect(all).toEqual([{ term: 'Solvane', category: 'Person' }]);
  });

  it('returns nothing for a form nobody filled in', () => {
    expect(namesFromFields(fields())).toEqual([]);
  });
});
