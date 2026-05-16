import { describe, it, expect, afterEach } from "vitest";

import { requireElement } from "../lib/dom";

describe("requireElement", () => {
  afterEach(() => {
    document.body.replaceChildren();
  });

  it("returns the element typed as the given constructor on a happy path", () => {
    const textarea = document.createElement("textarea");
    textarea.id = "my-editor";
    document.body.appendChild(textarea);

    const result = requireElement("my-editor", HTMLTextAreaElement);
    expect(result).toBe(textarea);
  });

  it("throws when no element with the given id exists", () => {
    expect(() => requireElement("missing", HTMLTextAreaElement)).toThrowError(
      /id=missing not found/,
    );
  });

  it("throws when the element exists but is the wrong type", () => {
    const div = document.createElement("div");
    div.id = "not-a-textarea";
    document.body.appendChild(div);

    expect(() => requireElement("not-a-textarea", HTMLTextAreaElement)).toThrowError(
      /id=not-a-textarea is not HTMLTextAreaElement/,
    );
  });
});
