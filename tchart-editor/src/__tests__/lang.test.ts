import { describe, it, expect, afterEach, vi } from "vitest";

import { detectUiLang } from "../lib/lang";

describe("detectUiLang", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("returns 'ja' when navigator.language starts with 'ja'", () => {
    vi.spyOn(navigator, "language", "get").mockReturnValue("ja-JP");
    expect(detectUiLang()).toBe("ja");
  });

  it("returns 'ja' for plain 'ja' as well", () => {
    vi.spyOn(navigator, "language", "get").mockReturnValue("ja");
    expect(detectUiLang()).toBe("ja");
  });

  it("returns 'en' for non-Japanese locales", () => {
    vi.spyOn(navigator, "language", "get").mockReturnValue("en-US");
    expect(detectUiLang()).toBe("en");
  });

  it("matches case-insensitively", () => {
    vi.spyOn(navigator, "language", "get").mockReturnValue("JA-JP");
    expect(detectUiLang()).toBe("ja");
  });

  it("returns 'en' when navigator.language is an empty string", () => {
    // Empty string is the closest typed proxy for an effectively
    // missing locale. The implementation also tolerates a non-string
    // navigator.language via its `typeof raw === "string"` guard, but
    // that branch cannot be exercised here without a silencing cast.
    vi.spyOn(navigator, "language", "get").mockReturnValue("");
    expect(detectUiLang()).toBe("en");
  });
});
