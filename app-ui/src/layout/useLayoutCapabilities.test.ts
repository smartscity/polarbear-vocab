import { describe, expect, it } from "vitest";

import { classifyLayoutCapabilities } from "./useLayoutCapabilities";

describe("classifyLayoutCapabilities", () => {
  it.each([
    [639, "compact"],
    [640, "medium"],
    [1023, "medium"],
    [1024, "wide"],
  ] as const)("maps width %i to %s", (width, layout) => {
    expect(classifyLayoutCapabilities(width, false)).toEqual({ input: "pointer", layout });
  });

  it("keeps touch capability independent from viewport width", () => {
    expect(classifyLayoutCapabilities(1280, true)).toEqual({ input: "touch", layout: "wide" });
  });
});
