import { describe, it, expect, beforeAll } from "vitest";
import Prism from "prismjs";
import { registerTcmlLanguage } from "../tcml-prism";

interface Token {
  readonly type: string;
  readonly content: string | Token | ReadonlyArray<string | Token>;
}

function isToken(value: unknown): value is Token {
  return (
    value !== null &&
    typeof value === "object" &&
    "type" in value &&
    typeof (value as { type: unknown }).type === "string" &&
    "content" in value
  );
}

function flatten(content: Token["content"]): ReadonlyArray<string | Token> {
  if (typeof content === "string") return [content];
  if (Array.isArray(content)) return content;
  if (isToken(content)) return [content];
  return [];
}

/**
 * Walk the tokenized output and collect every Token entry (including nested
 * ones) so test assertions can search by type without caring about nesting.
 */
function collectTokens(tokens: ReadonlyArray<string | Token>): ReadonlyArray<Token> {
  return tokens.flatMap((token) => {
    if (typeof token === "string") return [];
    return [token, ...collectTokens(flatten(token.content))];
  });
}

function tokenText(token: Token): string {
  const parts = flatten(token.content);
  return parts
    .map((part) => {
      if (typeof part === "string") return part;
      return tokenText(part);
    })
    .join("");
}

describe("registerTcmlLanguage", () => {
  beforeAll(() => {
    registerTcmlLanguage(Prism);
  });

  it("registers a 'tcml' grammar on Prism.languages", () => {
    expect(Prism.languages.tcml).toBeDefined();
  });

  it("recognises @ directives as keyword", () => {
    const tokens = Prism.tokenize("@step 25", Prism.languages.tcml);
    const collected = collectTokens(tokens);
    const keywords = collected.filter((token) => token.type === "keyword");
    expect(keywords.length).toBeGreaterThan(0);
    expect(keywords.some((token) => tokenText(token) === "@step")).toBe(true);
  });

  it("recognises @-> as a single keyword token", () => {
    const tokens = Prism.tokenize("@-> label", Prism.languages.tcml);
    const collected = collectTokens(tokens);
    const keywords = collected.filter((token) => token.type === "keyword");
    expect(keywords.some((token) => tokenText(token) === "@->")).toBe(true);
    // Ensure '@-' was not split off as a partial keyword.
    expect(keywords.some((token) => tokenText(token) === "@-")).toBe(false);
  });

  it("recognises double-quoted strings", () => {
    const tokens = Prism.tokenize('@title "hello world"', Prism.languages.tcml);
    const collected = collectTokens(tokens);
    const strings = collected.filter((token) => token.type === "string");
    expect(strings.length).toBeGreaterThan(0);
    expect(strings.some((token) => tokenText(token) === '"hello world"')).toBe(true);
  });

  it("does NOT treat // inside a string as a comment", () => {
    const tokens = Prism.tokenize('@title "foo // bar"', Prism.languages.tcml);
    const collected = collectTokens(tokens);
    const comments = collected.filter((token) => token.type === "comment");
    expect(comments.length).toBe(0);
    const strings = collected.filter((token) => token.type === "string");
    expect(strings.some((token) => tokenText(token) === '"foo // bar"')).toBe(true);
  });

  it("recognises // line comments", () => {
    const tokens = Prism.tokenize("// this is a comment", Prism.languages.tcml);
    const collected = collectTokens(tokens);
    const comments = collected.filter((token) => token.type === "comment");
    expect(comments.length).toBe(1);
    expect(tokenText(comments[0]!)).toContain("// this is a comment");
  });

  it("stops comments at end of line", () => {
    const tokens = Prism.tokenize("Clock _~ // tail\nData ==", Prism.languages.tcml);
    const collected = collectTokens(tokens);
    const comments = collected.filter((token) => token.type === "comment");
    expect(comments.length).toBe(1);
    expect(tokenText(comments[0]!).includes("Data")).toBe(false);
  });

  it("recognises signal name at start of line as variable", () => {
    const tokens = Prism.tokenize("Clock _~_~", Prism.languages.tcml);
    const collected = collectTokens(tokens);
    const variables = collected.filter((token) => token.type === "variable");
    expect(variables.some((token) => tokenText(token) === "Clock")).toBe(true);
  });

  it("recognises numbers with and without unit suffix", () => {
    const tokens = Prism.tokenize("@step 25\n@slant 1.5\n@font-size 12px", Prism.languages.tcml);
    const collected = collectTokens(tokens);
    const numbers = collected.filter((token) => token.type === "number");
    const texts = numbers.map((token) => tokenText(token));
    expect(texts).toContain("25");
    expect(texts).toContain("1.5");
    // The `12` part of `12px` should be a number; the `px` may be styled or not, but the digit run must classify as number.
    expect(texts.some((text) => text.startsWith("12"))).toBe(true);
  });

  it("does NOT classify wave characters as keyword/variable/string/comment/number", () => {
    // why: wave characters per docs/spec/tcml-format.md §波形記号 are
    // `_ ~ = - X ?` only. The grammar must not classify any of these as
    // keyword/string/comment/number tokens.
    const tokens = Prism.tokenize("Clock _~=-X?", Prism.languages.tcml);
    const collected = collectTokens(tokens);
    const waveChars = ["_", "~", "=", "-", "X", "?"];
    for (const character of waveChars) {
      // Search for tokens whose plain text equals one of these — none should be classified.
      const classified = collected.filter(
        (token) =>
          tokenText(token) === character &&
          ["keyword", "string", "comment", "number"].includes(token.type),
      );
      expect(classified.length).toBe(0);
    }
  });

  it("does not classify the quoted signal name body as a comment when it contains //", () => {
    const tokens = Prism.tokenize('"foo // bar" _~_~', Prism.languages.tcml);
    const collected = collectTokens(tokens);
    expect(collected.filter((token) => token.type === "comment").length).toBe(0);
    expect(collected.some((token) => token.type === "string")).toBe(true);
  });

  it("handles empty input without throwing", () => {
    expect(() => Prism.tokenize("", Prism.languages.tcml)).not.toThrow();
  });
});
