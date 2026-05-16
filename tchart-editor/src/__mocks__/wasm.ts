import { vi } from "vitest";

export const init = vi.fn(() => Promise.resolve());

// why: mirrors the real wasm API after the RenderResult migration. Successful
// renders return `{ svg }`; parse failures return `{ error }`. Mock parse
// errors fabricate a plausible location so editor-side underline logic can be
// exercised. Tests that need a specific error shape can override per-call
// via `.mockReturnValueOnce(...)`.
export const renderTcml = vi.fn((input: string) => {
  if (!input || input.trim() === "" || input.includes("INVALID")) {
    return {
      error: {
        line: 1,
        column: 1,
        length: input.length === 0 ? 0 : Math.min(input.length, 7),
        message: "Parse error: invalid TCML",
      },
    };
  }
  return {
    svg: '<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50"><text>mock</text></svg>',
  };
});

export const toWaveJson = vi.fn((input: string) => {
  if (!input || input.trim() === "" || input.includes("INVALID")) {
    throw new Error("Parse error: invalid TCML");
  }
  const warnings = input.includes("WARN") ? ["mock warning"] : [];
  return {
    json: '{"signal":[{"name":"clk","wave":"10"}]}',
    warnings,
  };
});

export const extractTcmlSource = vi.fn((svg: string) => {
  const open = svg.indexOf("<tchart:source>");
  const close = svg.indexOf("</tchart:source>");
  if (open < 0 || close < 0) return undefined;
  return svg.slice(open + "<tchart:source>".length, close);
});

// why: PNG magic number — the fixed 8-byte signature defined by RFC 2083.
const PNG_SIGNATURE = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
const PNG_MOCK_SOURCE_TAG = new TextEncoder().encode("MOCK_TCHART_SOURCE:");
// why: mirror the production module (`load.ts`) and instantiate
// `TextDecoder` once at module load instead of allocating one per call.
const UTF8_DECODER = new TextDecoder("utf-8");

function startsWithTagAt(bytes: Uint8Array, offset: number): boolean {
  if (offset + PNG_MOCK_SOURCE_TAG.length > bytes.length) return false;
  return PNG_MOCK_SOURCE_TAG.every((byte, index) => {
    // why: under `noUncheckedIndexedAccess`, `bytes[offset + index]` is
    // `number | undefined`. The bounds check at the top guarantees it is in
    // range, but rejecting `undefined` explicitly makes the intent local to
    // the read site without forcing the reader to chase the caller.
    const actual = bytes[offset + index];
    if (actual === undefined) return false;
    return actual === byte;
  });
}

function findMockSource(bytes: Uint8Array): string | undefined {
  for (let i = 0; i + PNG_MOCK_SOURCE_TAG.length <= bytes.length; i++) {
    if (!startsWithTagAt(bytes, i)) continue;
    const start = i + PNG_MOCK_SOURCE_TAG.length;
    // why: the payload is 0x00-terminated; locate the terminator directly
    // via `indexOf`.
    const nullIndex = bytes.indexOf(0x00, start);
    const end = nullIndex < 0 ? bytes.length : nullIndex;
    return UTF8_DECODER.decode(bytes.subarray(start, end));
  }
  return undefined;
}

function isPng(bytes: Uint8Array): boolean {
  if (bytes.length < PNG_SIGNATURE.length) return false;
  return PNG_SIGNATURE.every((byte, index) => bytes[index] === byte);
}

export const extractTcmlSourceFromPng = vi.fn((bytes: Uint8Array) => {
  if (!isPng(bytes)) return undefined;
  return findMockSource(bytes);
});

export const embedTcmlSourceInPng = vi.fn((bytes: Uint8Array, source: string) => {
  if (!isPng(bytes)) {
    throw new Error("not a PNG buffer");
  }
  const encoder = new TextEncoder();
  const tag = PNG_MOCK_SOURCE_TAG;
  const payload = encoder.encode(source);
  const out = new Uint8Array(bytes.length + tag.length + payload.length + 1);
  out.set(bytes, 0);
  out.set(tag, bytes.length);
  out.set(payload, bytes.length + tag.length);
  out[bytes.length + tag.length + payload.length] = 0x00;
  return out;
});

export default init;
