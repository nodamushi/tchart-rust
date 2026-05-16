import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";

vi.mock("tchart-web", () => import("../__mocks__/wasm"));

import { convertToWaveJson, formatStatus, handleWaveDromClick } from "../wavedrom";

describe("WaveDrom export", () => {
  let status: HTMLElement;

  beforeEach(() => {
    document.body.innerHTML = `<div id="status" class="status"></div>`;
    status = document.getElementById("status") as HTMLElement;
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("convertToWaveJson returns json and warnings", () => {
    const result = convertToWaveJson("Clock _~_~");
    expect(result.json).toContain("signal");
    expect(Array.isArray(result.warnings)).toBe(true);
  });

  it("convertToWaveJson throws on invalid input", () => {
    expect(() => convertToWaveJson("INVALID")).toThrow();
  });

  it("formatStatus collapses warnings appropriately", () => {
    expect(formatStatus([])).toBe("WaveJSON exported.");
    expect(formatStatus(["one"])).toContain("one");
    expect(formatStatus(["one", "two", "three"])).toContain("3 warnings");
  });

  it("handleWaveDromClick triggers a download for valid input", () => {
    const clickSpy = vi.spyOn(HTMLAnchorElement.prototype, "click");
    const result = handleWaveDromClick("Clock _~_~", status);

    expect(result).not.toBeNull();
    expect(clickSpy).toHaveBeenCalled();
    expect(status.textContent).toContain("WaveJSON exported");
    expect(status.classList.contains("error")).toBe(false);
  });

  it("handleWaveDromClick reports parse errors via status", () => {
    const result = handleWaveDromClick("INVALID", status);

    expect(result).toBeNull();
    expect(status.textContent).toContain("WaveDrom export failed");
    expect(status.classList.contains("error")).toBe(true);
  });

  it("handleWaveDromClick surfaces warnings to status", () => {
    const result = handleWaveDromClick("WARN clk _~", status);

    expect(result).not.toBeNull();
    expect(status.textContent).toContain("mock warning");
  });
});
