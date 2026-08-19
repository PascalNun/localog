import { describe, expect, it } from 'vitest';
import { buildDocx, escapeXml } from './docx';
import { crc32, writeZip } from './zip';

/** Read an archive back far enough to name its entries and their sizes. */
function readZipEntries(archive: Uint8Array): { path: string; size: number }[] {
  const view = new DataView(archive.buffer, archive.byteOffset, archive.byteLength);
  // The end-of-central-directory record is the last 22 bytes when no comment.
  const end = archive.length - 22;
  expect(view.getUint32(end, true)).toBe(0x06054b50);
  const count = view.getUint16(end + 10, true);
  let at = view.getUint32(end + 16, true);

  const entries: { path: string; size: number }[] = [];
  for (let index = 0; index < count; index += 1) {
    expect(view.getUint32(at, true)).toBe(0x02014b50);
    const size = view.getUint32(at + 24, true);
    const nameLength = view.getUint16(at + 28, true);
    const path = new TextDecoder().decode(archive.slice(at + 46, at + 46 + nameLength));
    entries.push({ path, size });
    at += 46 + nameLength + view.getUint16(at + 30, true) + view.getUint16(at + 32, true);
  }
  return entries;
}

/** Pull one entry's bytes back out through its local header. */
function readEntry(archive: Uint8Array, path: string): string {
  const view = new DataView(archive.buffer, archive.byteOffset, archive.byteLength);
  const end = archive.length - 22;
  const count = view.getUint16(end + 10, true);
  let at = view.getUint32(end + 16, true);
  for (let index = 0; index < count; index += 1) {
    const nameLength = view.getUint16(at + 28, true);
    const found = new TextDecoder().decode(archive.slice(at + 46, at + 46 + nameLength));
    const offset = view.getUint32(at + 42, true);
    if (found === path) {
      const localNameLength = view.getUint16(offset + 26, true);
      const extraLength = view.getUint16(offset + 28, true);
      const size = view.getUint32(offset + 18, true);
      const start = offset + 30 + localNameLength + extraLength;
      return new TextDecoder().decode(archive.slice(start, start + size));
    }
    at += 46 + nameLength + view.getUint16(at + 30, true) + view.getUint16(at + 32, true);
  }
  throw new Error(`no entry ${path}`);
}

describe('the zip writer', () => {
  it('computes the CRC the format specifies', () => {
    // The published check value for "123456789".
    expect(crc32(new TextEncoder().encode('123456789'))).toBe(0xcbf43926);
    expect(crc32(new Uint8Array())).toBe(0);
  });

  it('writes entries that can be found and read again', () => {
    const archive = writeZip([
      { path: 'a.txt', bytes: new TextEncoder().encode('first') },
      { path: 'nested/b.txt', bytes: new TextEncoder().encode('second') },
    ]);
    expect(readZipEntries(archive)).toEqual([
      { path: 'a.txt', size: 5 },
      { path: 'nested/b.txt', size: 6 },
    ]);
    expect(readEntry(archive, 'nested/b.txt')).toBe('second');
  });

  /// Exporting the same protocol twice should produce the same file, or a
  /// document that did not change looks like one that did.
  it('is the same bytes for the same input', () => {
    const once = writeZip([{ path: 'a.txt', bytes: new TextEncoder().encode('x') }]);
    const twice = writeZip([{ path: 'a.txt', bytes: new TextEncoder().encode('x') }]);
    expect(Array.from(once)).toEqual(Array.from(twice));
  });
});

describe('building a Word document', () => {
  const protocol = {
    title: 'Protokoll der Sitzung',
    subtitle: 'Beispielquartier · 2026-08-06',
    markdown: [
      '# Teilnehmende',
      '',
      'Frau **Bauleitung** und Herr *Planung*.',
      '',
      '## Optionen',
      '',
      '- Die leichtere Ausführung',
      '- Die schwerere Ausführung',
      '',
      '1. Erste Frage',
      '2. Zweite Frage',
      '',
      '| Aufgabe | Verantwortlich |',
      '| --- | --- |',
      '| Angebot einholen | Frau Bauleitung |',
    ].join('\n'),
  };

  it('is an archive holding the parts a docx must have', () => {
    const paths = readZipEntries(buildDocx(protocol)).map((entry) => entry.path);
    expect(paths).toEqual([
      '[Content_Types].xml',
      '_rels/.rels',
      'word/_rels/document.xml.rels',
      'word/document.xml',
      'word/styles.xml',
      'word/numbering.xml',
    ]);
  });

  it('carries the title, the headings and the text', () => {
    const document = readEntry(buildDocx(protocol), 'word/document.xml');
    expect(document).toContain('Protokoll der Sitzung');
    expect(document).toContain('<w:pStyle w:val="ProtocolTitle"/>');
    expect(document).toContain('<w:pStyle w:val="Heading1"/>');
    expect(document).toContain('<w:pStyle w:val="Heading2"/>');
    expect(document).toContain('Frau ');
  });

  it('keeps bold and italic as marks rather than as asterisks', () => {
    const document = readEntry(buildDocx(protocol), 'word/document.xml');
    expect(document).toContain('<w:rPr><w:b/></w:rPr><w:t xml:space="preserve">Bauleitung</w:t>');
    expect(document).toContain('<w:rPr><w:i/></w:rPr><w:t xml:space="preserve">Planung</w:t>');
    expect(document).not.toContain('**');
  });

  it('makes real lists, bulleted and numbered', () => {
    const document = readEntry(buildDocx(protocol), 'word/document.xml');
    expect(document).toContain('<w:numId w:val="1"/>');
    expect(document).toContain('<w:numId w:val="2"/>');
    const numbering = readEntry(buildDocx(protocol), 'word/numbering.xml');
    expect(numbering).toContain('<w:numFmt w:val="bullet"/>');
    expect(numbering).toContain('<w:numFmt w:val="decimal"/>');
  });

  it('makes the actions table a table, with a repeating header row', () => {
    const document = readEntry(buildDocx(protocol), 'word/document.xml');
    expect(document).toContain('<w:tbl>');
    expect(document).toContain('<w:tblHeader/>');
    expect(document).toContain('Angebot einholen');
    expect(document).not.toContain('| Aufgabe');
  });

  it('sets A4', () => {
    const document = readEntry(buildDocx(protocol), 'word/document.xml');
    expect(document).toContain('<w:pgSz w:w="11906" w:h="16838"/>');
  });

  it('escapes what XML cannot carry, and drops what it cannot represent', () => {
    expect(escapeXml('Fassade & <Dach>')).toBe('Fassade &amp; &lt;Dach&gt;');
    expect(escapeXml('badchar')).toBe('badchar');
    const document = readEntry(
      buildDocx({ title: 'A & B', subtitle: '', markdown: '<script>' }),
      'word/document.xml',
    );
    expect(document).toContain('A &amp; B');
    expect(document).toContain('&lt;script&gt;');
  });

  it('builds an empty protocol without failing', () => {
    expect(() => buildDocx({ title: 'Leer', subtitle: '', markdown: '' })).not.toThrow();
  });
});
