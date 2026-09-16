import { describe, expect, it } from "vitest";

import { translateEnglishMarkdown } from "./articleTranslation";

describe("article translation", () => {
  it("translates prose while preserving Markdown structure and code", async () => {
    const markdown = "# Guide\n\nHello **world**.\n\n```ts\nconst greeting = 'Hello';\n```\n\n[OpenAI](https://openai.com)";
    const translated = await translateEnglishMarkdown(markdown, async () => ({
      translate: async ({ text }) => ({ target: { text: `译:${text}` } }),
    }));

    expect(translated).toContain("# 译:Guide");
    expect(translated).toContain("译:Hello **译:world**");
    expect(translated).toContain("const greeting = 'Hello';");
    expect(translated).toContain("[译:OpenAI](https://openai.com)");
  });
});
