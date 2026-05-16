/**
 * Parse error position + message returned by `renderTcml` (see
 * docs/spec/web.md §renderTcml). 1-based line and column in character units
 * (not bytes); `length` is in character units too. A length of 0 marks an
 * insertion-point error (e.g. an open `"`) and is still drawn as one
 * character wide so it stays visible.
 */
export interface ParseErrorInfo {
  readonly line: number;
  readonly column: number;
  readonly length: number;
  readonly message: string;
}

/**
 * Default offsets used when computing the underline position. Kept here so
 * callers don't need to repeat them.
 *
 * `paddingLeftCh` / `paddingTopEm` express the host element's inner
 * padding in character / em-height units so the underline lines up with the
 * first column / first line of the highlighted text. Tweak via the
 * constructor when the editor uses non-default padding.
 */
export interface UnderlineMetrics {
  readonly paddingLeftCh: number;
  readonly paddingTopEm: number;
  readonly lineHeightEm: number;
}

const DEFAULT_METRICS: UnderlineMetrics = {
  paddingLeftCh: 0,
  paddingTopEm: 0,
  lineHeightEm: 1.5,
};

/**
 * Manages a single `<div class="tcml-error-underline">` placed absolutely
 * inside a host element. Drawing relies on the host being a monospace
 * container — x positions are computed in `ch` units (width of "0") and y
 * positions in line-height multiples.
 *
 * `show` replaces any previous underline; `clear` removes it. The host must
 * have `position: relative` (or otherwise establish a containing block) for
 * the absolutely-positioned underline to land on top of its text.
 */
export class ErrorUnderlineOverlay {
  readonly #host: HTMLElement;
  readonly #metrics: UnderlineMetrics;
  #node: HTMLDivElement | null = null;

  constructor(host: HTMLElement, metrics: Partial<UnderlineMetrics> = {}) {
    this.#host = host;
    this.#metrics = {
      paddingLeftCh: metrics.paddingLeftCh ?? DEFAULT_METRICS.paddingLeftCh,
      paddingTopEm: metrics.paddingTopEm ?? DEFAULT_METRICS.paddingTopEm,
      lineHeightEm: metrics.lineHeightEm ?? DEFAULT_METRICS.lineHeightEm,
    };
  }

  /**
   * Place the underline at the given position, creating the element on
   * first use and updating an existing one on subsequent calls so only a
   * single underline is ever attached to the host.
   */
  show(position: ParseErrorInfo): void {
    const node = this.#ensureNode();
    // The host's padding box (not the border box) is the origin of absolutely
    // positioned children, so absolute `top: 0; left: 0` sits at the padding
    // edge — i.e. before the host's CSS padding pushes the text down/right.
    // Read the live computed padding so the underline tracks any change to
    // code-input's --padding vars without re-instantiating the overlay.
    const hostStyle = getComputedStyle(this.#host);
    const paddingLeftPx = parseFloat(hostStyle.paddingLeft) || 0;
    const paddingTopPx = parseFloat(hostStyle.paddingTop) || 0;
    // why: `column - 1` because column is 1-based; `line - 1` is the
    // zero-based line offset multiplied by line-height.
    const leftCh = this.#metrics.paddingLeftCh + Math.max(0, position.column - 1);
    const topEm =
      this.#metrics.paddingTopEm + Math.max(0, position.line - 1) * this.#metrics.lineHeightEm;
    const widthCh = Math.max(position.length, 1);

    node.style.left = `calc(${paddingLeftPx}px + ${leftCh}ch)`;
    node.style.top = `calc(${paddingTopPx}px + ${topEm}em)`;
    // why: the wavy line is drawn by `text-decoration` on the inner span;
    // its width comes from the NBSP run, so the outer `<div>` width matches
    // by being sized to the same character count for layout stability.
    node.style.width = `${widthCh}ch`;
    node.title = position.message;
    this.#setUnderlineWidth(node, widthCh);
  }

  /**
   * Remove the underline element if one is currently attached. Safe to call
   * when nothing is shown.
   */
  clear(): void {
    if (this.#node === null) return;
    this.#node.remove();
    this.#node = null;
  }

  #ensureNode(): HTMLDivElement {
    if (this.#node !== null) return this.#node;
    const node = document.createElement("div");
    node.className = "tcml-error-underline";
    // Absolute positioning + pointer-events:none so the underline never
    // intercepts caret clicks on the underlying textarea / code-input.
    node.style.position = "absolute";
    node.style.pointerEvents = "none";
    node.style.height = `${this.#metrics.lineHeightEm}em`;
    node.style.boxSizing = "border-box";
    // why: the wavy underline is drawn by `text-decoration: underline wavy`
    // on an inner span filled with NBSPs. `border-style: wavy` is not a
    // valid CSS value — only `text-decoration-style` accepts `wavy` — so
    // the visible wave must come from a text-decoration on actual text.
    // The text itself is invisible (`color: transparent`); only its
    // underline is drawn. The fallback colour literal here must stay in
    // sync with `--error-color` in `style.css`.
    const inner = document.createElement("span");
    inner.style.color = "transparent";
    inner.style.textDecorationLine = "underline";
    inner.style.textDecorationStyle = "wavy";
    inner.style.textDecorationColor = "var(--error-color, #d32f2f)";
    node.appendChild(inner);
    this.#host.appendChild(node);
    this.#node = node;
    return node;
  }

  /**
   * Fill the inner span with `count` NBSP characters so the wavy underline
   * spans exactly that many monospace cells. NBSP is used instead of regular
   * spaces so the inline text is not collapsed.
   */
  #setUnderlineWidth(node: HTMLDivElement, count: number): void {
    const inner = node.firstElementChild;
    if (!(inner instanceof HTMLElement)) return;
    inner.textContent = " ".repeat(count);
  }
}
