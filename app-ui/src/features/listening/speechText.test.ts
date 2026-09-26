import { expect, test } from "vitest";

import { markdownToSpeech } from "./speechText";

test("listening narration omits list numbers and Markdown notation", () => {
  expect(markdownToSpeech("# At the restaurant\n\n1. Could I see the **menu**?\n2. The bill, please.", false))
    .toBe("Could I see the menu?\n\nThe bill, please.");
});

test("article narration retains headings and ignores code blocks", () => {
  expect(markdownToSpeech("## Meeting notes\n\nPlease review the [patch](https://example.com).\n\n```ts\nconst x = 1;\n```"))
    .toBe("Meeting notes\n\nPlease review the patch.");
});
