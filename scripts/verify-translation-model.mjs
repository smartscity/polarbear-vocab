#!/usr/bin/env node

import { createHash } from "node:crypto";
import { createReadStream } from "node:fs";
import { stat } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const modelDirectory = path.join(root, "app-ui/public/models/en-zh");
const expectedFiles = {
  "lex.50.50.enzh.s2t.bin.gz": [2536039, "806f75821c0b838f4a8f4afe5bab3db8289cb7e5187753ba04c3bceadd75687a"],
  "model.enzh.intgemm.alphas.bin.gz": [33375922, "7f255403b3bb2502f08ac4d5ca397a8a5a13f899d2f2e987a4934e089d241d16"],
  "srcvocab.enzh.spm.gz": [407784, "7846e3c236388390f4e5d321f8413d67f34c1bab5f066165eeb673bfd07607cc"],
  "trgvocab.enzh.spm.gz": [425748, "4d641ce165b1f8478ee2ffb5149d2d46fab3779dc8fa1e9b97f9af1d2206c091"],
};

for (const [name, [expectedSize, expectedHash]] of Object.entries(expectedFiles)) {
  const file = path.join(modelDirectory, name);
  const details = await stat(file).catch(() => undefined);
  if (!details || details.size !== expectedSize) fail(`${name} is missing or incomplete`);
  const actualHash = await sha256(file);
  if (actualHash !== expectedHash) fail(`${name} failed its SHA-256 check`);
}

console.log("Verified the bundled English-Chinese translation model.");

function sha256(file) {
  return new Promise((resolve, reject) => {
    const hash = createHash("sha256");
    createReadStream(file).on("data", (chunk) => hash.update(chunk)).on("error", reject)
      .on("end", () => resolve(hash.digest("hex")));
  });
}

function fail(message) {
  console.error(`Translation model verification failed: ${message}.`);
  process.exit(1);
}
