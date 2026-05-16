import { setupTabHandler } from "./editor";
import { PreviewController, type OutputButtons } from "./preview";
import { downloadSvg, downloadPng } from "./export";
import { handleWaveDromClick } from "./wavedrom";
import { openHelpModal } from "./help";
import { openLicenseModal } from "./license";
import { loadFile } from "./load";
import { requireElement } from "./lib/dom";
import { extractErrorMessage } from "./lib/errors";
import { detectUiLang } from "./lib/lang";
import {
  setupCodeInputTemplate,
  requireCodeInputElement,
  type CodeInputElement,
} from "./editor-element";
import "@webcoder49/code-input/code-input.css";
import "./style.css";

const PRIVACY_JA = "このページは外部に情報を送信しません";
const PRIVACY_EN = "No data is sent externally";

function start(): void {
  // Register the TCML Prism grammar + matching code-input template before
  // the browser upgrades the <code-input> custom element.
  setupCodeInputTemplate();

  const editor = requireCodeInputElement("editor");
  const editorPane = editor.parentElement;
  if (editorPane === null) {
    throw new Error("editor element has no parent pane");
  }
  // The error underline must overlay the highlighted text, which lives inside
  // code-input's inner `<pre>`. Using that as the host (instead of the outer
  // pane) keeps the underline's absolute positioning rooted at the same
  // padding box as the text, so column / line offsets map 1:1 to character
  // positions. The custom element's connectedCallback creates the <pre>
  // synchronously on upgrade, which has already happened by here because
  // `setupCodeInputTemplate` ran above and registered the element.
  const underlineHost = editor.preElement;
  if (underlineHost === undefined) {
    throw new Error("code-input has not initialised its <pre> overlay");
  }
  const preview = requireElement("preview", HTMLElement);
  const saveSvgBtn = requireElement("save-svg", HTMLButtonElement);
  const savePngBtn = requireElement("save-png", HTMLButtonElement);
  const wavedromBtn = requireElement("save-wavedrom", HTMLButtonElement);
  const helpBtn = requireElement("help", HTMLButtonElement);
  const licenseBtn = requireElement("license", HTMLButtonElement);
  const status = requireElement("status", HTMLElement);
  const loadBtn = requireElement("load", HTMLButtonElement);
  const fileInput = requireElement("file-input", HTMLInputElement);
  const privacyNote = requireElement("privacy-note", HTMLElement);
  privacyNote.textContent = detectUiLang() === "ja" ? PRIVACY_JA : PRIVACY_EN;

  const appVersion = requireElement("app-version", HTMLElement);
  appVersion.textContent = `v${__APP_VERSION__}`;

  const buttons: OutputButtons = {
    saveSvg: saveSvgBtn,
    savePng: savePngBtn,
    wavedrom: wavedromBtn,
  };

  const controller = new PreviewController({
    editor,
    preview,
    buttons,
    status,
    underlineHost,
  });

  attachTabHandler(editor);

  editor.addEventListener("input", () => {
    controller.handleInput();
  });

  saveSvgBtn.addEventListener("click", () => {
    const svg = controller.getCurrentSvg();
    if (svg !== null) downloadSvg(svg);
  });

  savePngBtn.addEventListener("click", () => {
    const svg = controller.getCurrentSvg();
    if (svg !== null) downloadPng(svg, editor.value);
  });

  wavedromBtn.addEventListener("click", () => {
    handleWaveDromClick(editor.value, status);
  });

  helpBtn.addEventListener("click", openHelpModal);
  licenseBtn.addEventListener("click", openLicenseModal);

  loadBtn.addEventListener("click", () => fileInput.click());

  fileInput.addEventListener("change", () => {
    const file = fileInput.files?.[0];
    fileInput.value = "";
    if (file) {
      void handleLoad(file, editor, controller, status);
    }
  });

  void controller.init();
}

/**
 * Attach the tab handler to the inner `<textarea>` once code-input has
 * upgraded the custom element and exposed it. If the textarea is not yet
 * present (the upgrade is asynchronous), listen for the `code-input_load`
 * event which code-input dispatches when the textarea is wired up.
 */
function attachTabHandler(editor: CodeInputElement): void {
  const textareaElement = editor.textareaElement;
  if (textareaElement !== undefined) {
    setupTabHandler(textareaElement);
    return;
  }
  editor.addEventListener(
    "code-input_load",
    () => {
      const ready = editor.textareaElement;
      if (ready !== undefined) setupTabHandler(ready);
    },
    { once: true },
  );
}

async function handleLoad(
  file: File,
  editor: CodeInputElement,
  controller: PreviewController,
  status: HTMLElement,
): Promise<void> {
  try {
    const tcml = await loadFile(file);
    if (tcml === null) {
      status.textContent = `Load failed: no tchart source found in ${file.name}`;
      status.classList.add("error");
      return;
    }
    editor.value = tcml;
    status.textContent = `Loaded ${file.name}`;
    status.classList.remove("error");
    controller.handleInput();
  } catch (error: unknown) {
    const message = extractErrorMessage(error);
    status.textContent = `Load failed: ${message}`;
    status.classList.add("error");
  }
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", start, { once: true });
} else {
  start();
}
