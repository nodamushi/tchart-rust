import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";

vi.mock("tchart-web", () => import("../__mocks__/wasm"));

import { downloadSvg, downloadPng } from "../export";

describe("SVG download", () => {
  let clickedDownload: string | null;
  let revokedUrl: string | null;

  beforeEach(() => {
    clickedDownload = null;
    revokedUrl = null;

    vi.spyOn(URL, "createObjectURL").mockReturnValue("blob:mock-url");
    vi.spyOn(URL, "revokeObjectURL").mockImplementation((url: string) => {
      revokedUrl = url;
    });

    // Intercept anchor click
    vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(
      function (this: HTMLAnchorElement) {
        clickedDownload = this.download;
      },
    );
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("should create a Blob with image/svg+xml type", () => {
    const svgContent = "<svg><text>test</text></svg>";
    downloadSvg(svgContent);

    expect(URL.createObjectURL).toHaveBeenCalledTimes(1);
    const blob = (URL.createObjectURL as ReturnType<typeof vi.fn>).mock.calls[0][0] as Blob;
    expect(blob).toBeInstanceOf(Blob);
    expect(blob.type).toBe("image/svg+xml");
  });

  it("should download with filename tchart.svg", () => {
    downloadSvg("<svg></svg>");
    expect(clickedDownload).toBe("tchart.svg");
  });

  it("should revoke the object URL after download", () => {
    downloadSvg("<svg></svg>");
    expect(revokedUrl).toBe("blob:mock-url");
  });
});

describe("PNG download", () => {
  let clickedDownload: string | null;

  beforeEach(() => {
    clickedDownload = null;

    vi.spyOn(URL, "createObjectURL").mockReturnValue("blob:mock-png-url");
    vi.spyOn(URL, "revokeObjectURL").mockImplementation(() => {});

    vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(
      function (this: HTMLAnchorElement) {
        clickedDownload = this.download;
      },
    );
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("should create canvas with 4x resolution and download as tchart.png", async () => {
    const svgContent = '<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50"></svg>';

    // Mock Image
    const mockImg = {
      naturalWidth: 100,
      naturalHeight: 50,
      onload: null as (() => void) | null,
      set src(_: string) {
        // Trigger onload asynchronously
        setTimeout(() => this.onload?.(), 0);
      },
    };

    vi.spyOn(globalThis, "Image").mockImplementation(() => mockImg as unknown as HTMLImageElement);

    // Mock canvas
    const mockCtx = {
      drawImage: vi.fn(),
    };

    // Minimal valid PNG signature so the wasm mock recognises it as PNG.
    const minimalPng = new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
    const mockToBlob = vi.fn((callback: (blob: Blob | null) => void, type: string) => {
      expect(type).toBe("image/png");
      callback(new Blob([minimalPng], { type: "image/png" }));
    });

    vi.spyOn(document, "createElement").mockImplementation(((tagName: string) => {
      if (tagName === "canvas") {
        return {
          width: 0,
          height: 0,
          getContext: () => mockCtx,
          toBlob: mockToBlob,
        } as unknown as HTMLCanvasElement;
      }
      if (tagName === "a") {
        const a = document.createElementNS(
          "http://www.w3.org/1999/xhtml",
          "a",
        ) as HTMLAnchorElement;
        return a;
      }
      return document.createElementNS("http://www.w3.org/1999/xhtml", tagName);
    }) as typeof document.createElement);

    downloadPng(svgContent, "Clock _~_~");

    // Wait for Image onload + async toBlob handler to fire
    await new Promise((resolve) => setTimeout(resolve, 30));

    // Canvas should be 4x the SVG dimensions
    expect(mockCtx.drawImage).toHaveBeenCalled();
    expect(mockToBlob).toHaveBeenCalled();
    expect(clickedDownload).toBe("tchart.png");
  });
});
