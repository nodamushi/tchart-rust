import init, { renderTcml } from "tchart-web";
import { extractErrorMessage } from "./lib/errors";
import { ErrorUnderlineOverlay, type ParseErrorInfo } from "./error-underline";

/**
 * Sample TCML shown in the editor on first load. Kept in sync with the
 * example described in docs/spec/editor.md.
 */
const SAMPLE_TCML = `// Sample
@step 15
@slant 3

@clock(pos)
Clock  
Data   =D0====XD1====XD2====
Enable ____~~~~________`;

// why: 300ms is an empirical debounce window — short enough to feel
// responsive once typing pauses, long enough to skip re-renders during
// active typing.
const DEBOUNCE_MS = 300;

/**
 * Options object bundling references to the export buttons
 * (SVG / PNG / WaveDrom).
 */
export interface OutputButtons {
  readonly saveSvg: HTMLButtonElement;
  readonly savePng: HTMLButtonElement;
  readonly wavedrom: HTMLButtonElement;
}

/**
 * Minimal structural type for the editor element PreviewController needs.
 * In production this is a `<code-input>` custom element; in tests it is a
 * plain `<textarea>`. Both expose `value` and forward standard
 * `addEventListener("input", ...)` semantics.
 */
export interface PreviewEditorElement {
  value: string;
  addEventListener: HTMLElement["addEventListener"];
}

/**
 * Options object bundling the DOM element references required to construct
 * a {@link PreviewController}. Passed as a single argument so callers refer
 * to fields by name and cannot mis-order positional parameters.
 *
 * `underlineHost` is optional: when omitted no underline overlay is managed
 * (the controller still drives preview / status normally). Production code
 * (`main.ts`) passes the code-input overlay container; tests that don't
 * exercise the underline can leave it out.
 */
export interface PreviewControllerOptions {
  readonly editor: PreviewEditorElement;
  readonly preview: HTMLElement;
  readonly buttons: OutputButtons;
  readonly status: HTMLElement;
  readonly underlineHost?: HTMLElement;
}

/**
 * Shape of a render returned by the wasm `renderTcml` function. Modeled as a
 * discriminated union on `kind` so the exhaustiveness check in
 * `#renderPreview` is enforced by the type system rather than runtime
 * defensive throws.
 */
type RenderResult =
  | { readonly kind: "svg"; readonly svg: string }
  | { readonly kind: "error"; readonly error: ParseErrorInfo };

function isParseErrorInfo(value: unknown): value is ParseErrorInfo {
  if (typeof value !== "object" || value === null) return false;
  return (
    "line" in value &&
    typeof (value as { line: unknown }).line === "number" &&
    "column" in value &&
    typeof (value as { column: unknown }).column === "number" &&
    "length" in value &&
    typeof (value as { length: unknown }).length === "number" &&
    "message" in value &&
    typeof (value as { message: unknown }).message === "string"
  );
}

/**
 * Normalize the wasm-side `{ svg?, error? }` shape (see docs/spec/web.md
 * §renderTcml — exactly one of the two fields is populated) into the
 * internal {@link RenderResult} discriminated union. Returns `null` when
 * the value violates the contract so the caller can throw.
 */
function toRenderResult(value: unknown): RenderResult | null {
  if (typeof value !== "object" || value === null) return null;
  if ("svg" in value) {
    const candidate = (value as { svg: unknown }).svg;
    if (typeof candidate === "string") return { kind: "svg", svg: candidate };
  }
  if ("error" in value) {
    const candidate = (value as { error: unknown }).error;
    if (isParseErrorInfo(candidate)) return { kind: "error", error: candidate };
  }
  return null;
}

/**
 * Owns the editor preview lifecycle: WASM bootstrap, debounced re-render
 * in response to textarea input, and the rendered SVG cache used by the
 * SVG / PNG export buttons.
 *
 * State (last rendered SVG, parse-error flag, pending debounce timer) is
 * encapsulated as runtime-private fields so call sites cannot bypass the
 * controller and mutate it directly.
 */
export class PreviewController {
  readonly #editor: PreviewEditorElement;
  readonly #preview: HTMLElement;
  readonly #buttons: OutputButtons;
  readonly #status: HTMLElement;
  readonly #underline: ErrorUnderlineOverlay | null;

