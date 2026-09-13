import { describe, expect, it } from "vitest";

import { browserAppInfo } from "./branding";

describe("browserAppInfo", () => {
  it("uses the canonical product and knowledge-module names", () => {
    expect(browserAppInfo.name).toBe("Polarbear Vocab");
    expect(browserAppInfo.knowledgeModule).toBe("Polarbear Lexicon");
  });
});

