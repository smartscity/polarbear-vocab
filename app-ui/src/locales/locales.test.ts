import { describe, expect, it } from "vitest";

import en from "./en.json";
import zhCn from "./zh-CN.json";

describe("locale contracts", () => {
  it("keeps English and Simplified Chinese keys in sync", () => {
    expect(Object.keys(zhCn).sort()).toEqual(Object.keys(en).sort());
  });

  it("keeps interpolation placeholders in sync", () => {
    for (const key of Object.keys(en) as Array<keyof typeof en>) {
      expect(placeholders(zhCn[key]), key).toEqual(placeholders(en[key]));
    }
  });
});

function placeholders(value: string): string[] {
  return [...value.matchAll(/\{([^}]+)\}/g)].map((match) => match[1]).sort();
}
