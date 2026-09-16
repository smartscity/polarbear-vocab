import { gunzipSync } from "fflate";
import type { Text } from "mdast";
import remarkParse from "remark-parse";
import remarkStringify from "remark-stringify";
import { unified } from "unified";
import { visit } from "unist-util-visit";

const MODEL_BASE = "https://storage.googleapis.com/moz-fx-translations-data--303e-prod-translations-data/models/en-zh/llmaat_finetune10M_qe8_f2_ByQcSxGXQRqGi-UTxYE43g/exported";
const MODEL_CACHE = "polarbear-vocab-en-zh-v1";
const MODEL_HASH = "4e5accc141373565ddc8fa1565bceaa8d0c3482a82cab8131c719ebcc6c2157c";

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
  const translations = await Promise.all(nodes.map((node) => translateText(engine, node.value)));
  nodes.forEach((node, index) => { node.value = translations[index]; });
  return processor.stringify(tree).trim();
}

function collectEnglishTextNodes(tree: ReturnType<ReturnType<typeof unified>["parse"]>): Text[] {
  const nodes: Text[] = [];
  visit(tree, "text", (node: Text) => {
    if (/[A-Za-z]/.test(node.value)) nodes.push(node);
  });
  return nodes;
}

async function translateText(translator: TextTranslator, text: string): Promise<string> {
  const response = await translator.translate({ from: "en", to: "zh", text, html: false });
  return response.target.text.trim();
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
      const compressed = await fetchCached(url, options?.signal);
      const bytes = gunzipSync(new Uint8Array(compressed));
      if (checksum) await verifySha256(bytes, checksum);
      return exactBuffer(bytes);
    }
  })({ downloadTimeout: 300_000 });
}

function modelFiles() {
  return {
    lex: { name: `${MODEL_BASE}/lex.50.50.enzh.s2t.bin.gz` },
    model: { name: `${MODEL_BASE}/model.enzh.intgemm.alphas.bin.gz`, expectedSha256Hash: MODEL_HASH },
    srcvocab: { name: `${MODEL_BASE}/srcvocab.enzh.spm.gz` },
    trgvocab: { name: `${MODEL_BASE}/trgvocab.enzh.spm.gz` },
  };
}

async function fetchCached(url: string, signal?: AbortSignal): Promise<ArrayBuffer> {
  const cache = "caches" in window ? await caches.open(MODEL_CACHE) : undefined;
  let response = await cache?.match(url);
  if (!response) {
    response = await fetch(url, { credentials: "omit", signal });
    if (!response.ok) throw new Error(`Translation model download failed (${response.status})`);
    await cache?.put(url, response.clone());
  }
  return response.arrayBuffer();
}

async function verifySha256(bytes: Uint8Array, expected: string): Promise<void> {
  const digest = await crypto.subtle.digest("SHA-256", exactBuffer(bytes));
  const actual = [...new Uint8Array(digest)].map((value) => value.toString(16).padStart(2, "0")).join("");
  if (actual !== expected) throw new Error("Translation model integrity check failed");
}

function exactBuffer(bytes: Uint8Array): ArrayBuffer {
  return bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength) as ArrayBuffer;
}
