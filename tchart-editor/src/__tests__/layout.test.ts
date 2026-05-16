import { describe, it, expect, beforeEach } from "vitest";
import { readFileSync } from "fs";
import { resolve } from "path";

const STYLE_CSS_PATH = resolve(__dirname, "../style.css");
const STYLE_CSS = readFileSync(STYLE_CSS_PATH, "utf-8");

function matchOrThrow(source: string, pattern: RegExp, label: string): RegExpMatchArray {
  const match = source.match(pattern);
  if (match === null) {
    throw new Error(`expected ${label} to match in style.css`);
  }
  return match;
}

describe("Layout", () => {
  beforeEach(() => {
    const html = readFileSync(resolve(__dirname, "../../index.html"), "utf-8");
    // Extract body content, excluding script tags to avoid happy-dom fetch errors
    const bodyMatch = html.match(/<body[^>]*>([\s\S]*)<\/body>/i);
    const bodyContent = bodyMatch ? bodyMatch[1] : "";
    document.body.innerHTML = bodyContent.replace(/<script[\s\S]*?<\/script>/gi, "");
  });

  it("should have a toolbar", () => {
    const toolbar = document.querySelector(".toolbar");
    expect(toolbar).not.toBeNull();
  });

  it("should have Save SVG button in toolbar", () => {
    const btn = document.querySelector("#save-svg") as HTMLButtonElement;
    expect(btn).not.toBeNull();
    // The icon now embeds a "SVG" <text> glyph that bleeds into textContent, so
    // verify the human-readable label specifically.
    const label = btn.querySelector(".btn-label");
    expect(label?.textContent).toBe("Save SVG");
  });

  it("should have Save PNG button in toolbar", () => {
    const btn = document.querySelector("#save-png") as HTMLButtonElement;
    expect(btn).not.toBeNull();
    const label = btn.querySelector(".btn-label");
    expect(label?.textContent).toBe("Save PNG");
  });

  it("should have an editor pane with the editor element", () => {
    const pane = document.querySelector(".editor-pane");
    expect(pane).not.toBeNull();
    const editor = pane!.querySelector("#editor");
    expect(editor).not.toBeNull();
  });

  it("should have a preview pane with preview div", () => {
    const pane = document.querySelector(".preview-pane");
    expect(pane).not.toBeNull();
    const preview = pane!.querySelector("#preview");
    expect(preview).not.toBeNull();
  });

  it("should have editor and preview panes side by side in a container", () => {
    const container = document.querySelector(".container");
    expect(container).not.toBeNull();
    const editorPane = container!.querySelector(".editor-pane");
    const previewPane = container!.querySelector(".preview-pane");
    expect(editorPane).not.toBeNull();
    expect(previewPane).not.toBeNull();
  });

  it("style.css does not override code-input's grid layout on #editor", () => {
    // code-input requires `display: grid` on the host so the inner <textarea>
    // and highlight overlay <pre> share `grid-area: 1 / 1` and overlap. An
    // app-side `#editor { display: ... }` rule has higher specificity than
    // `code-input { ... }` and silently breaks the overlay (textarea becomes
    // invisible because code-input renders text as `color: transparent`).
    const ruleMatch = matchOrThrow(STYLE_CSS, /#editor\s*\{([^}]+)\}/, "#editor rule");
    const ruleBody = ruleMatch[1];
    expect(ruleBody).not.toMatch(/(^|[\s;])display\s*:/);
  });

  it("zeroes the host margin on #editor so the default editor pane has no scrollbars", () => {
    // `@webcoder49/code-input`'s bundled CSS ships `code-input { margin: 8px }`
    // at element-selector specificity. The reset `* { margin: 0 }` (specificity
    // 0) loses to the element selector (specificity 1), leaving an 8px outer
    // margin on `#editor`. Combined with `width: 100%; height: 100%`, that
    // pushes the host 16px past the editor pane and forces both axes of the
    // pane to grow scrollbars even when the visible TCML easily fits. An
    // `#editor` id-selector rule of `margin: 0` beats the bundled rule and
    // removes the spurious overflow without touching the library.
    const ruleMatch = matchOrThrow(STYLE_CSS, /#editor\s*\{([^}]+)\}/, "#editor rule");
    const ruleBody = ruleMatch[1];
    expect(ruleBody).toMatch(/(^|[\s;])margin\s*:\s*0\s*(;|$)/);
  });

  it("keeps overflow:auto on .editor-pane so long TCML can still scroll", () => {
    // Zeroing the host margin on `#editor` must not silently disable scrolling
    // on the pane itself. When a TCML pasted by the user exceeds the pane in
    // either dimension, the browser must still show scrollbars on
    // `.editor-pane` (or let code-input's own `overflow: auto` kick in on the
    // host). Either way, the pane's `overflow: auto` declaration is what makes
    // content scrollable when it really overflows.
    const ruleMatch = matchOrThrow(
      STYLE_CSS,
      /\.editor-pane\s*,\s*\.preview-pane\s*\{([^}]+)\}/,
      ".editor-pane, .preview-pane rule",
    );
    const ruleBody = ruleMatch[1];
    expect(ruleBody).toMatch(/overflow\s*:\s*auto/);
  });
});
