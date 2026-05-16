/**
 * Attach a handler so that pressing Tab inside the textarea inserts a tab
 * character instead of moving focus to the next focusable element.
 */
export function setupTabHandler(textarea: HTMLTextAreaElement): void {
  textarea.addEventListener("keydown", (event) => {
    if (event.key === "Tab") {
      event.preventDefault();
      const start = textarea.selectionStart;
      const end = textarea.selectionEnd;
      textarea.value = textarea.value.substring(0, start) + "\t" + textarea.value.substring(end);
      textarea.selectionStart = textarea.selectionEnd = start + 1;
    }
  });
}
