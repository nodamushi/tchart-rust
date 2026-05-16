/**
 * Type of the Prism object accepted by the registration helper. Only the
 * fields we actually touch are declared so unit tests can pass a small fake
 * Prism instance if needed.
 *
 * `languages` is the writable grammar registry: the helper mutates this
 * record in place to install the `tcml` grammar.
 */
export interface PrismLike {
  readonly languages: Record<string, unknown>;
}

/**
 * Register the TCML grammar on the given Prism instance. Idempotent: calling
 * twice replaces the existing grammar with a fresh definition.
 *
 * The grammar covers five token kinds described in docs/spec/editor.md
 * §シンタックスハイライト: comments, strings, `@`-directives, numbers, and
 * signal-name variables at the start of timing lines. Wave characters
 * (`_~=-X?:|[]`) intentionally fall through to the default (unstyled) token
 * class.
 */
export function registerTcmlLanguage(prism: PrismLike): void {
  // why: `//` line comments must lose the race against `"..."` strings so
  // that `"foo // bar"` stays a single string. Prism tokenises top-down and
  // returns the first match wins per offset, so ordering matters.
  const grammar = {
    comment: {
      pattern: /\/\/.*/,
      greedy: true,
    },
    string: {
      // why: limited to a single line — TCML strings don't span lines (a
      // newline before the closing quote is a parse error). `[^"\\]` plus
      // `\\.` handles backslash escapes without a hard list of escape forms.
      pattern: /"(?:[^"\\\n]|\\.)*"/,
      greedy: true,
    },
    // why: `@->` and `@signal` and `@step` etc. — any `@` followed by either
    // a multi-char arrow (`->`) or a run of word chars / `-` (for kebab-case
    // directives like `@font-size`). Both `@->` and `@font-size` are common,
    // so accept both shapes in one alternation; the arrow form is listed
    // first so the regex engine commits to it before considering the
    // identifier branch.
    keyword: {
      pattern: /@(?:->|[A-Za-z][A-Za-z0-9-]*)/,
      greedy: true,
    },
    number: {
      // why: integer or decimal; an optional unit suffix (`px`, `lh`) is NOT
      // included in the number token so styling lands on the digits only.
      // We use `\b` only on the leading edge: a trailing `\b` would reject
      // `12px` because `2`/`p` is not a word boundary.
      pattern: /\b\d+(?:\.\d+)?/,
      greedy: true,
    },
    variable: {
      // why: a signal name only appears at the start of a line. We use a
      // lookbehind-style first capture group (`^[ \t]*`) so Prism discards
      // the leading whitespace from the produced token and only the name
      // itself is highlighted. The branch alternation covers both ASCII
      // identifier-like names and multibyte names (any non-ASCII signal
      // identifier the user types).
      pattern: /(^[ \t]*)(?:[A-Za-z_][\w-]*|[^\s\d_~=Xx?:|[\]"/@-][^\s_~=Xx?:|[\]"/@-]*)/m,
      lookbehind: true,
      greedy: true,
    },
  };
  // Mutate `prism.languages` in-place: this is the standard Prism extension
  // shape and the lib expects identity equality, not a returned grammar.
  // The mutation lives entirely inside this helper.
  prism.languages.tcml = grammar;
}