  #currentSvg: string | null = null;
  #hasParseError = false;
  #debounceTimer: ReturnType<typeof setTimeout> | null = null;

  constructor(options: PreviewControllerOptions) {
    this.#editor = options.editor;
    this.#preview = options.preview;
    this.#buttons = options.buttons;
    this.#status = options.status;
    this.#underline =
      options.underlineHost === undefined ? null : new ErrorUnderlineOverlay(options.underlineHost);
  }

  /**
   * Initialize the WASM module, seed the textarea with the sample TCML,
   * and render the first preview.
   */
  async init(): Promise<void> {
    await init();
    this.#editor.value = SAMPLE_TCML;
    this.#renderPreview(SAMPLE_TCML);
  }

  /**
   * Schedule a debounced re-render in response to a textarea `input` event.
   */
  handleInput(): void {
    if (this.#debounceTimer) clearTimeout(this.#debounceTimer);
    this.#debounceTimer = setTimeout(() => {
      this.#renderPreview(this.#editor.value);
    }, DEBOUNCE_MS);
  }

  /**
   * Return the most recently rendered SVG string, or `null` if no
   * successful render has happened yet.
   */
  getCurrentSvg(): string | null {
    return this.#currentSvg;
  }

  #renderPreview(text: string): void {
    try {
      const raw: unknown = renderTcml(text);
      const result = toRenderResult(raw);
      if (result === null) {
        throw new Error("renderTcml returned an unexpected shape");
      }
      switch (result.kind) {
        case "svg":
          this.#applySuccess(result.svg);
          break;
        case "error":
          this.#applyParseError(result.error);
          break;
        default: {
          const exhaustive: never = result;
          throw new Error(`unreachable RenderResult: ${String(exhaustive)}`);
        }
      }
    } catch (error: unknown) {
      // Internal / font / layout failures still throw. They are *not* parse
      // errors, so we don't draw an underline; we just surface the message.
      this.#hasParseError = false;
      const message = extractErrorMessage(error);
      this.#showNoPreviewPlaceholder();
      this.#status.textContent = `Render error: ${message}`;
      this.#status.classList.add("error");
      if (this.#underline !== null) this.#underline.clear();
    }
    this.#updateButtons();
  }

  #applySuccess(svg: string): void {
    this.#currentSvg = svg;
    this.#hasParseError = false;
    // why: renderTcml is the Rust SVG renderer (tchart_core::svg::render).
    // It owns escaping of every textual value reaching the SVG output, so the
    // string returned here has no unescaped user input. innerHTML assignment
    // is safe under that contract; if a future Rust change relaxes escaping,
    // this site has to be revisited.
    this.#preview.innerHTML = svg;
    this.#clearStatusError();
    if (this.#underline !== null) this.#underline.clear();
  }

  #applyParseError(error: ParseErrorInfo): void {
    this.#hasParseError = true;
    this.#showNoPreviewPlaceholder();
    this.#status.textContent = `Parse error: ${error.message}`;
    this.#status.classList.add("error");
    if (this.#underline !== null) this.#underline.show(error);
  }

  /**
   * Render the `(no preview)` placeholder in the right pane when no valid
   * SVG has ever been produced. Behaves as a no-op once a successful render
   * has populated `#currentSvg`, so callers can fire it unconditionally on
   * the error / exception paths without clobbering the previous good SVG.
   */
  #showNoPreviewPlaceholder(): void {
    if (this.#currentSvg !== null) return;
    this.#preview.innerHTML = "";
    const placeholder = document.createElement("div");
    placeholder.className = "preview-empty";
    placeholder.textContent = "(no preview)";
    this.#preview.appendChild(placeholder);
  }

  #clearStatusError(): void {
    if (this.#status.classList.contains("error")) {
      this.#status.textContent = "";
      this.#status.classList.remove("error");
    }
  }

  #updateButtons(): void {
    const hasValidSvg = this.#currentSvg !== null;
    this.#buttons.saveSvg.disabled = !hasValidSvg;
    this.#buttons.savePng.disabled = !hasValidSvg;
    this.#buttons.wavedrom.disabled = this.#hasParseError || !hasValidSvg;
  }
}
