#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

const source = path.resolve("src-tauri/icons/ios");
const destination = path.resolve(
  "src-tauri/gen/apple/Assets.xcassets/AppIcon.appiconset",
);

if (!fs.existsSync(destination)) {
  console.log("iOS project is not initialized; icon sync skipped.");
  process.exit(0);
}

const icons = fs.readdirSync(source).filter((file) => file.endsWith(".png"));

if (icons.length !== 18) {
  console.error(`Expected 18 iOS icons, found ${icons.length}.`);
  process.exit(1);
}

for (const icon of icons) {
  fs.copyFileSync(path.join(source, icon), path.join(destination, icon));
}

console.log(`Synchronized ${icons.length} iOS bear icons.`);
