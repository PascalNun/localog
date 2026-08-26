/**
 * A protocol as a Word document.
 *
 * Built from the same blocks and runs the screen and the PDF are built from, so
 * the three cannot drift: what changes the document changes all of them. A `.docx`
 * is a ZIP holding four small XML parts, which is why this needs no library —
 * writing the parts is more honest than configuring something that writes them.
 *
 * The measurements below are the format's own units. A twip is a twentieth of a
 * point, so A4 is 11906 × 16838 twips; half-points are what run sizes are given
 * in, so 21 is a 10.5pt body.
 */

import { wordFontName, wordSizes } from './appearance';
import { readBlocks, readInline, type Block, type Run } from './markdown';
import { resolveRow, rowIsEmpty } from './furniture';
import type { DocumentAppearance, FurnitureRow, PageFurniture } from '../workflow/types';
import type { DocumentFacts } from './furniture';
import type { ProtocolDocument } from './document';
import { writeZip, type ZipEntry } from './zip';

const NAMESPACE =
  'xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" ' +
  'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"';

/** A4 with the margins a printed protocol uses, matching the PDF. */
function page(furniture: PageFurniture | undefined): string {
  const references = [
    furniture && !rowIsEmpty(furniture.header)
      ? '<w:headerReference w:type="default" r:id="rIdHeader"/>'
      : '',
    furniture && !rowIsEmpty(furniture.footer)
      ? '<w:footerReference w:type="default" r:id="rIdFooter"/>'
      : '',
  ].join('');
  return (
    `<w:sectPr>${references}<w:pgSz w:w="11906" w:h="16838"/>` +
    '<w:pgMar w:top="1418" w:right="1134" w:bottom="1247" w:left="1134" ' +
    'w:header="709" w:footer="709" w:gutter="0"/></w:sectPr>'
  );
}

/**
 * A header or footer part.
 *
 * Three slots in one paragraph, held apart by tab stops rather than by a table:
 * a centre tab at the middle of the text column and a right tab at its end, which
 * is how Word's own headers are built.
 */
function furniturePart(row: FurnitureRow, facts: DocumentFacts, kind: 'hdr' | 'ftr'): string {
  // Word counts its own pages, so the marker is a field code rather than a number.
  const marker = {
    number: '\u0001PAGE\u0001',
    ofCount: '\u0001PAGE\u0001 / \u0001NUMPAGES\u0001',
  };
  const slots = [
    resolveRow(row.left, facts, marker),
    resolveRow(row.centre, facts, marker),
    resolveRow(row.right, facts, marker),
  ];
  const tag = kind === 'hdr' ? 'w:hdr' : 'w:ftr';
  const runs = [
    slotRuns(slots[0] ?? ''),
    '<w:r><w:tab/></w:r>',
    slotRuns(slots[1] ?? ''),
    '<w:r><w:tab/></w:r>',
    slotRuns(slots[2] ?? ''),
  ].join('');
  return (
    `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>` +
    `<${tag} ${NAMESPACE}><w:p><w:pPr><w:tabs>` +
    '<w:tab w:val="center" w:pos="4819"/><w:tab w:val="right" w:pos="9638"/>' +
    `</w:tabs><w:rPr><w:sz w:val="17"/><w:color w:val="595959"/></w:rPr></w:pPr>${runs}</w:p></${tag}>`
  );
}

/**
 * One slot's text, with page fields turned into the codes Word evaluates.
 *
 * A field is three runs — begin, the instruction, end — which is why the text
 * arrives here with the instruction names marked out rather than already written.
 */
function slotRuns(text: string): string {
  if (text === '') return '';
  return text
    .split('\u0001')
    .map((piece, index) => {
      const small = '<w:rPr><w:sz w:val="17"/><w:color w:val="595959"/></w:rPr>';
      if (index % 2 === 0) {
        return piece === ''
          ? ''
          : `<w:r>${small}<w:t xml:space="preserve">${escapeXml(piece)}</w:t></w:r>`;
      }
      return (
        `<w:r>${small}<w:fldChar w:fldCharType="begin"/></w:r>` +
        `<w:r>${small}<w:instrText xml:space="preserve"> ${piece} </w:instrText></w:r>` +
        `<w:r>${small}<w:fldChar w:fldCharType="end"/></w:r>`
      );
    })
    .join('');
}

