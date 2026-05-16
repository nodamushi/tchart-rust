/// <reference types="vitest" />
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { defineConfig, type Plugin } from "vite";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";
import { viteSingleFile } from "vite-plugin-singlefile";

const pkg = JSON.parse(
  readFileSync(fileURLToPath(new URL("./package.json", import.meta.url)), "utf-8"),
);

function stripModuleScript(): Plugin {
  return {
    name: "tchart-strip-module",
    apply: "build",
    enforce: "post",
    transformIndexHtml(html) {
      return html.replace(/<script\b([^>]*)\btype="module"([^>]*)>/g, "<script$1$2>");
    },
  };
}

export default defineConfig({
  base: "./",
  define: {
    __APP_VERSION__: JSON.stringify(pkg.version),
  },
  plugins: [wasm(), topLevelAwait(), viteSingleFile(), stripModuleScript()],
  build: {
    assetsInlineLimit: 100 * 1024 * 1024,
    cssCodeSplit: false,
    target: "esnext",
    rollupOptions: {
      output: {
        format: "iife",
        inlineDynamicImports: true,
      },
    },
  },
  server: {
    fs: {
      allow: [".", "../help/output", "../tchart-web/pkg"],
    },
  },
  test: {
    environment: "happy-dom",
    globals: true,
    server: {
      deps: {
        inline: [/help\/output\//],
      },
    },
  },
  optimizeDeps: {
    exclude: ["tchart-web"],
  },
});
