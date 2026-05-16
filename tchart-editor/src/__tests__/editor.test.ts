import { describe, it, expect, beforeEach } from "vitest";
import { setupTabHandler } from "../editor";

describe("Tab key handling", () => {
  let textarea: HTMLTextAreaElement;

  beforeEach(() => {
    textarea = document.createElement("textarea");
    document.body.appendChild(textarea);
    setupTabHandler(textarea);
  });

  it("should insert a tab character at cursor position on Tab key", () => {
    textarea.value = "hello";
    textarea.selectionStart = 3;
    textarea.selectionEnd = 3;

    const event = new KeyboardEvent("keydown", { key: "Tab", bubbles: true });
    textarea.dispatchEvent(event);

    expect(textarea.value).toBe("hel\tlo");
  });

  it("should prevent default behavior on Tab key", () => {
    textarea.value = "test";
    textarea.selectionStart = 0;
    textarea.selectionEnd = 0;

    const event = new KeyboardEvent("keydown", {
      key: "Tab",
      bubbles: true,
      cancelable: true,
    });
    textarea.dispatchEvent(event);

    expect(event.defaultPrevented).toBe(true);
  });

  it("should advance cursor position by 1 after tab insertion", () => {
    textarea.value = "abcdef";
    textarea.selectionStart = 2;
    textarea.selectionEnd = 2;

    const event = new KeyboardEvent("keydown", { key: "Tab", bubbles: true });
    textarea.dispatchEvent(event);

    expect(textarea.selectionStart).toBe(3);
    expect(textarea.selectionEnd).toBe(3);
  });

  it("should replace selected text with tab character", () => {
    textarea.value = "hello world";
    textarea.selectionStart = 2;
    textarea.selectionEnd = 7;

    const event = new KeyboardEvent("keydown", { key: "Tab", bubbles: true });
    textarea.dispatchEvent(event);

    expect(textarea.value).toBe("he\torld");
    expect(textarea.selectionStart).toBe(3);
  });

  it("should not intercept non-Tab keys", () => {
    textarea.value = "test";
    textarea.selectionStart = 4;
    textarea.selectionEnd = 4;

    const event = new KeyboardEvent("keydown", {
      key: "Enter",
      bubbles: true,
      cancelable: true,
    });
    textarea.dispatchEvent(event);

    expect(event.defaultPrevented).toBe(false);
    expect(textarea.value).toBe("test");
  });
});