export function buildDocx(protocol: ProtocolDocument): Uint8Array {
  const blocks = readBlocks(protocol.markdown);
  const hasHeader =
    !!protocol.furniture && !!protocol.facts && !rowIsEmpty(protocol.furniture.header);
  const hasFooter =
    !!protocol.furniture && !!protocol.facts && !rowIsEmpty(protocol.furniture.footer);
  const body = [
    paragraph(runs(protocol.title), 'ProtocolTitle'),
    protocol.subtitle ? paragraph(runs(protocol.subtitle), 'ProtocolSubtitle') : '',
    ...blocks.map(blockToXml),
    page(protocol.furniture),
  ].join('');

  const document =
    `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>` +
    `<w:document ${NAMESPACE}><w:body>${body}</w:body></w:document>`;

  const entries: ZipEntry[] = [
    { path: '[Content_Types].xml', bytes: bytes(contentTypes(hasHeader, hasFooter)) },
    { path: '_rels/.rels', bytes: bytes(ROOT_RELATIONSHIPS) },
    {
      path: 'word/_rels/document.xml.rels',
      bytes: bytes(documentRelationships(hasHeader, hasFooter)),
    },
    { path: 'word/document.xml', bytes: bytes(document) },
    { path: 'word/styles.xml', bytes: bytes(styles(protocol.appearance)) },
    { path: 'word/numbering.xml', bytes: bytes(NUMBERING) },
  ];
  const facts = protocol.facts;
  if (protocol.furniture && facts) {
    if (hasHeader) {
      entries.push({
        path: 'word/header1.xml',
        bytes: bytes(furniturePart(protocol.furniture.header, facts, 'hdr')),
      });
    }
    if (hasFooter) {
      entries.push({
        path: 'word/footer1.xml',
        bytes: bytes(furniturePart(protocol.furniture.footer, facts, 'ftr')),
      });
    }
  }
  return writeZip(entries);
}

function bytes(text: string): Uint8Array {
  return new TextEncoder().encode(text);
}

function blockToXml(block: Block): string {
  switch (block.kind) {
    case 'rule':
      // Word has no horizontal rule element; an empty paragraph with a bottom
      // border is what a rule is in this format.
      return '<w:p><w:pPr><w:pBdr><w:bottom w:val="single" w:sz="6" w:space="1" w:color="BBBBBB"/></w:pBdr></w:pPr></w:p>';
    case 'heading':
      return paragraph(runs(block.text), `Heading${Math.min(block.level, 4)}`);
    case 'paragraph':
      return paragraph(runs(block.text));
    case 'quote':
      return paragraph(runs(block.lines.join(' ')), 'ProtocolQuote');
    case 'list':
      return block.items
        .map((item) => paragraph(runs(item), 'ListParagraph', block.ordered ? 2 : 1))
        .join('');
    case 'table':
      return table(block.head, block.rows);
  }
}

function runs(text: string): Run[] {
  return readInline(text);
}

function paragraph(content: Run[], style?: string, numbering?: number): string {
  const properties = [
    style ? `<w:pStyle w:val="${style}"/>` : '',
    numbering === undefined
      ? ''
      : `<w:numPr><w:ilvl w:val="0"/><w:numId w:val="${numbering}"/></w:numPr>`,
  ].join('');
  const withProperties = properties ? `<w:pPr>${properties}</w:pPr>` : '';
  return `<w:p>${withProperties}${content.map(runToXml).join('')}</w:p>`;
}

function runToXml(run: Run): string {
  const marks = [
    run.bold ? '<w:b/>' : '',
    run.italic ? '<w:i/>' : '',
    run.code ? '<w:rFonts w:ascii="Courier New" w:hAnsi="Courier New"/>' : '',
  ].join('');
  const properties = marks ? `<w:rPr>${marks}</w:rPr>` : '';
  // A link keeps its destination in the text, because carrying it as a real
  // hyperlink needs a relationship per link and a protocol's links are rare.
  const text = run.link ? `${run.text} (${run.link})` : run.text;
  return `<w:r>${properties}<w:t xml:space="preserve">${escapeXml(text)}</w:t></w:r>`;
}

