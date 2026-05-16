import { registerTemplate } from "@webcoder49/code-input";
import PrismTemplate from "@webcoder49/code-input/templates/prism.mjs";
import Prism from "prismjs";
import { registerTcmlLanguage } from "./tcml-prism";

// why: side-effect import — the package self-registers the custom element
// on `customElements.define("code-input", ...)` when its module is loaded.
//
// The Prism `code-input` template needs to be registered exactly once at
// module load, after the TCML grammar is added to `Prism.languages`. The
// custom-element registration and the template registration are both
// idempotent at the call sites that matter (vite HMR / test reload), so we
// guard with a simple flag.

let registered = false;

/**
 * Register the `tcml` Prism grammar and the matching `code-input` template
 * named "tcml". Safe to call more than once. Must be called before the
 * first `<code-input template="tcml">` element is upgraded by the browser;
 * the recommended invocation site is during app bootstrap (`main.ts`).
 */
export function setupCodeInputTemplate(): void {
  if (registered) return;
  registerTcmlLanguage(Prism);
  // why: an empty plugin list keeps the template lightweight. Indent / Tab
  // handling stays with `setupTabHandler` against the inner textarea so the
  // existing unit test coverage applies unchanged.
  registerTemplate("tcml", new PrismTemplate(Prism, []));
  registered = true;
}

/**
 * Minimal structural type for the `<code-input>` element fields we rely on.
 * Mirrors the public ambient declaration shipped by the package, but pinned
 * down to just the surface PreviewController and the export helpers touch.
 *
 * `value` is intentionally writable: `main.ts` assigns the loaded TCML text
 * back through `editor.value = ...`. The inner `textareaElement`, `preElement`
 * and `codeElement` references are read-only — the element itself owns their
 * lifecycle.
 */
export interface CodeInputElement extends HTMLElement {
  value: string;
  readonly textareaElement?: HTMLTextAreaElement;
  readonly preElement?: HTMLPreElement;
  readonly codeElement?: HTMLElement;
}

/**
 * Look up a `<code-input>` element by id, throwing on missing / wrong
 * element. Cannot delegate to {@link requireElement} in `lib/dom.ts`: that
 * helper checks `instanceof HTMLXxxElement`, while at runtime `<code-input>`
 * actually extends plain `HTMLElement` (the package's `.d.ts` claims
 * `HTMLTextAreaElement` but the prototype chain disagrees). This helper
 * therefore branches on tag name and then attaches the structural
 * `CodeInputElement` type so callers can read `value` without suppressing
 * the type system.
 */
export function requireCodeInputElement(id: string): CodeInputElement {
  const element = document.getElementById(id);
  if (element === null) {
    throw new Error(`requireCodeInputElement: id=${id} not found`);
  }
  if (element.tagName.toLowerCase() !== "code-input") {
    throw new Error(`requireCodeInputElement: id=${id} is not <code-input>`);
  }
  // The runtime tag-name check above is the only guarantee available; the
  // cast attaches the structural interface that mirrors the library's
  // contract (validate-then-brand pattern).
  return element as CodeInputElement;
}
