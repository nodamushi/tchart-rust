#!/usr/bin/env node
// Build a static list of bundled runtime library licenses for the editor.
//
// Output: tchart-editor/src/generated/licenses.ts
//   - TypeScript package.json `dependencies` (excluding the local
//     `tchart-web` workspace package, which is part of the project itself).
//     Reads name / version / license from node_modules/<pkg>/package.json
//     and concatenates every LICENSE* file shipped in the package.
//   - Rust runtime crates pulled into the wasm32-unknown-unknown build of
//     `tchart-web`, restricted to normal (non dev / non build kind=null)
//     deps via `cargo metadata --filter-platform`. Reads LICENSE files
//     from each crate's manifest directory.
//
// Own crates (`tchart-core`, `tchart-cli`, `tchart-web`, `tchart-editor`)
// are deliberately excluded: the project ships under 0BSD which carries
// no end-user attribution requirement.

import { execFileSync } from "node:child_process";
import { readFileSync, readdirSync, writeFileSync, existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const editorRoot = resolve(here, "..");
const repoRoot = resolve(editorRoot, "..");
const outputPath = join(editorRoot, "src", "generated", "licenses.ts");

const OWN_PACKAGES = new Set(["tchart-core", "tchart-cli", "tchart-web", "tchart-editor"]);

// why: external inputs (package.json / cargo metadata output) arrive as
// unknown shapes. Parse to `unknown` and route through `assertObject` /
// `getString` / `getArray` helpers below so any missing or unexpectedly
// typed field throws with a clear path, instead of silently producing a
// wrong licenses.ts via `undefined.foo` propagation.
function parseJsonFile(path) {
  /** @type {unknown} */
  const value = JSON.parse(readFileSync(path, "utf-8"));
  return value;
}

function isPlainObject(value) {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function assertObject(value, context) {
  if (!isPlainObject(value)) {
    throw new Error(`${context}: expected object, got ${typeof value}`);
  }
  return value;
}

function getString(record, key, context) {
  const value = record[key];
  if (typeof value !== "string") {
    throw new Error(`${context}.${key}: expected string, got ${typeof value}`);
  }
  return value;
}

function getOptionalString(record, key) {
  const value = record[key];
  return typeof value === "string" ? value : null;
}

function getArray(record, key, context) {
  const value = record[key];
  if (!Array.isArray(value)) {
    throw new Error(`${context}.${key}: expected array, got ${typeof value}`);
  }
  return value;
}

function listLicenseFiles(directory) {
  let entries;
  try {
    entries = readdirSync(directory, { withFileTypes: true });
  } catch {
    return [];
  }
  return entries
    .filter((entry) => entry.isFile())
    .map((entry) => entry.name)
    .filter((name) => /^LICEN[CS]E/i.test(name) || /^UNLICENSE/i.test(name))
    .sort();
}

function readLicenseText(directory) {
  const files = listLicenseFiles(directory);
  if (files.length === 0) return null;
  const parts = [];
  for (const name of files) {
    const fullPath = join(directory, name);
    let text;
    try {
      text = readFileSync(fullPath, "utf-8");
    } catch {
      continue;
    }
    parts.push(`----- ${name} -----\n${text.trim()}`);
  }
  if (parts.length === 0) return null;
  return parts.join("\n\n");
}

// why: MIT / BSD / etc. carry a per-library copyright line that varies by
// crate, while the bulk of the text (permission notice) is identical across
// every user of the same license. Group entries by the body with copyright
// lines stripped out, so the same MIT permission notice is shown once, with
// each library contributing its own copyright line to the group.
//
// Heuristic: a real copyright notice begins the line with `Copyright`
// (capital C), `©`, or `(C)` / `(c)` — no leading non-whitespace prose
// before it — and is therefore the first token on its own line. Body
// phrases like `the copyright owner that is granting the License.` are
// preceded by indentation + lowercase prose, and headings like
// `COPYRIGHT AND PERMISSION NOTICE` are all-caps; both are excluded by
// the case-sensitive `Copyright` literal at the line start.
function isCopyrightLine(line) {
  return /^\s*(?:©|\([cC]\)|Copyright)\b/.test(line);
}

function extractCopyrightLines(text) {
  const lines = text.split(/\r?\n/);
  const matches = [];
  for (const line of lines) {
    if (isCopyrightLine(line)) {
      matches.push(line.trim());
    }
  }
  return matches;
}

function stripCopyrightLines(text) {
  return text
    .split(/\r?\n/)
    .filter((line) => !isCopyrightLine(line))
    .join("\n");
}

// why: collapse every run of whitespace (newlines included) to a single
// space so trivial diffs — trailing spaces, varying blank-line run length,
// indentation drift — never splinter the same MIT / BSD permission notice
// into multiple groups. The displayed body keeps its original formatting;
// this normalized form is only used as a dedup key.
function normalizeForKey(text) {
  return text.replace(/\s+/g, " ").trim();
}

function resolveNpmLicense(pkgJson) {
  const direct = pkgJson["license"];
  if (typeof direct === "string") return direct;
  if (isPlainObject(direct)) {
    const type = direct["type"];
    if (typeof type === "string") return type;
  }
  const list = pkgJson["licenses"];
  if (Array.isArray(list) && list.length > 0) {
    const first = list[0];
    if (isPlainObject(first)) {
      const type = first["type"];
      if (typeof type === "string") return type;
    }
  }
  return "UNKNOWN";
}

function collectNpmDependencies() {
  const editorPkgRaw = parseJsonFile(join(editorRoot, "package.json"));
  const editorPkg = assertObject(editorPkgRaw, "editor package.json");
  const dependenciesRaw = editorPkg["dependencies"];
  const dependencies = isPlainObject(dependenciesRaw) ? dependenciesRaw : {};
  const entries = [];
  for (const name of Object.keys(dependencies).sort()) {
    if (OWN_PACKAGES.has(name)) continue;
    const pkgDirectory = join(editorRoot, "node_modules", name);
    const pkgJsonPath = join(pkgDirectory, "package.json");
    if (!existsSync(pkgJsonPath)) {
      throw new Error(`npm package not installed: ${name}`);
    }
    const pkgJson = assertObject(parseJsonFile(pkgJsonPath), `npm package ${name}`);
    const version = getOptionalString(pkgJson, "version") ?? "unknown";
    const spdx = resolveNpmLicense(pkgJson);
    const text = readLicenseText(pkgDirectory);
    if (text === null) {
      throw new Error(`no LICENSE file for npm package: ${name}`);
    }
    entries.push({
      name,
      version,
      spdx,
      origin: "npm",
      text,
    });
  }
  return entries;
}

function runCargoMetadata() {
  const out = execFileSync(
    "cargo",
    [
      "metadata",
      "--format-version=1",
      "--filter-platform",
      "wasm32-unknown-unknown",
    ],
    { cwd: repoRoot, maxBuffer: 64 * 1024 * 1024 },
  );
  /** @type {unknown} */
  const parsed = JSON.parse(out.toString("utf-8"));
  return parsed;
}

function collectRustDependencies() {
  const metadata = assertObject(runCargoMetadata(), "cargo metadata");
  const rawPackages = getArray(metadata, "packages", "cargo metadata");
  const resolve = assertObject(metadata["resolve"], "cargo metadata.resolve");
  const rawNodes = getArray(resolve, "nodes", "cargo metadata.resolve");

  const packages = new Map();
  for (const entry of rawPackages) {
    const pkg = assertObject(entry, "cargo metadata.packages[]");
    packages.set(getString(pkg, "id", "package"), pkg);
  }
  const nodes = new Map();
  for (const entry of rawNodes) {
    const node = assertObject(entry, "cargo metadata.resolve.nodes[]");
    nodes.set(getString(node, "id", "node"), node);
  }

  const rootEntry = rawPackages.find(
    (entry) => isPlainObject(entry) && entry["name"] === "tchart-web",
  );
  if (rootEntry === undefined) {
    throw new Error("tchart-web package not found in cargo metadata");
  }
  const root = assertObject(rootEntry, "tchart-web package");
  const rootId = getString(root, "id", "tchart-web");

  const visited = new Set();
  const stack = [rootId];
  while (stack.length > 0) {
    const id = stack.pop();
    if (id === undefined || visited.has(id)) continue;
    visited.add(id);
    const node = nodes.get(id);
    if (node === undefined) continue;
    const deps = getArray(node, "deps", `node ${id}`);
    for (const depRaw of deps) {
      const dep = assertObject(depRaw, `node ${id}.deps[]`);
      const depKindsRaw = getArray(dep, "dep_kinds", `node ${id}.deps[].dep_kinds`);
      const kinds = depKindsRaw.map((kindRaw) => {
        const kindRecord = assertObject(kindRaw, "dep_kind");
        return kindRecord["kind"];
      });
      // dep_kinds: [{kind: null, ...}] = normal; "dev" / "build" excluded.
      if (kinds.includes(null) || kinds.includes("normal") || kinds.length === 0) {
        stack.push(getString(dep, "pkg", "dep"));
      }
    }
  }

  const entries = [];
  for (const id of visited) {
    const pkg = packages.get(id);
    if (pkg === undefined) continue;
    const name = getString(pkg, "name", "package");
    if (OWN_PACKAGES.has(name)) continue;
    const version = getString(pkg, "version", `package ${name}`);
    const manifestPath = getString(pkg, "manifest_path", `package ${name}`);
    const directory = dirname(manifestPath);
    const text = readLicenseText(directory);
    if (text === null) {
      throw new Error(`no LICENSE file for Rust crate: ${name} ${version}`);
    }
    entries.push({
      name,
      version,
      spdx: getOptionalString(pkg, "license") ?? "UNKNOWN",
      origin: "rust",
      text,
    });
  }
  entries.sort((a, b) => a.name.localeCompare(b.name));
  return entries;
}

function escapeForTemplate(value) {
  return value.replace(/\\/g, "\\\\").replace(/`/g, "\\`").replace(/\$\{/g, "\\${");
}

function groupByLicenseBody(entries) {
  /** @type {Map<string, { spdxLabels: Set<string>; body: string; libraries: Array<{ name: string; version: string; origin: string; copyright: string }> }>} */
  const groups = new Map();
  for (const entry of entries) {
    const copyrights = extractCopyrightLines(entry.text);
    const bodyOnly = stripCopyrightLines(entry.text).trim();
    const key = normalizeForKey(bodyOnly);
    let group = groups.get(key);
    if (group === undefined) {
      group = {
        spdxLabels: new Set(),
        body: bodyOnly,
        libraries: [],
      };
      groups.set(key, group);
    }
    group.spdxLabels.add(entry.spdx);
    group.libraries.push({
      name: entry.name,
      version: entry.version,
      origin: entry.origin,
      // why: MIT / BSD attribution is satisfied by the per-library Copyright
      // line. Most LICENSE files carry exactly one such line; if a crate has
      // more, join them so none are dropped.
      copyright: copyrights.join(" / ") || "",
    });
  }
  const out = [];
  for (const group of groups.values()) {
    group.libraries.sort((a, b) => {
      if (a.origin !== b.origin) return a.origin === "npm" ? -1 : 1;
      return a.name.localeCompare(b.name);
    });
    out.push({
      spdx: [...group.spdxLabels].sort().join(" / "),
      body: group.body,
      libraries: group.libraries,
    });
  }
  out.sort((a, b) => a.spdx.localeCompare(b.spdx));
  return out;
}

function emit(groups) {
  const lines = [];
  lines.push("// This file is auto-generated by scripts/build-licenses.mjs.");
  lines.push("// Do not edit by hand; rerun the script to refresh.");
  lines.push("");
  lines.push("/**");
  lines.push(" * One bundled third-party library entry. `copyright` is the per-library");
  lines.push(' * "Copyright (c) ... Holder" line lifted out of the upstream LICENSE so');
  lines.push(" * the license body itself can be deduplicated across libraries that share");
  lines.push(" * the same permission notice (typical for MIT / BSD).");
  lines.push(" */");
  lines.push("export interface LicenseLibrary {");
  lines.push("  readonly name: string;");
  lines.push("  readonly version: string;");
  lines.push('  readonly origin: "npm" | "rust";');
  lines.push("  readonly copyright: string;");
  lines.push("}");
  lines.push("");
  lines.push("/**");
  lines.push(" * One license-body group. `spdx` is a slash-joined list of SPDX labels");
  lines.push(" * observed across the grouped libraries (usually a single label).");
  lines.push(" */");
  lines.push("export interface LicenseGroup {");
  lines.push("  readonly spdx: string;");
  lines.push("  readonly body: string;");
  lines.push("  readonly libraries: ReadonlyArray<LicenseLibrary>;");
  lines.push("}");
  lines.push("");
  lines.push("export const LICENSE_GROUPS: ReadonlyArray<LicenseGroup> = [");
  for (const group of groups) {
    lines.push("  {");
    lines.push(`    spdx: ${JSON.stringify(group.spdx)},`);
    lines.push(`    body: \`${escapeForTemplate(group.body)}\`,`);
    lines.push("    libraries: [");
    for (const library of group.libraries) {
      lines.push("      {");
      lines.push(`        name: ${JSON.stringify(library.name)},`);
      lines.push(`        version: ${JSON.stringify(library.version)},`);
      lines.push(`        origin: ${JSON.stringify(library.origin)},`);
      lines.push(`        copyright: ${JSON.stringify(library.copyright)},`);
      lines.push("      },");
    }
    lines.push("    ],");
    lines.push("  },");
  }
  lines.push("];");
  lines.push("");
  return lines.join("\n");
}

const npmEntries = collectNpmDependencies();
const rustEntries = collectRustDependencies();
const mergedEntries = [...npmEntries, ...rustEntries];
const grouped = groupByLicenseBody(mergedEntries);
const totalLibraries = grouped.reduce((acc, g) => acc + g.libraries.length, 0);
const generatedSource = emit(grouped);
writeFileSync(outputPath, generatedSource);
process.stdout.write(
  `wrote ${outputPath} (${grouped.length} license groups, ${totalLibraries} libraries)\n`,
);
