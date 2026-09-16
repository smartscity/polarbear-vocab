import { useRef, useState } from "react";

import {
  addToMyVocabulary,
  removeFromMyVocabulary,
  searchLexicon,
  type LexiconEntry,
} from "../../lib/commands";

export function useLexiconSearch(onError: (error: unknown) => void) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<LexiconEntry[]>([]);
  const [busy, setBusy] = useState(false);
  const [searched, setSearched] = useState(false);
  const requestId = useRef(0);

  const updateQuery = (nextQuery: string) => {
    requestId.current += 1;
    setQuery(nextQuery);
    setResults([]);
    setSearched(false);
    setBusy(false);
  };

  const search = async (nextQuery = query) => {
    const normalized = nextQuery.trim();
    if (!normalized) return;
    const currentRequest = ++requestId.current;
    setBusy(true);
    try {
      const matches = await searchLexicon(normalized);
      if (currentRequest === requestId.current) {
        setResults(matches);
        setSearched(true);
      }
    } catch (error) {
      if (currentRequest === requestId.current) onError(error);
    } finally {
      if (currentRequest === requestId.current) setBusy(false);
    }
  };

  const toggleVocabulary = async (entry: LexiconEntry) => {
    setBusy(true);
    try {
      if (entry.inMyVocabulary) await removeFromMyVocabulary(entry.senseUid);
      else await addToMyVocabulary(entry.senseUid);
      setResults((current) => current.map((item) => (
        item.senseUid === entry.senseUid
          ? { ...item, inMyVocabulary: !item.inMyVocabulary }
          : item
      )));
    } catch (error) {
      onError(error);
    } finally {
      setBusy(false);
    }
  };

  return { busy, query, results, search, searched, setQuery: updateQuery, toggleVocabulary };
}
