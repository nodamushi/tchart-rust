import { describe, it, expect } from "vitest";

import { extractErrorMessage } from "../lib/errors";

describe("extractErrorMessage", () => {
  it("returns the `message` field when given an Error instance", () => {
    const error = new Error("boom");
    expect(extractErrorMessage(error)).toBe("boom");
  });

  it("preserves subclass messages", () => {
    class ParseError extends Error {}
    expect(extractErrorMessage(new ParseError("bad token"))).toBe("bad token");
  });

  it("stringifies non-Error values", () => {
    expect(extractErrorMessage("plain string")).toBe("plain string");
    expect(extractErrorMessage(42)).toBe("42");
    expect(extractErrorMessage(null)).toBe("null");
    expect(extractErrorMessage(undefined)).toBe("undefined");
  });

  it("stringifies plain objects without throwing", () => {
    expect(extractErrorMessage({ code: 1 })).toBe("[object Object]");
  });
});
