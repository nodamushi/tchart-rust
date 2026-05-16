/**
 * Trigger a browser download of the given blob under the given filename.
 * Creates an object URL, dispatches a programmatic click on a temporary
 * anchor, and revokes the URL afterwards so the blob can be garbage
 * collected.
 */
export function triggerDownload(blob: Blob, filename: string): void {
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  document.body.appendChild(anchor);
  anchor.click();
  document.body.removeChild(anchor);
  URL.revokeObjectURL(url);
}
