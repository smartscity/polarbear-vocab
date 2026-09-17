import { gunzipSync } from "fflate";
import type { Root, Text } from "mdast";
import remarkParse from "remark-parse";
import remarkStringify from "remark-stringify";
import { unified } from "unified";
import { visit } from "unist-util-visit";

const MODEL_BASE = "/models/en-zh";
const MODEL_HASHES = {
  lex: "8575d8daa10e2dbff316dcdf8e1ce475357bcc2c92bdc63b736a2d5add22f681",
  model: "4e5accc141373565ddc8fa1565bceaa8d0c3482a82cab8131c719ebcc6c2157c",
  srcvocab: "bd9b65504acc6d9726dd281f7defc2adb7c2c22d0688fe2f84697de25197c8c5",
  trgvocab: "aded6993c36e440284d11cec3f6b8aef9c0e43188a772d80be342a713adf223d",
} as const;
const TRANSLATION_BATCH_SIZE = 32;

interface TextTranslator {
  translate(request: { from: string; to: string; text: string; html?: boolean }): Promise<{
    target: { text: string };
  }>;
}

let translatorPromise: Promise<TextTranslator> | undefined;

export async function translateEnglishMarkdown(
  markdown: string,
  translator = loadTranslator,
): Promise<string> {
  const processor = unified().use(remarkParse).use(remarkStringify);
  const tree = processor.parse(markdown);
  const nodes = collectEnglishTextNodes(tree);
  const engine = await translator();
  await translateNodes(engine, nodes);
  return processor.stringify(tree).trim();
}

async function translateNodes(translator: TextTranslator, nodes: Text[]): Promise<void> {
  for (let start = 0; start < nodes.length; start += TRANSLATION_BATCH_SIZE) {
    const batch = nodes.slice(start, start + TRANSLATION_BATCH_SIZE);
    const translations = await Promise.all(
      batch.map((node) => translateText(translator, node.value)),
    );
    batch.forEach((node, index) => { node.value = translations[index]; });
  }
}

function collectEnglishTextNodes(tree: Root): Text[] {
  const nodes: Text[] = [];
  visit(tree, "text", (node: Text) => {
    if (/[A-Za-z]/.test(node.value)) nodes.push(node);
  });
  return nodes;
}

async function translateText(translator: TextTranslator, text: string): Promise<string> {
  const leading = text.match(/^\s*/)?.[0] ?? "";
  const trailing = text.match(/\s*$/)?.[0] ?? "";
  const end = trailing.length > 0 ? -trailing.length : undefined;
  const content = text.slice(leading.length, end);
  const response = await translator.translate({ from: "en", to: "zh", text: content, html: false });
  return `${leading}${response.target.text.trim()}${trailing}`;
}

async function loadTranslator(): Promise<TextTranslator> {
  translatorPromise ??= createTranslator();
  return translatorPromise;
}

async function createTranslator(): Promise<TextTranslator> {
  const { BatchTranslator } = await import("@browsermt/bergamot-translator/translator.js");
  return new BatchTranslator(
    { batchSize: 8, downloadTimeout: 300_000, workers: 1 },
    await createModelBacking(),
  );
}

async function createModelBacking() {
  const { TranslatorBacking } = await import("@browsermt/bergamot-translator/translator.js");
  return new (class extends TranslatorBacking {
    override async loadModelRegistery() {
      return [{ from: "en", to: "zh", files: modelFiles() }];
    }

    override async fetch(url: string, checksum?: string, options?: { signal?: AbortSignal }) {
      const response = await fetch(url, { credentials: "same-origin", signal: options?.signal });
      if (!response.ok) throw new Error(`Bundled translation model is missing (${response.status})`);
      const compressed = await response.arrayBuffer();
      const downloaded = new Uint8Array(compressed);
      const bytes = decodeBundledModel(downloaded);
      if (checksum) await verifySha256(bytes, checksum);
      return exactBuffer(bytes);
    }
  })({ downloadTimeout: 300_000 });
}

export function decodeBundledModel(bytes: Uint8Array): Uint8Array {
  const isGzip = bytes.length >= 2 && bytes[0] === 0x1f && bytes[1] === 0x8b;
  return isGzip ? gunzipSync(bytes) : bytes;
}

function modelFiles() {
  return {
    lex: { name: `${MODEL_BASE}/lex.50.50.enzh.s2t.bin.gz`, expectedSha256Hash: MODEL_HASHES.lex },
    model: { name: `${MODEL_BASE}/model.enzh.intgemm.alphas.bin.gz`, expectedSha256Hash: MODEL_HASHES.model },
    srcvocab: { name: `${MODEL_BASE}/srcvocab.enzh.spm.gz`, expectedSha256Hash: MODEL_HASHES.srcvocab },
    trgvocab: { name: `${MODEL_BASE}/trgvocab.enzh.spm.gz`, expectedSha256Hash: MODEL_HASHES.trgvocab },
  };
}

async function verifySha256(bytes: Uint8Array, expected: string): Promise<void> {
  const digest = await crypto.subtle.digest("SHA-256", exactBuffer(bytes));
  const actual = [...new Uint8Array(digest)].map((value) => value.toString(16).padStart(2, "0")).join("");
  if (actual !== expected) throw new Error("Translation model integrity check failed");
}

function exactBuffer(bytes: Uint8Array): ArrayBuffer {
  return bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength) as ArrayBuffer;
}