function table(head: string[], rows: string[][]): string {
  const columns = Math.max(head.length, ...rows.map((row) => row.length), 1);
  // The usable width inside the margins, shared evenly. Word needs a grid even
  // when the widths are equal.
  const width = Math.floor(9638 / columns);
  const grid = Array.from({ length: columns }, () => `<w:gridCol w:w="${width}"/>`).join('');

  const cell = (text: string, header: boolean) =>
    `<w:tc><w:tcPr><w:tcW w:w="${width}" w:type="dxa"/></w:tcPr>` +
    paragraph(
      header ? runs(text).map((run) => ({ ...run, bold: true })) : runs(text),
      'TableText',
    ) +
    '</w:tc>';

  const pad = (row: string[]) => Array.from({ length: columns }, (_, index) => row[index] ?? '');

  const headRow =
    '<w:tr><w:trPr><w:tblHeader/></w:trPr>' +
    pad(head)
      .map((text) => cell(text, true))
      .join('') +
    '</w:tr>';
  const bodyRows = rows
    .map(
      (row) =>
        `<w:tr>${pad(row)
          .map((text) => cell(text, false))
          .join('')}</w:tr>`,
    )
    .join('');

  return (
    '<w:tbl><w:tblPr><w:tblStyle w:val="ProtocolTable"/>' +
    '<w:tblW w:w="0" w:type="auto"/>' +
    '<w:tblBorders>' +
    '<w:top w:val="single" w:sz="4" w:space="0" w:color="BBBBBB"/>' +
    '<w:bottom w:val="single" w:sz="4" w:space="0" w:color="BBBBBB"/>' +
    '<w:insideH w:val="single" w:sz="4" w:space="0" w:color="BBBBBB"/>' +
    '</w:tblBorders></w:tblPr>' +
    `<w:tblGrid>${grid}</w:tblGrid>${headRow}${bodyRows}</w:tbl>`
  );
}

const XML_ESCAPES: Record<string, string> = {
  '&': '&amp;',
  '<': '&lt;',
  '>': '&gt;',
  '"': '&quot;',
  "'": '&apos;',
};

