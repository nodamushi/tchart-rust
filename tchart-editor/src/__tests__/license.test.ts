import { describe, it, expect, beforeEach, afterEach } from "vitest";

import { openLicenseModal, closeLicenseModal, isLicenseModalOpen } from "../license";
import { openHelpModal, closeHelpModal, isHelpModalOpen } from "../help";
import { LICENSE_GROUPS } from "../generated/licenses";

const ALL_LIBRARIES = LICENSE_GROUPS.flatMap((group) => group.libraries);

const MODAL_ID = "license-modal";

describe("License modal", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  afterEach(() => {
    closeLicenseModal();
    closeHelpModal();
  });

  it("opens a modal containing license-body groups", () => {
    openLicenseModal();
    expect(isLicenseModalOpen()).toBe(true);
    const overlay = document.getElementById(MODAL_ID);
    expect(overlay).not.toBeNull();
    if (overlay === null) return;
    const groups = overlay.querySelectorAll(".license-group");
    expect(groups.length).toBeGreaterThan(0);
  });

  it("closes the modal when Escape is pressed", () => {
    openLicenseModal();
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    expect(isLicenseModalOpen()).toBe(false);
  });

  it("closes the modal when the close button is clicked", () => {
    openLicenseModal();
    const closeBtn = document.querySelector(".license-close");
    expect(closeBtn).not.toBeNull();
    if (closeBtn === null) return;
    expect(closeBtn instanceof HTMLButtonElement).toBe(true);
    if (!(closeBtn instanceof HTMLButtonElement)) return;
    closeBtn.click();
    expect(isLicenseModalOpen()).toBe(false);
  });

  it("does not open multiple modals on repeated open calls", () => {
    openLicenseModal();
    openLicenseModal();
    expect(document.querySelectorAll(`#${MODAL_ID}`).length).toBe(1);
  });

  it("does not leak modal DOM after repeated open/close cycles", () => {
    for (let cycle = 0; cycle < 5; cycle++) {
      openLicenseModal();
      closeLicenseModal();
    }
    expect(document.querySelectorAll(`#${MODAL_ID}`).length).toBe(0);
    openLicenseModal();
    expect(document.querySelectorAll(`#${MODAL_ID}`).length).toBe(1);
  });

  it("locks body scrolling while the modal is open and restores it on close", () => {
    const previous = document.body.style.overflow;
    openLicenseModal();
    expect(document.body.style.overflow).toBe("hidden");
    closeLicenseModal();
    expect(document.body.style.overflow).toBe(previous);
  });

  it("renders SPDX label, license body once and a library list per group", () => {
    openLicenseModal();
    const groups = document.querySelectorAll(".license-group");
    expect(groups.length).toBe(LICENSE_GROUPS.length);
    for (const group of Array.from(groups)) {
      const spdx = group.querySelector(".license-spdx");
      const body = group.querySelector(".license-text");
      const libraries = group.querySelectorAll(".license-library");
      expect(spdx).not.toBeNull();
      expect(body).not.toBeNull();
      expect(libraries.length).toBeGreaterThan(0);
      // Body must appear exactly once per group, never duplicated.
      expect(group.querySelectorAll(".license-text").length).toBe(1);
      if (spdx === null || body === null) continue;
      expect((spdx.textContent ?? "").trim().length).toBeGreaterThan(0);
      expect((body.textContent ?? "").trim().length).toBeGreaterThan(0);
      for (const library of Array.from(libraries)) {
        const name = library.querySelector(".license-name");
        const version = library.querySelector(".license-version");
        expect(name).not.toBeNull();
        expect(version).not.toBeNull();
        if (name === null || version === null) continue;
        expect((name.textContent ?? "").trim().length).toBeGreaterThan(0);
        expect((version.textContent ?? "").trim().length).toBeGreaterThan(0);
      }
    }
  });

  it("never shows the same normalized license body in more than one group", () => {
    const keys = new Set();
    for (const group of LICENSE_GROUPS) {
      const key = group.body.replace(/\s+$/gm, "").trim();
      expect(keys.has(key), `duplicate license body across groups`).toBe(false);
      keys.add(key);
    }
  });
});

describe("License data integrity", () => {
  it("includes the TypeScript runtime dependencies", () => {
    const names = ALL_LIBRARIES.map((library) => library.name);
    expect(names).toContain("@webcoder49/code-input");
    expect(names).toContain("prismjs");
  });

  it("includes at least one wasm-side Rust runtime crate", () => {
    const rust = ALL_LIBRARIES.filter((library) => library.origin === "rust");
    expect(rust.length).toBeGreaterThan(0);
  });

  it("excludes the project's own crates / packages", () => {
    const own = new Set(["tchart-core", "tchart-cli", "tchart-web", "tchart-editor"]);
    for (const library of ALL_LIBRARIES) {
      expect(own.has(library.name), `unexpected own crate: ${library.name}`).toBe(false);
    }
  });

  it("every group has a non-empty license body", () => {
    for (const group of LICENSE_GROUPS) {
      expect(group.body.trim().length, `${group.spdx} missing body`).toBeGreaterThan(0);
    }
  });
});

describe("License modal isolation from Help modal", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  afterEach(() => {
    closeLicenseModal();
    closeHelpModal();
  });

  it("opening the License modal while Help is open does not stack both", () => {
    openHelpModal();
    expect(isHelpModalOpen()).toBe(true);
    openLicenseModal();
    // Spec allows either: Help auto-closes, or License is suppressed. In any
    // case both must not remain visible simultaneously.
    const helpOpen = isHelpModalOpen();
    const licenseOpen = isLicenseModalOpen();
    expect(helpOpen && licenseOpen).toBe(false);
  });

  it("closing the License modal does not also close a separately opened Help modal afterwards", () => {
    openLicenseModal();
    closeLicenseModal();
    openHelpModal();
    expect(isHelpModalOpen()).toBe(true);
    expect(isLicenseModalOpen()).toBe(false);
  });
});
