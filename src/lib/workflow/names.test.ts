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
    expect(parseNames('Halde, Fachplanung; Prüfstelle\nVermessung')).toEqual([
      'Halde',
      'Fachplanung',
      'Prüfstelle',
      'Vermessung',
    ]);
  });

  /**
   * The one that would quietly ruin this. A person is "Halde" and a client is
   * "Klinker-Nord"; splitting on spaces would put half-names into the initial
   * prompt, which biases the transcriber towards a fragment.
   */
  it('never splits a name on its spaces', () => {
    expect(parseNames('Halde, Halle 4, Klinker-Nord')).toEqual([
      'Halde',
      'Halle 4',
      'Klinker-Nord',
    ]);
  });

  it('survives the punctuation people actually leave behind', () => {
    expect(parseNames('  HOAI ,, Tragwerk ,  ')).toEqual(['HOAI', 'Tragwerk']);
    expect(parseNames('')).toEqual([]);
    expect(parseNames('   \n  ')).toEqual([]);
  });

  it('keeps a repeat once, spelled the way it was typed first', () => {
    // Not a mistake to report: somebody who writes a name twice meant it once.
    expect(parseNames('HOAI, hoai, Hoai')).toEqual(['HOAI']);
  });
});

describe('the names from the whole form', () => {
  it('takes each name’s category from the field it was typed into', () => {
    expect(
      namesFromFields(fields({ Person: 'Halde', Organisation: 'HOAI', Project: 'Halle 4' })),
    ).toEqual([
      { term: 'Halde', category: 'Person' },
      { term: 'HOAI', category: 'Organisation' },
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
        Organisation: 'HOAI',
        Person: 'Halde',
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
    const all = namesFromFields(fields({ Person: 'Prüfstelle', Organisation: 'prüfstelle' }));
    expect(all).toEqual([{ term: 'Prüfstelle', category: 'Person' }]);
  });

  it('returns nothing for a form nobody filled in', () => {
    expect(namesFromFields(fields())).toEqual([]);
  });
});