export function escapeXml(text: string): string {
  return (
    text
      .replace(/[&<>"']/g, (character) => XML_ESCAPES[character] ?? character)
      // Control characters are not representable in XML 1.0 and Word refuses a
      // document containing them outright rather than skipping them.
      // eslint-disable-next-line no-control-regex -- matching control characters is the job.
      .replace(/[\u0000-\u0008\u000B\u000C\u000E-\u001F]/g, '')
  );
}

function contentTypes(hasHeader: boolean, hasFooter: boolean): string {
  return (
    `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>` +
    `<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">` +
    `<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>` +
    `<Default Extension="xml" ContentType="application/xml"/>` +
    `<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>` +
    `<Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>` +
    `<Override PartName="/word/numbering.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/>` +
    (hasHeader
      ? `<Override PartName="/word/header1.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml"/>`
      : '') +
    (hasFooter
      ? `<Override PartName="/word/footer1.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml"/>`
      : '') +
    `</Types>`
  );
}

const ROOT_RELATIONSHIPS =
  `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>` +
  `<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">` +
  `<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>` +
  `</Relationships>`;

function documentRelationships(hasHeader: boolean, hasFooter: boolean): string {
  const base = 'http://schemas.openxmlformats.org/officeDocument/2006/relationships';
  return (
    `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>` +
    `<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">` +
    `<Relationship Id="rId1" Type="${base}/styles" Target="styles.xml"/>` +
    `<Relationship Id="rId2" Type="${base}/numbering" Target="numbering.xml"/>` +
    (hasHeader ? `<Relationship Id="rIdHeader" Type="${base}/header" Target="header1.xml"/>` : '') +
    (hasFooter ? `<Relationship Id="rIdFooter" Type="${base}/footer" Target="footer1.xml"/>` : '') +
    `</Relationships>`
  );
}

/** Named styles, so that a person receiving this can restyle it in one place. */
/** Named styles, so that a person receiving this can restyle it in one place. */
function styles(appearance: DocumentAppearance): string {
  const size = wordSizes(appearance);
  const font = wordFontName(appearance.font);
  return (
    `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>` +
    `<w:styles ${NAMESPACE}>` +
    `<w:docDefaults><w:rPrDefault><w:rPr>` +
    `<w:rFonts w:ascii="${font}" w:hAnsi="${font}"/><w:sz w:val="${size.body}"/></w:rPr></w:rPrDefault>` +
    `<w:pPrDefault><w:pPr><w:spacing w:after="120" w:line="${size.line}" w:lineRule="auto"/></w:pPr></w:pPrDefault>` +
    `</w:docDefaults>` +
    style('Normal', 'Normal', '', true) +
    style(
      'ProtocolTitle',
      'Protocol title',
      '<w:spacing w:after="60"/>',
      false,
      `<w:b/><w:sz w:val="${Math.round(size.heading1 * 1.2)}"/>`,
    ) +
    style(
      'ProtocolSubtitle',
      'Protocol subtitle',
      '<w:spacing w:after="360"/>',
      false,
      `<w:color w:val="595959"/><w:sz w:val="${Math.round(size.body * 0.9)}"/>`,
    ) +
    style(
      'Heading1',
      'heading 1',
      '<w:keepNext/><w:spacing w:before="360" w:after="120"/>',
      false,
      `<w:b/><w:sz w:val="${size.heading1}"/>`,
    ) +
    style(
      'Heading2',
      'heading 2',
      '<w:keepNext/><w:spacing w:before="280" w:after="100"/>',
      false,
      `<w:b/><w:sz w:val="${size.heading2}"/>`,
    ) +
    style(
      'Heading3',
      'heading 3',
      '<w:keepNext/><w:spacing w:before="240" w:after="80"/>',
      false,
      `<w:b/><w:sz w:val="${size.heading3}"/>`,
    ) +
    style(
      'Heading4',
      'heading 4',
      '<w:keepNext/><w:spacing w:before="200" w:after="80"/>',
      false,
      `<w:b/><w:sz w:val="${size.heading4}"/>`,
    ) +
    style('ListParagraph', 'List Paragraph', '<w:spacing w:after="60"/><w:ind w:left="567"/>') +
    style(
      'ProtocolQuote',
      'Quote',
      '<w:ind w:left="454"/>',
      false,
      '<w:i/><w:color w:val="595959"/>',
    ) +
    style(
      'TableText',
      'Table text',
      '<w:spacing w:before="60" w:after="60"/>',
      false,
      `<w:sz w:val="${Math.round(size.body * 0.9)}"/>`,
    ) +
    `</w:styles>`
  );
}

function style(
  id: string,
  name: string,
  paragraphProperties = '',
  isDefault = false,
  runProperties = '',
): string {
  return (
    `<w:style w:type="paragraph"${isDefault ? ' w:default="1"' : ''} w:styleId="${id}">` +
    `<w:name w:val="${name}"/>` +
    (id === 'Normal' ? '' : '<w:basedOn w:val="Normal"/>') +
    (paragraphProperties ? `<w:pPr>${paragraphProperties}</w:pPr>` : '') +
    (runProperties ? `<w:rPr>${runProperties}</w:rPr>` : '') +
    `</w:style>`
  );
}

/** One bulleted list and one numbered list, which is every list a protocol has. */
const NUMBERING =
  `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>` +
  `<w:numbering ${NAMESPACE}>` +
  `<w:abstractNum w:abstractNumId="1"><w:multiLevelType w:val="singleLevel"/><w:lvl w:ilvl="0">` +
  `<w:start w:val="1"/><w:numFmt w:val="bullet"/><w:lvlText w:val="•"/>` +
  `<w:lvlJc w:val="left"/><w:pPr><w:ind w:left="567" w:hanging="283"/></w:pPr>` +
  // Word draws the bullet in the glyph's own font; without this it can arrive as
  // a missing character rather than a dot.
  `<w:rPr><w:rFonts w:ascii="Symbol" w:hAnsi="Symbol" w:hint="default"/></w:rPr>` +
  `</w:lvl></w:abstractNum>` +
  `<w:abstractNum w:abstractNumId="2"><w:multiLevelType w:val="singleLevel"/><w:lvl w:ilvl="0">` +
  `<w:start w:val="1"/><w:numFmt w:val="decimal"/><w:lvlText w:val="%1."/>` +
  `<w:lvlJc w:val="left"/><w:pPr><w:ind w:left="567" w:hanging="283"/></w:pPr>` +
  `</w:lvl></w:abstractNum>` +
  `<w:num w:numId="1"><w:abstractNumId w:val="1"/></w:num>` +
  `<w:num w:numId="2"><w:abstractNumId w:val="2"/></w:num>` +
  `</w:numbering>`;
