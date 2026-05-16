import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";

vi.mock("tchart-web", () => import("../__mocks__/wasm"));

import { PreviewController, type OutputButtons } from "../preview";

interface TestHarness {
  readonly editor: HTMLTextAreaElement;
  readonly preview: HTMLElement;
  readonly saveSvgBtn: HTMLButtonElement;
  readonly savePngBtn: HTMLButtonElement;
  readonly wavedromBtn: HTMLButtonElement;
  readonly status: HTMLElement;
  readonly buttons: OutputButtons;
  readonly controller: PreviewController;
  readonly underlineHost: HTMLElement | null;
}

interface TestHarnessWithUnderline extends TestHarness {
  readonly underlineHost: HTMLElement;
}

interface BuildHarnessOptions {
  readonly withUnderline: boolean;
}

function buildHarness(options: BuildHarnessOptions): TestHarness {
  const underlineMarkup = options.withUnderline ? `<div id="underline-host"></div>` : "";
  document.body.innerHTML = `
    <textarea id="editor"></textarea>
    ${underlineMarkup}
    <div id="preview"></div>
    <button id="save-svg">Save SVG</button>
    <button id="save-png">Save PNG</button>
    <button id="save-wavedrom">WaveDrom</button>
    <div id="status"></div>
  `;
  const editor = document.getElementById("editor");
  const preview = document.getElementById("preview");
  const saveSvgBtn = document.getElementById("save-svg");
  const savePngBtn = document.getElementById("save-png");
  const wavedromBtn = document.getElementById("save-wavedrom");
  const status = document.getElementById("status");
  const underlineHost = options.withUnderline ? document.getElementById("underline-host") : null;
  if (
    !(editor instanceof HTMLTextAreaElement) ||
    !(preview instanceof HTMLElement) ||
    !(saveSvgBtn instanceof HTMLButtonElement) ||
    !(savePngBtn instanceof HTMLButtonElement) ||
    !(wavedromBtn instanceof HTMLButtonElement) ||
    !(status instanceof HTMLElement) ||
    (options.withUnderline && !(underlineHost instanceof HTMLElement))
  ) {
    throw new Error("test fixture DOM missing expected elements");
  }
  const buttons: OutputButtons = {
    saveSvg: saveSvgBtn,
    savePng: savePngBtn,
    wavedrom: wavedromBtn,
  };
  const controller =
    underlineHost === null
      ? new PreviewController({ editor, preview, buttons, status })
      : new PreviewController({ editor, preview, buttons, status, underlineHost });
  return {
    editor,
    preview,
    saveSvgBtn,
    savePngBtn,
    wavedromBtn,
    status,
    buttons,
    controller,
    underlineHost,
  };
}

function setupHarness(): TestHarness {
  return buildHarness({ withUnderline: false });
}

function setupHarnessWithUnderline(): TestHarnessWithUnderline {
  const harness = buildHarness({ withUnderline: true });
  if (harness.underlineHost === null) {
    throw new Error("buildHarness with underline returned null underlineHost");
  }
  return { ...harness, underlineHost: harness.underlineHost };
}

