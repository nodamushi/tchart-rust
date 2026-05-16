import { toWaveJson } from "tchart-web";
import { extractErrorMessage } from "./lib/errors";
import { triggerDownload } from "./lib/download";

/**
 * Result of a WaveDrom conversion: the generated WaveJSON string and any
 * warnings emitted during conversion.
 */
export interface WaveDromResult {
  readonly json: string;
  readonly warnings: readonly string[];
}

// why: values crossing the wasm boundary arrive untyped, so validate the
// shape via a type predicate before narrowing to `WaveDromResult`. A bare
// `as` cast without validation would silently accept malformed input.
function isWaveDromResult(value: unknown): value is WaveDromResult {
  if (value === null || typeof value !== "object") return false;
  if (!("json" in value) || typeof value.json !== "string") return false;
  if (!("warnings" in value) || !Array.isArray(value.warnings)) return false;
  return value.warnings.every((warning) => typeof warning === "string");
}

/**
 * Convert TCML text into WaveDrom-compatible WaveJSON. The wasm side throws
 * on parse failure.
 */
export function convertToWaveJson(text: string): WaveDromResult {
  const result: unknown = toWaveJson(text);
  if (!isWaveDromResult(result)) {
    throw new Error("invalid wavejson result shape");
  }
  return result;
}

/**
 * Trigger a browser download of the given WaveJSON string as `tchart.json`.
 */
export function downloadWaveJson(json: string): void {
  const blob = new Blob([json], { type: "application/json" });
  triggerDownload(blob, "tchart.json");
}

/**
 * Build the status-line message based on how many warnings were emitted.
 */
export function formatStatus(warnings: readonly string[]): string {
  if (warnings.length === 0) return "WaveJSON exported.";
  if (warnings.length === 1) return `WaveJSON exported. ${warnings[0]}`;
  return `WaveJSON exported. ${warnings.length} warnings.`;
}

/**
 * Entry point invoked when the WaveDrom button is clicked. Runs convert →
 * download → status update in one shot, and on failure flips the status
 * line into an error state and returns `null`.
 */
export function handleWaveDromClick(text: string, status: HTMLElement): WaveDromResult | null {
  try {
    const result = convertToWaveJson(text);
    downloadWaveJson(result.json);
    status.textContent = formatStatus(result.warnings);
    status.classList.remove("error");
    return result;
  } catch (error: unknown) {
    const message = extractErrorMessage(error);
    status.textContent = `WaveDrom export failed: ${message}`;
    status.classList.add("error");
    return null;
  }
}
