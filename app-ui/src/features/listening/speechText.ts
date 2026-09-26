import type { Nodes, Root } from "mdast";
import remarkParse from "remark-parse";
import { unified } from "unified";

export function markdownToSpeech(markdown: string, includeHeadings = true): string {
  const tree = unified().use(remarkParse).parse(markdown);
  const paragraphs: string[] = [];
  collectSpeechParagraphs(tree, paragraphs, includeHeadings);
  return paragraphs.join("\n\n");
}

function collectSpeechParagraphs(node: Root | Nodes, output: string[], includeHeadings: boolean): void {
  if (node.type === "paragraph" || (includeHeadings && node.type === "heading")) {
    const text = inlineText(node).replace(/\s+/g, " ").trim();
    if (text) output.push(text);
    return;
  }
  if ("children" in node) {
    for (const child of node.children) collectSpeechParagraphs(child, output, includeHeadings);
  }
}

function inlineText(node: Nodes): string {
  if (node.type === "text" || node.type === "inlineCode") return node.value;
  if (node.type === "break") return " ";
  if ("children" in node) return node.children.map(inlineText).join("");
  return "";
}
