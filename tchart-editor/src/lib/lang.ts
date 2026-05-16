/**
 * UI language identifier used for the privacy notice and help modal.
 * Only Japanese and English are supported; everything else falls back to
 * English.
 */
export type UiLang = "ja" | "en";

/**
 * Detect the preferred UI language from `navigator.language`. Returns
 * `"ja"` when the browser locale starts with `"ja"` (e.g. `"ja"`,
 * `"ja-JP"`); otherwise `"en"`. Defensive against a missing or
 * non-string `navigator.language`.
 */
export function detectUiLang(): UiLang {
  const raw: unknown = navigator.language;
  const language = typeof raw === "string" ? raw.toLowerCase() : "";
  return language.startsWith("ja") ? "ja" : "en";
}
