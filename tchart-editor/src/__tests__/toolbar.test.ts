import { describe, it, expect, beforeEach } from "vitest";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const STYLE_CSS_PATH = resolve(HERE, "../style.css");
const INDEX_HTML_PATH = resolve(HERE, "../../index.html");

const STYLE_CSS = readFileSync(STYLE_CSS_PATH, "utf-8");

/**
 * Load index.html into the test DOM with all `<script>` tags removed so
 * happy-dom does not try to fetch the module entry point.
 */
function loadIndexHtml(): void {
  const html = readFileSync(INDEX_HTML_PATH, "utf-8");
  const bodyMatch = html.match(/<body[^>]*>([\s\S]*)<\/body>/i);
  const bodyContent = bodyMatch !== null ? (bodyMatch[1] ?? "") : "";
  document.body.innerHTML = bodyContent.replace(/<script[\s\S]*?<\/script>/gi, "");
}

const ACTION_BUTTON_IDS = ["load", "save-svg", "save-png", "save-wavedrom", "help"] as const;

// WaveDrom is a third-party project name; the spec keeps that button as
// text-only so we never imply ownership of its visual identity.
const ICONIFIED_BUTTON_IDS = ["load", "save-svg", "save-png", "help", "license"] as const;

describe("Toolbar three-zone layout", () => {
  beforeEach(() => {
    loadIndexHtml();
  });

  it("contains left / center / right zone containers inside the toolbar", () => {
    const toolbar = document.querySelector(".toolbar");
    expect(toolbar).not.toBeNull();
    if (toolbar === null) return;
    const left = toolbar.querySelector(".toolbar-left");
    const center = toolbar.querySelector(".toolbar-center");
    const right = toolbar.querySelector(".toolbar-right");
    expect(left).not.toBeNull();
    expect(center).not.toBeNull();
    expect(right).not.toBeNull();
  });

  it("places Load / Save SVG / Save PNG / WaveDrom / Help inside the left zone", () => {
    const left = document.querySelector(".toolbar-left");
    expect(left).not.toBeNull();
    if (left === null) return;
    for (const id of ACTION_BUTTON_IDS) {
      const button = left.querySelector(`#${id}`);
      expect(button, `${id} expected in left zone`).not.toBeNull();
    }
  });

  it("does not place the left-zone action buttons in the center or right zones", () => {
    const center = document.querySelector(".toolbar-center");
    const right = document.querySelector(".toolbar-right");
    expect(center).not.toBeNull();
    expect(right).not.toBeNull();
    if (center === null || right === null) return;
    for (const id of ACTION_BUTTON_IDS) {
      expect(center.querySelector(`#${id}`), `${id} must not be in center`).toBeNull();
      expect(right.querySelector(`#${id}`), `${id} must not be in right`).toBeNull();
    }
  });

  it("renders the text logo `tchart rust editor` in the center zone", () => {
    const center = document.querySelector(".toolbar-center");
    expect(center).not.toBeNull();
    if (center === null) return;
    const text = (center.textContent ?? "").replace(/\s+/g, " ").trim();
    expect(text).toBe("tchart rust editor");
  });

  it("renders the privacy note and the License button in the right zone", () => {
    const right = document.querySelector(".toolbar-right");
    expect(right).not.toBeNull();
    if (right === null) return;
    const privacy = right.querySelector("#privacy-note");
    const licenseButton = right.querySelector("#license");
    expect(privacy).not.toBeNull();
    expect(licenseButton).not.toBeNull();
    expect(licenseButton instanceof HTMLButtonElement).toBe(true);
    const left = document.querySelector(".toolbar-left");
    const center = document.querySelector(".toolbar-center");
    expect(left).not.toBeNull();
    expect(center).not.toBeNull();
    if (left === null || center === null) return;
    expect(left.querySelector("#license")).toBeNull();
    expect(center.querySelector("#license")).toBeNull();
  });
});

describe("Toolbar action button structure", () => {
  beforeEach(() => {
    loadIndexHtml();
  });

  it("each iconified button contains an inline <svg> and a non-empty text label", () => {
    for (const id of ICONIFIED_BUTTON_IDS) {
      const button = document.getElementById(id);
      expect(button, `${id} button missing`).not.toBeNull();
      if (button === null) continue;
      const svg = button.querySelector("svg");
      expect(svg, `${id} must contain an inline <svg>`).not.toBeNull();
      const label = (button.textContent ?? "").trim();
      expect(label.length, `${id} must have a non-empty text label`).toBeGreaterThan(0);
    }
  });

  it("WaveDrom button is text-only without any <svg>", () => {
    const button = document.getElementById("save-wavedrom");
    expect(button).not.toBeNull();
    if (button === null) return;
    expect(button.querySelector("svg"), "WaveDrom button must not contain an <svg>").toBeNull();
    expect((button.textContent ?? "").trim()).toContain("WaveDrom");
  });

  it("Save SVG and Save PNG render distinct icon paths", () => {
    const svg = document.getElementById("save-svg");
    const png = document.getElementById("save-png");
    expect(svg).not.toBeNull();
    expect(png).not.toBeNull();
    if (svg === null || png === null) return;
    const svgIcon = svg.querySelector("svg");
    const pngIcon = png.querySelector("svg");
    expect(svgIcon).not.toBeNull();
    expect(pngIcon).not.toBeNull();
    if (svgIcon === null || pngIcon === null) return;
    expect(svgIcon.innerHTML.trim()).not.toBe(pngIcon.innerHTML.trim());
  });

  it("rejects external icon references such as <img src> or <use href> on the iconified buttons", () => {
    for (const id of ICONIFIED_BUTTON_IDS) {
      const button = document.getElementById(id);
      expect(button).not.toBeNull();
      if (button === null) continue;
      expect(button.querySelector("img"), `${id} must not use <img>`).toBeNull();
      expect(button.querySelector("use"), `${id} must not use <use>`).toBeNull();
    }
  });

  it("places the icon to the left of the label inside each iconified button", () => {
    for (const id of ICONIFIED_BUTTON_IDS) {
      const button = document.getElementById(id);
      expect(button).not.toBeNull();
      if (button === null) continue;
      const children = Array.from(button.children);
      const svgIndex = children.findIndex((child) => child.tagName.toLowerCase() === "svg");
      const labelIndex = children.findIndex(
        (child) => child.classList.contains("btn-label") || child.tagName.toLowerCase() === "span",
      );
      expect(svgIndex, `${id} must contain an svg child`).toBeGreaterThanOrEqual(0);
      expect(labelIndex, `${id} must contain a label child`).toBeGreaterThanOrEqual(0);
      expect(svgIndex, `${id} svg must appear before its label`).toBeLessThan(labelIndex);
    }
  });
});

describe("Toolbar background decoration", () => {
  it("uses a gradient background on the toolbar (not a single solid color)", () => {
    // The computed style of a CSS gradient is not reliably populated by
    // happy-dom, so we read the source rule directly. Either the toolbar
    // selector or one of its zone selectors must declare a `*-gradient(...)`
    // background-image.
    const toolbarRuleMatch = STYLE_CSS.match(/\.toolbar\b[^{]*\{([^}]+)\}/);
    expect(toolbarRuleMatch).not.toBeNull();
    if (toolbarRuleMatch === null) return;
    const body = toolbarRuleMatch[1] ?? "";
    const gradientPattern = /(background|background-image)\s*:\s*[^;]*-gradient\s*\(/;
    expect(body).toMatch(gradientPattern);
  });
});
