import { describe, it, expect, beforeEach, afterEach } from "vitest";

import { openHelpModal, closeHelpModal, isHelpModalOpen } from "../help";

describe("Help modal", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  afterEach(() => {
    closeHelpModal();
  });

  it("opens a modal with iframe whose srcdoc embeds the help HTML", () => {
    openHelpModal();

    expect(isHelpModalOpen()).toBe(true);
    const overlay = document.getElementById("help-modal");
    expect(overlay).not.toBeNull();
    const iframe = overlay!.querySelector("iframe");
    expect(iframe).not.toBeNull();
    expect(iframe!.getAttribute("src")).toBeNull();
    const srcdoc = iframe!.getAttribute("srcdoc") ?? "";
    expect(srcdoc).toContain("<!DOCTYPE html>");
    expect(srcdoc).toMatch(/<html lang="(ja|en)">/);
  });

  it("does not open multiple modals when clicked twice", () => {
    openHelpModal();
    openHelpModal();

    const overlays = document.querySelectorAll("#help-modal");
    expect(overlays.length).toBe(1);
  });

  it("closes when × button is clicked", () => {
    openHelpModal();
    const closeBtn = document.querySelector(".help-close") as HTMLButtonElement;
    closeBtn.click();

    expect(isHelpModalOpen()).toBe(false);
  });

  it("closes when overlay background is clicked", () => {
    openHelpModal();
    const overlay = document.getElementById("help-modal") as HTMLElement;
    overlay.dispatchEvent(new MouseEvent("click", { bubbles: true, composed: true }));

    expect(isHelpModalOpen()).toBe(false);
  });

  it("closes when Escape key is pressed", () => {
    openHelpModal();
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));

    expect(isHelpModalOpen()).toBe(false);
  });

  it("does not close when clicking inside the dialog", () => {
    openHelpModal();
    const dialog = document.querySelector(".help-dialog") as HTMLElement;
    dialog.dispatchEvent(new MouseEvent("click", { bubbles: true, composed: true }));

    expect(isHelpModalOpen()).toBe(true);
  });

  it("embeds an in-iframe TOC click handler so srcdoc anchor links do not escape to the parent frame", () => {
    openHelpModal();
    const iframe = document.querySelector("iframe") as HTMLIFrameElement;
    const srcdoc = iframe.getAttribute("srcdoc") ?? "";

    expect(srcdoc).toMatch(/data-tcml-toc-handler/);
    expect(srcdoc).toContain('addEventListener("click"');
    expect(srcdoc).toContain("preventDefault");
    expect(srcdoc).toContain("scrollIntoView");
  });
});
