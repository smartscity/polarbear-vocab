import { searchLexicon, type LexiconEntry } from "../../lib/commands";

export function lexiconLookupCandidates(selected: string): string[] {
  const word = selected.toLowerCase();
  const candidates = [word];
  if (word.endsWith("ied") && word.length > 4) candidates.push(`${word.slice(0, -3)}y`);
  if (word.endsWith("ing") && word.length > 5) {
    candidates.push(word.slice(0, -3), `${word.slice(0, -3)}e`);
  }
  if (word.endsWith("ed") && word.length > 4) {
    candidates.push(word.slice(0, -2), word.slice(0, -1));
  }
  if (word.endsWith("ies") && word.length > 4) candidates.push(`${word.slice(0, -3)}y`);
  if (word.endsWith("es") && word.length > 4) candidates.push(word.slice(0, -2));
  if (word.endsWith("s") && word.length > 3) candidates.push(word.slice(0, -1));
  return [...new Set(candidates)];
}

export async function findLexiconWord(word: string): Promise<LexiconEntry | null> {
  for (const candidate of lexiconLookupCandidates(word)) {
    const results = await searchLexicon(candidate);
    const found = results.find((entry) => entry.lemma.toLowerCase() === candidate);
    if (found) return found;
  }
  return null;
}
