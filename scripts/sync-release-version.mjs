#!/usr/bin/env node

import fs from "node:fs";

const version = process.argv[2] ?? process.env.RELEASE_VERSION;

if (!version) {
  console.error("Missing release version. Usage: node scripts/sync-release-version.mjs <version>");
  process.exit(1);
}

if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version)) {
  console.error(`Invalid release version: ${version}`);
  process.exit(1);
}

const jsonFiles = [
  "package.json",
  "app-ui/package.json",
  "src-tauri/tauri.conf.json",
];

for (const file of jsonFiles) {
  if (!fs.existsSync(file)) {
    console.error(`Missing ${file}`);
    process.exit(1);
  }

  const json = JSON.parse(fs.readFileSync(file, "utf8"));
  json.version = version;
  fs.writeFileSync(file, `${JSON.stringify(json, null, 2)}\n`);
  console.log(`${file}: ${version}`);
}

const cargoFile = "Cargo.toml";

if (!fs.existsSync(cargoFile)) {
  console.error(`Missing ${cargoFile}`);
  process.exit(1);
}

const lines = fs.readFileSync(cargoFile, "utf8").split("\n");

let inWorkspacePackage = false;
let updatedCargoVersion = false;

for (let i = 0; i < lines.length; i += 1) {
  const trimmed = lines[i].trim();

  if (trimmed === "[workspace.package]") {
    inWorkspacePackage = true;
    continue;
  }

  if (
    inWorkspacePackage &&
    trimmed.startsWith("[") &&
    trimmed.endsWith("]")
  ) {
    inWorkspacePackage = false;
  }

  if (inWorkspacePackage && /^version\s*=/.test(trimmed)) {
    lines[i] = `version = "${version}"`;
    updatedCargoVersion = true;
    break;
  }
}

if (!updatedCargoVersion) {
  console.error("Missing version in [workspace.package] of Cargo.toml");
  process.exit(1);
}

fs.writeFileSync(cargoFile, lines.join("\n"));

console.log(`Cargo.toml [workspace.package]: ${version}`);

for (const file of jsonFiles) {
  const actual = JSON.parse(fs.readFileSync(file, "utf8")).version;

  if (actual !== version) {
    console.error(
      `Version sync failed: ${file}=${actual}, expected=${version}`
    );
    process.exit(1);
  }
}

const cargoText = fs.readFileSync(cargoFile, "utf8");
const workspaceSection =
  cargoText.match(/\[workspace\.package\]([\s\S]*?)(?:\n\[|$)/)?.[1] ?? "";

const cargoVersion =
  workspaceSection.match(/^version\s*=\s*"([^"]+)"/m)?.[1];

if (cargoVersion !== version) {
  console.error(
    `Version sync failed: Cargo.toml=${cargoVersion ?? "missing"}, expected=${version}`
  );
  process.exit(1);
}

console.log(`Release version synchronized successfully: ${version}`);
