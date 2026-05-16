import { describe, it, expect, beforeEach, afterEach, vi, type MockInstance } from "vitest";

import { triggerDownload } from "../lib/download";

describe("triggerDownload", () => {
  let clickedDownload: string | null;
  let clickedHref: string | null;
  let revokedUrl: string | null;
  let createObjectUrlSpy: MockInstance<typeof URL.createObjectURL>;

  beforeEach(() => {
    clickedDownload = null;
    clickedHref = null;
    revokedUrl = null;

    createObjectUrlSpy = vi.spyOn(URL, "createObjectURL").mockReturnValue("blob:mock-url");
    vi.spyOn(URL, "revokeObjectURL").mockImplementation((url: string) => {
      revokedUrl = url;
    });

    vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(
      function (this: HTMLAnchorElement) {
        clickedDownload = this.download;
        clickedHref = this.href;
      },
    );
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("creates an object URL from the blob and clicks the anchor with the filename", () => {
    const blob = new Blob(["hello"], { type: "text/plain" });
    triggerDownload(blob, "hello.txt");

    expect(createObjectUrlSpy).toHaveBeenCalledTimes(1);
    const passedBlob = createObjectUrlSpy.mock.calls[0]?.[0];
    if (!(passedBlob instanceof Blob)) {
      throw new Error("expected createObjectURL to be called with a Blob");
    }
    expect(passedBlob).toBe(blob);
    expect(clickedDownload).toBe("hello.txt");
    expect(clickedHref).toContain("blob:mock-url");
  });

  it("revokes the object URL after the click", () => {
    triggerDownload(new Blob(["x"], { type: "text/plain" }), "x.txt");
    expect(revokedUrl).toBe("blob:mock-url");
  });
});