describe("PreviewController", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  describe("init", () => {
    it("seeds the textarea with non-empty initial content", async () => {
      const harness = setupHarness();
      await harness.controller.init();
      expect(harness.editor.value.length).toBeGreaterThan(0);
    });

    it("renders SVG preview on init", async () => {
      const harness = setupHarness();
      await harness.controller.init();
      expect(harness.preview.innerHTML).toContain("<svg");
    });

    it("exposes the rendered SVG via getCurrentSvg after init", async () => {
      const harness = setupHarness();
      await harness.controller.init();
      const svg = harness.controller.getCurrentSvg();
      expect(svg).not.toBeNull();
      expect(svg).toContain("<svg");
    });
  });

  describe("handleInput", () => {
    it("renders after 300ms debounce window", async () => {
      const harness = setupHarness();
      await harness.controller.init();
      harness.editor.value = "Clock _~_~";
      harness.controller.handleInput();

      vi.advanceTimersByTime(300);

      expect(harness.preview.innerHTML).toContain("<svg");
    });

    it("debounces multiple rapid inputs to a single render call", async () => {
      const harness = setupHarness();
      const { renderTcml } = await import("../__mocks__/wasm");
      await harness.controller.init();

      const renderMock = vi.mocked(renderTcml);
      renderMock.mockClear();

      for (let i = 0; i < 5; i++) {
        harness.editor.value = `Clock _~_~ ${i}`;
        harness.controller.handleInput();
        vi.advanceTimersByTime(100);
      }

      vi.advanceTimersByTime(300);

      expect(renderTcml).toHaveBeenCalledTimes(1);
    });
  });

  describe("error path", () => {
    it("shows an error message in the status row when render returns an error", async () => {
      const harness = setupHarness();
      await harness.controller.init();
      harness.editor.value = "INVALID";
      harness.controller.handleInput();
      vi.advanceTimersByTime(300);

      expect(harness.status.classList.contains("error")).toBe(true);
      expect(harness.status.textContent).toContain("Parse error");
    });

    it("keeps previous SVG markup in the preview on parse error", async () => {
      const harness = setupHarness();
      await harness.controller.init();
      const before = harness.preview.innerHTML;
      expect(before).toContain("<svg");

      harness.editor.value = "INVALID";
      harness.controller.handleInput();
      vi.advanceTimersByTime(300);

      expect(harness.preview.innerHTML).toBe(before);
    });

    it("retains the previous currentSvg on parse error", async () => {
      const harness = setupHarness();
      await harness.controller.init();
      const before = harness.controller.getCurrentSvg();
      expect(before).not.toBeNull();

      harness.editor.value = "INVALID";
      harness.controller.handleInput();
      vi.advanceTimersByTime(300);

      expect(harness.controller.getCurrentSvg()).toBe(before);
    });

    it("keeps SVG/PNG buttons enabled and disables WaveDrom on parse error", async () => {
      const harness = setupHarness();
      await harness.controller.init();
      harness.editor.value = "INVALID";
      harness.controller.handleInput();
      vi.advanceTimersByTime(300);

      expect(harness.saveSvgBtn.disabled).toBe(false);
      expect(harness.savePngBtn.disabled).toBe(false);
      expect(harness.wavedromBtn.disabled).toBe(true);
    });

    it("clears the status error and re-enables WaveDrom on recovery", async () => {
      const harness = setupHarness();
      await harness.controller.init();

      harness.editor.value = "INVALID";
      harness.controller.handleInput();
      vi.advanceTimersByTime(300);
      expect(harness.status.classList.contains("error")).toBe(true);
      expect(harness.wavedromBtn.disabled).toBe(true);

      harness.editor.value = "Clock _~_~";
      harness.controller.handleInput();
      vi.advanceTimersByTime(300);

      expect(harness.preview.innerHTML).toContain("<svg");
      expect(harness.status.classList.contains("error")).toBe(false);
      expect(harness.status.textContent).toBe("");
      expect(harness.saveSvgBtn.disabled).toBe(false);
      expect(harness.savePngBtn.disabled).toBe(false);
      expect(harness.wavedromBtn.disabled).toBe(false);
      expect(harness.controller.getCurrentSvg()).not.toBeNull();
    });

    it("shows a single underline element on parse error", async () => {
      const harness = setupHarnessWithUnderline();
      await harness.controller.init();
      harness.editor.value = "INVALID";
      harness.controller.handleInput();
      vi.advanceTimersByTime(300);

      const underlines = harness.underlineHost.querySelectorAll(".tcml-error-underline");
      expect(underlines.length).toBe(1);
    });

    it("clears the underline element on recovery", async () => {
      const harness = setupHarnessWithUnderline();
      await harness.controller.init();
      harness.editor.value = "INVALID";
      harness.controller.handleInput();
      vi.advanceTimersByTime(300);
      expect(harness.underlineHost.querySelectorAll(".tcml-error-underline").length).toBe(1);

      harness.editor.value = "Clock _~_~";
      harness.controller.handleInput();
      vi.advanceTimersByTime(300);

      expect(harness.underlineHost.querySelectorAll(".tcml-error-underline").length).toBe(0);
    });

    it("sets the underline title to the error message", async () => {
      const { renderTcml } = await import("../__mocks__/wasm");
      const renderMock = vi.mocked(renderTcml);
      renderMock.mockReturnValueOnce({
        svg: '<svg xmlns="http://www.w3.org/2000/svg" width="50" height="50"></svg>',
      });
      renderMock.mockReturnValueOnce({
        error: { line: 3, column: 2, length: 5, message: "specific underline message" },
      });

      const harness = setupHarnessWithUnderline();
      await harness.controller.init();
      harness.editor.value = "broken";
      harness.controller.handleInput();
      vi.advanceTimersByTime(300);

      const node = harness.underlineHost.querySelector(".tcml-error-underline");
      expect(node?.getAttribute("title")).toBe("specific underline message");
    });

    it("draws an underline spanning the full length when length > 1 (e.g. `(_=3,~3` no-close paren)", async () => {
      // Simulates the `@clock(_=3,~3` (no closing `)`) shape that the core
      // parser reports with line=1, column=7, length=7.
      const { renderTcml } = await import("../__mocks__/wasm");
      const renderMock = vi.mocked(renderTcml);
      renderMock.mockReturnValueOnce({
        svg: '<svg xmlns="http://www.w3.org/2000/svg" width="50" height="50"></svg>',
      });
      renderMock.mockReturnValueOnce({
        error: {
          line: 1,
          column: 7,
          length: 7,
          message: '@clock has an invalid attribute: "(_=3,~3"',
        },
      });

      const harness = setupHarnessWithUnderline();
      await harness.controller.init();
      harness.editor.value = "broken";
      harness.controller.handleInput();
      vi.advanceTimersByTime(300);

      const node = harness.underlineHost.querySelector(".tcml-error-underline");
      expect(node).not.toBeNull();
      // The inner NBSP-filled span carries the underline width; assert that
      // span's text length matches the reported `length` so the wavy
      // text-decoration covers all 7 characters.
      const inner = node!.querySelector("*");
      expect(inner).not.toBeNull();
      expect(inner!.textContent!.length).toBe(7);
      expect(node!.getAttribute("title")).toContain('"(_=3,~3"');
    });

    it("preserves a single-character underline when length = 1 (e.g. `Sig _~]_`)", async () => {
      // Simulates `Sig _~]_` (UnopenedHighlightEnd) which the core parser
      // reports with length=1 so the caret sits on the `]` only.
      const { renderTcml } = await import("../__mocks__/wasm");
      const renderMock = vi.mocked(renderTcml);
      renderMock.mockReturnValueOnce({
        svg: '<svg xmlns="http://www.w3.org/2000/svg" width="50" height="50"></svg>',
      });
      renderMock.mockReturnValueOnce({
        error: {
          line: 1,
          column: 7,
          length: 1,
          message: "`]` has no matching `[`",
        },
      });

      const harness = setupHarnessWithUnderline();
      await harness.controller.init();
      harness.editor.value = "broken";
      harness.controller.handleInput();
      vi.advanceTimersByTime(300);

      const node = harness.underlineHost.querySelector(".tcml-error-underline");
      expect(node).not.toBeNull();
      const inner = node!.querySelector("*");
      expect(inner!.textContent!.length).toBe(1);
    });
  });

  describe("empty input", () => {
    it("handles empty input without crashing", async () => {
      const harness = setupHarness();
      await harness.controller.init();
      harness.editor.value = "";
      harness.controller.handleInput();
      vi.advanceTimersByTime(300);

      expect(harness.preview.innerHTML).toBeTruthy();
    });
  });

  describe("instance isolation", () => {
    it("keeps state separate between two controller instances", async () => {
      const firstHarness = setupHarness();
      await firstHarness.controller.init();
      const firstSvg = firstHarness.controller.getCurrentSvg();
      expect(firstSvg).not.toBeNull();

      // Build a fresh DOM + controller; the first controller's cached SVG
      // must not bleed into the second instance before it renders anything.
      const secondHarness = setupHarness();
      expect(secondHarness.controller.getCurrentSvg()).toBeNull();
    });
  });
});
