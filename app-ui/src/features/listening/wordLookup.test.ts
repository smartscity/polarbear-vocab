import { describe, expect, it } from "vitest";

import { lexiconLookupCandidates } from "./wordLookup";

describe("listening word lookup", () => {
  it("can reach a Lexicon lemma from a selected inflected word", () => {
    expect(lexiconLookupCandidates("earned")).toContain("earn");
    expect(lexiconLookupCandidates("studied")).toContain("study");
    expect(lexiconLookupCandidates("earning")).toContain("earn");
    expect(lexiconLookupCandidates("travels")).toContain("travel");
  });
});
