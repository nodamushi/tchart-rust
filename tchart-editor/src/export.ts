import { embedTcmlSourceInPng } from "tchart-web";
import { triggerDownload } from "./lib/download";

/**
 * Trigger a browser download of the given SVG string as `tchart.svg`.
 */
export function downloadSvg(svgContent: string): void {
  const blob = new Blob([svgContent], { type: "image/svg+xml" });
  triggerDownload(blob, "tchart.svg");
}

/**
 * Rasterize the SVG to a 4x resolution PNG, embed the TCML source as an
 * iTXt chunk, and trigger a browser download as `tchart.png`.
 */
export function downloadPng(svgContent: string, tcmlSource: string): void {
  const img = new Image();
  const svgBlob = new Blob([svgContent], {
    type: "image/svg+xml;charset=utf-8",
  });
  const url = URL.createObjectURL(svgBlob);

  img.onload = () => {
    // why: 4x upscale keeps edges crisp on Retina / HiDPI displays.
    const scale = 4;
    const canvas = document.createElement("canvas");
    canvas.width = img.naturalWidth * scale;
    canvas.height = img.naturalHeight * scale;
    const context = canvas.getContext("2d");
    if (context === null) {
      URL.revokeObjectURL(url);
      throw new Error("Failed to acquire 2D rendering context");
    }
    context.drawImage(img, 0, 0, canvas.width, canvas.height);
    canvas.toBlob(async (blob) => {
      if (!blob) return;
      const raw = new Uint8Array(await blob.arrayBuffer());
      const withSource = embedTcmlSourceInPng(raw, tcmlSource);
      // why: `Uint8Array.buffer` is typed as `ArrayBufferLike`
      // (`SharedArrayBuffer | ArrayBuffer`) in some environments, while the
      // `Blob` constructor accepts only plain `ArrayBuffer`. Narrow with
      // `instanceof` so the type matches the runtime contract.
      const buffer = withSource.buffer;
      if (!(buffer instanceof ArrayBuffer)) {
        throw new Error("embedTcmlSourceInPng returned a non-ArrayBuffer Uint8Array");
      }
      const finalBlob = new Blob([buffer], {
        type: "image/png",
      });
      triggerDownload(finalBlob, "tchart.png");
    }, "image/png");
    URL.revokeObjectURL(url);
  };
  img.src = url;
}
