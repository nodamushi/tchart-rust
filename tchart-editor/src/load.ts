import { extractTcmlSource, extractTcmlSourceFromPng } from "tchart-web";

// why: PNG magic number — the fixed 8-byte signature defined by RFC 2083.
const PNG_SIGNATURE = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];

// why: instantiate `TextDecoder` once at module load to avoid the per-call
// allocation cost.
const UTF8_DECODER = new TextDecoder("utf-8");

function isPng(bytes: Uint8Array): boolean {
  if (bytes.length < PNG_SIGNATURE.length) return false;
  return PNG_SIGNATURE.every((byte, index) => bytes[index] === byte);
}

/**
 * Read the given `File` and extract `<tchart:source>`: from the iTXt chunk
 * if it is a PNG, otherwise by decoding the bytes as UTF-8 text. Returns
 * `null` when no source can be recovered.
 */
export async function loadFile(file: File): Promise<string | null> {
  const bytes = new Uint8Array(await file.arrayBuffer());
  if (isPng(bytes)) {
    return extractTcmlSourceFromPng(bytes) ?? null;
  }
  const text = UTF8_DECODER.decode(bytes);
  return extractTcmlSource(text) ?? null;
}
