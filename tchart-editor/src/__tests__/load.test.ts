import { describe, it, expect, vi, afterEach } from "vitest";

vi.mock("tchart-web", () => import("../__mocks__/wasm"));

import { loadFile } from "../load";

const PNG_SIGNATURE = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];

function makeMockPng(): Uint8Array<ArrayBuffer> {
  return new Uint8Array(PNG_SIGNATURE);
}

function makeMockPngWithMockSource(source: string): Uint8Array<ArrayBuffer> {
  const enc = new TextEncoder();
  const tag = enc.encode("MOCK_TCHART_SOURCE:");
  const payload = enc.encode(source);
  const out = new Uint8Array(PNG_SIGNATURE.length + tag.length + payload.length + 1);
  out.set(PNG_SIGNATURE, 0);
  out.set(tag, PNG_SIGNATURE.length);
  out.set(payload, PNG_SIGNATURE.length + tag.length);
  out[PNG_SIGNATURE.length + tag.length + payload.length] = 0x00;
  return out;
}

function makeFile(content: BlobPart, name: string, type: string): File {
  return new File([content], name, { type });
}

describe("loadFile", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("routes SVG content to the SVG extractor", async () => {
    const svg = "<svg><tchart:source>Clock _~_~</tchart:source></svg>";
    const file = makeFile(svg, "chart.svg", "image/svg+xml");
    expect(await loadFile(file)).toBe("Clock _~_~");
  });

  it("returns null when the SVG has no <tchart:source>", async () => {
    const file = makeFile("<svg></svg>", "x.svg", "image/svg+xml");
    expect(await loadFile(file)).toBeNull();
  });

  it("routes PNG content to the PNG extractor", async () => {
    const png = makeMockPngWithMockSource("@title 日本語\nclk _~");
    const file = makeFile(png, "chart.png", "image/png");
    expect(await loadFile(file)).toBe("@title 日本語\nclk _~");
  });

  it("returns null when the PNG has no tchart-source iTXt chunk", async () => {
    const png = makeMockPng();
    const file = makeFile(png, "chart.png", "image/png");
    expect(await loadFile(file)).toBeNull();
  });

  it("returns null for a non-PNG, non-SVG file (no <tchart:source>)", async () => {
    const file = makeFile(new Uint8Array([1, 2, 3]), "junk.bin", "application/octet-stream");
    expect(await loadFile(file)).toBeNull();
  });

  it("detects PNG by content signature even when the filename says .svg", async () => {
    const png = makeMockPngWithMockSource("Clock _~_~");
    const file = makeFile(png, "renamed.svg", "image/svg+xml");
    expect(await loadFile(file)).toBe("Clock _~_~");
  });
});
