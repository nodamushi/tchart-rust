#!/usr/bin/env node
// Build the tchart-web wasm package for the editor.
//
// Output: tchart-web/pkg/
//   - tchart_web.js / .d.ts / _bg.wasm / _bg.wasm.d.ts  (from wasm-bindgen)
//   - package.json  (written here so pnpm `file:` dep can resolve before
//     wasm-bindgen has ever run; wasm-bindgen itself does not emit it)
//
// The package.json version is read from the workspace Cargo.toml so that
// editor and wasm crate versions stay in lockstep automatically.
//
// Usage:
//   node scripts/build-wasm-pkg.mjs

import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const editorRoot = resolve(here, "..");
const repoRoot = resolve(editorRoot, "..");
const pkgDir = join(repoRoot, "tchart-web", "pkg");
const cargoTomlPath = join(repoRoot, "Cargo.toml");
const wasmArtifact = join(
  repoRoot,
  "target",
  "wasm32-unknown-unknown",
  "release",
  "tchart_web.wasm",
);

function readWorkspaceVersion() {
  const toml = readFileSync(cargoTomlPath, "utf-8");
  const match = toml.match(/\[workspace\.package\][^[]*?version\s*=\s*"([^"]+)"/);
  if (!match) {
    throw new Error(`Could not find workspace.package.version in ${cargoTomlPath}`);
  }
  return match[1];
}

function writeManifest(version) {
  mkdirSync(pkgDir, { recursive: true });
  const manifest = {
    name: "tchart-web",
    type: "module",
    version,
    license: "0BSD",
    files: ["tchart_web_bg.wasm", "tchart_web.js", "tchart_web.d.ts"],
    main: "tchart_web.js",
    types: "tchart_web.d.ts",
    sideEffects: ["./snippets/*"],
  };
  writeFileSync(join(pkgDir, "package.json"), `${JSON.stringify(manifest, null, 2)}\n`);
}

function run(cmd, args, cwd) {
  const result = spawnSync(cmd, args, { cwd, stdio: "inherit" });
  if (result.status !== 0) {
    throw new Error(`${cmd} ${args.join(" ")} failed with status ${result.status}`);
  }
}

function buildWasm() {
  run(
    "cargo",
    ["build", "--manifest-path", cargoTomlPath, "--target", "wasm32-unknown-unknown", "--release", "-p", "tchart-web"],
    repoRoot,
  );
  run("wasm-bindgen", ["--target", "web", "--out-dir", pkgDir, wasmArtifact], repoRoot);
}

function main() {
  const version = readWorkspaceVersion();
  writeManifest(version);
  buildWasm();
}

main();
