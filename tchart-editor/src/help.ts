import helpJa from "../../help/output/tcml-format.html?raw";
import helpEn from "../../help/output/tcml-format.en.html?raw";
import { detectUiLang, type UiLang } from "./lib/lang";

const MODAL_ID = "help-modal";

function stripInternalLangSwitch(html: string): string {
  return html.replace(/<p class="lang-switch">[\s\S]*?<\/p>/, "");
}

const HELP_JA = stripInternalLangSwitch(helpJa);
const HELP_EN = stripInternalLangSwitch(helpEn);

function helpHtml(lang: UiLang): string {
  return lang === "ja" ? HELP_JA : HELP_EN;
}

/**
 * Open the help modal. No-op if it is already open. The modal can be closed
 * via the Escape key, a click on the backdrop, or the close button.
 */
export function openHelpModal(): void {
  if (document.getElementById(MODAL_ID)) return;

  let currentLang: UiLang = detectUiLang();

  const overlay = document.createElement("div");
  overlay.id = MODAL_ID;
  overlay.className = "help-overlay";

  const dialog = document.createElement("div");
  dialog.className = "help-dialog";
  dialog.setAttribute("role", "dialog");
  dialog.setAttribute("aria-modal", "true");

  const langBtn = document.createElement("button");
  langBtn.className = "help-lang";
  langBtn.type = "button";

  const closeBtn = document.createElement("button");
  closeBtn.className = "help-close";
  closeBtn.setAttribute("aria-label", "Close help");
  closeBtn.textContent = "×";

  const iframe = document.createElement("iframe");
  iframe.className = "help-iframe";
  iframe.title = "TCML help";

  const apply = () => {
    iframe.srcdoc = helpHtml(currentLang);
    langBtn.textContent = currentLang === "ja" ? "EN" : "JA";
    langBtn.setAttribute(
      "aria-label",
      currentLang === "ja" ? "Switch to English" : "Switch to Japanese",
    );
  };
  apply();

  langBtn.addEventListener("click", () => {
    currentLang = currentLang === "ja" ? "en" : "ja";
    apply();
  });

  dialog.append(langBtn, closeBtn, iframe);
  overlay.append(dialog);
  document.body.append(overlay);

  // why: the Escape handler is registered on `document`, so the close button
  // and backdrop-click paths must also remove it; otherwise listeners pile
  // up each time the modal is reopened.
  const closeAndCleanup = () => {
    document.removeEventListener("keydown", onKeyDown);
    closeHelpModal();
  };
  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape") {
      closeAndCleanup();
    }
  };
  closeBtn.addEventListener("click", closeAndCleanup);
  overlay.addEventListener("click", (event) => {
    if (event.target === overlay) closeAndCleanup();
  });
  document.addEventListener("keydown", onKeyDown);
}

/**
 * Remove the help modal from the DOM if it is open. Listener cleanup is
 * handled inside the closure set up by `openHelpModal`.
 */
export function closeHelpModal(): void {
  const overlay = document.getElementById(MODAL_ID);
  if (overlay) overlay.remove();
}

/**
 * Return whether the help modal is currently mounted in the DOM.
 */
export function isHelpModalOpen(): boolean {
  return document.getElementById(MODAL_ID) !== null;
}
