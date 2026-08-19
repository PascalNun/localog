/**
 * Just enough ZIP to write a `.docx`.
 *
 * A Word document is a ZIP holding four small XML files, and the entries may be
 * stored rather than compressed — the whole protocol is a few tens of kilobytes,
 * so the compression would save nothing worth a dependency. This writes the three
 * structures the format needs and nothing else: no reading, no directories, no
 * Zip64, no encryption.
 */

/** One file inside the archive. */
export interface ZipEntry {
  path: string;
  bytes: Uint8Array;
}

/**
 * The DOS timestamp every entry carries.
 *
 * Fixed rather than taken from the clock, so that exporting the same protocol
 * twice produces the same bytes. A document that differs only in its timestamp
 * looks like a document that changed.
 */
const STAMPED_TIME = 0;
const STAMPED_DATE = 0x21; // 1 January 1980, the earliest the format can express.

export function writeZip(entries: ZipEntry[]): Uint8Array {
  const locals: Uint8Array[] = [];
  const centrals: Uint8Array[] = [];
  let offset = 0;

  for (const entry of entries) {
    const name = new TextEncoder().encode(entry.path);
    const crc = crc32(entry.bytes);
    const size = entry.bytes.length;

    const local = new Uint8Array(30 + name.length);
    const localView = new DataView(local.buffer);
    localView.setUint32(0, 0x04034b50, true);
    localView.setUint16(4, 20, true); // version needed
    localView.setUint16(6, 0, true); // flags
    localView.setUint16(8, 0, true); // stored
    localView.setUint16(10, STAMPED_TIME, true);
    localView.setUint16(12, STAMPED_DATE, true);
    localView.setUint32(14, crc, true);
    localView.setUint32(18, size, true);
    localView.setUint32(22, size, true);
    localView.setUint16(26, name.length, true);
    localView.setUint16(28, 0, true);
    local.set(name, 30);
    locals.push(local, entry.bytes);

    const central = new Uint8Array(46 + name.length);
    const centralView = new DataView(central.buffer);
    centralView.setUint32(0, 0x02014b50, true);
    centralView.setUint16(4, 20, true); // version made by
    centralView.setUint16(6, 20, true); // version needed
    centralView.setUint16(8, 0, true);
    centralView.setUint16(10, 0, true);
    centralView.setUint16(12, STAMPED_TIME, true);
    centralView.setUint16(14, STAMPED_DATE, true);
    centralView.setUint32(16, crc, true);
    centralView.setUint32(20, size, true);
    centralView.setUint32(24, size, true);
    centralView.setUint16(28, name.length, true);
    centralView.setUint16(30, 0, true); // extra
    centralView.setUint16(32, 0, true); // comment
    centralView.setUint16(34, 0, true); // disk
    centralView.setUint16(36, 0, true); // internal attributes
    centralView.setUint32(38, 0, true); // external attributes
    centralView.setUint32(42, offset, true);
    central.set(name, 46);
    centrals.push(central);

    offset += local.length + size;
  }

  const directorySize = centrals.reduce((total, part) => total + part.length, 0);
  const end = new Uint8Array(22);
  const endView = new DataView(end.buffer);
  endView.setUint32(0, 0x06054b50, true);
  endView.setUint16(4, 0, true);
  endView.setUint16(6, 0, true);
  endView.setUint16(8, entries.length, true);
  endView.setUint16(10, entries.length, true);
  endView.setUint32(12, directorySize, true);
  endView.setUint32(16, offset, true);
  endView.setUint16(20, 0, true);

  return concat([...locals, ...centrals, end]);
}

function concat(parts: Uint8Array[]): Uint8Array {
  const total = parts.reduce((sum, part) => sum + part.length, 0);
  const out = new Uint8Array(total);
  let at = 0;
  for (const part of parts) {
    out.set(part, at);
    at += part.length;
  }
  return out;
}

/** The table is built once; the polynomial is the standard reversed 0xEDB88320. */
let table: Uint32Array | null = null;

function crcTable(): Uint32Array {
  if (table) return table;
  const built = new Uint32Array(256);
  for (let index = 0; index < 256; index += 1) {
    let value = index;
    for (let bit = 0; bit < 8; bit += 1) {
      value = value & 1 ? 0xedb88320 ^ (value >>> 1) : value >>> 1;
    }
    built[index] = value >>> 0;
  }
  table = built;
  return built;
}

export function crc32(bytes: Uint8Array): number {
  const lookup = crcTable();
  let crc = 0xffffffff;
  for (const byte of bytes) {
    crc = (lookup[(crc ^ byte) & 0xff] ?? 0) ^ (crc >>> 8);
  }
  return (crc ^ 0xffffffff) >>> 0;
}
