/**
 * Normalize a value caught from a `try` / `catch` (which is typed `unknown`)
 * into a human-readable message string. Returns `error.message` when the
 * value is an `Error` instance, otherwise the result of `String(error)`.
 */
export function extractErrorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  return String(error);
}
