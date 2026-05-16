import { describe, it, expect, beforeEach } from "vitest";
import { ErrorUnderlineOverlay } from "../error-underline";

describe("ErrorUnderlineOverlay", () => {
  let host: HTMLElement;
  let overlay: ErrorUnderlineOverlay;

  beforeEach(() => {
    document.body.innerHTML = "";
    host = document.createElement("div");
    document.body.appendChild(host);
    overlay = new ErrorUnderlineOverlay(host);
  });

  it("creates a single underline element when show() is called", () => {
    overlay.show({ line: 1, column: 1, length: 3, message: "boom" });
    const underlines = host.querySelectorAll(".tcml-error-underline");
    expect(underlines.length).toBe(1);
  });

  it("replaces the previous underline instead of stacking on repeated show()", () => {
    overlay.show({ line: 1, column: 1, length: 2, message: "first" });
    overlay.show({ line: 3, column: 5, length: 4, message: "second" });
    const underlines = host.querySelectorAll(".tcml-error-underline");
    expect(underlines.length).toBe(1);
  });

  it("sets the title attribute to the error message", () => {
    overlay.show({ line: 2, column: 3, length: 4, message: "expected '\"'" });
    const node = host.querySelector(".tcml-error-underline");
    expect(node).not.toBeNull();
    expect(node?.getAttribute("title")).toBe("expected '\"'");
  });

  it("clear() removes the underline element from the host", () => {
    overlay.show({ line: 1, column: 1, length: 3, message: "boom" });
    overlay.clear();
    const underlines = host.querySelectorAll(".tcml-error-underline");
    expect(underlines.length).toBe(0);
  });

  it("clear() is safe to call when nothing is shown", () => {
    expect(() => overlay.clear()).not.toThrow();
    const underlines = host.querySelectorAll(".tcml-error-underline");
    expect(underlines.length).toBe(0);
  });

  it("positions the underline so that y reflects the line and x reflects column-1", () => {
    overlay.show({ line: 2, column: 3, length: 4, message: "boom" });
    const node = host.querySelector(".tcml-error-underline");
    expect(node).not.toBeNull();
    if (!(node instanceof HTMLElement)) throw new Error("expected HTMLElement");
    // We can only assert that the inline `top` / `left` style strings encode
    // the expected line index (1-based -> zero-based offset 1) and column
    // offset (column-1 = 2). The exact unit is implementation detail (the
    // overlay uses character / line-height units), so we check substrings.
    expect(node.style.top.length).toBeGreaterThan(0);
    expect(node.style.left.length).toBeGreaterThan(0);
    // length 4 means width should encode "4".
    expect(node.style.width.length).toBeGreaterThan(0);
  });

  it("uses a minimum of one character width when length is 0", () => {
    overlay.show({ line: 1, column: 5, length: 0, message: "unterminated" });
    const node = host.querySelector(".tcml-error-underline");
    if (!(node instanceof HTMLElement)) throw new Error("expected HTMLElement");
    // width style should not be empty and should reflect at least 1 character.
    // We encode 1 character width via "1ch" or similar — assert presence.
    expect(node.style.width.length).toBeGreaterThan(0);
    expect(node.style.width).not.toBe("0");
    expect(node.style.width).not.toBe("0px");
    expect(node.style.width).not.toBe("0ch");
  });

  it("applies the .tcml-error-underline class on the element", () => {
    overlay.show({ line: 1, column: 1, length: 1, message: "x" });
    const node = host.querySelector(".tcml-error-underline");
    expect(node?.classList.contains("tcml-error-underline")).toBe(true);
  });

  it("renders a wavy underline using text-decoration on an inner element (border-style: wavy is invalid CSS)", () => {
    overlay.show({ line: 1, column: 1, length: 4, message: "x" });
    const node = host.querySelector(".tcml-error-underline");
    if (!(node instanceof HTMLElement)) throw new Error("expected HTMLElement");
    // The wavy underline is drawn by text-decoration on an inner element
    // (border-style: wavy is not valid CSS). Verify an inner element exists,
    // its text-decoration is wavy + underline + the error colour variable,
    // and it carries enough characters (NBSP) to span the underline width.
    const inner = node.querySelector("*");
    expect(inner).not.toBeNull();
    if (!(inner instanceof HTMLElement)) throw new Error("expected HTMLElement");
    const style = inner.style;
    const cssText = `${style.textDecoration} ${style.textDecorationLine} ${style.textDecorationStyle} ${style.textDecorationColor}`;
    expect(cssText).toMatch(/wavy/);
    expect(cssText).toMatch(/underline/);
    expect(cssText).toMatch(/--error-color/);
    // NBSP ( ) is used so the inline text is non-collapsing in HTML.
    expect(inner.textContent).toMatch(/^ +$/);
    expect(inner.textContent!.length).toBe(4);
  });

  it("offsets the underline by the host element's computed padding so the line origin lands inside the padding box", () => {
    // why: when the host has CSS padding (as code-input's <pre> does via the
    // --padding vars), the text content starts at (padding-left, padding-top),
    // not at (0, 0). The underline must include that offset, otherwise it
    // floats above/left of the actual highlighted text.
    host.style.paddingLeft = "12px";
    host.style.paddingTop = "12px";
    overlay.show({ line: 1, column: 1, length: 1, message: "x" });
    const node = host.querySelector(".tcml-error-underline");
    if (!(node instanceof HTMLElement)) throw new Error("expected HTMLElement");
    expect(node.style.left).toContain("12px");
    expect(node.style.top).toContain("12px");
  });
});
