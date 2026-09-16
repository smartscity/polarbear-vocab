import { describe, expect, it } from "vitest";

import type { DatasetSummary } from "../../lib/commands";
import { moveDataset } from "./DatasetList";

const datasets: DatasetSummary[] = ["first", "second", "third"].map((id, index) => ({
  id,
  name: id,
  createdAt: index,
  updatedAt: index,
  preloaded: false,
  wordCount: 0,
}));

describe("moveDataset", () => {
  it("moves a dataset to the hovered position", () => {
    expect(moveDataset(datasets, "first", "third").map((dataset) => dataset.id))
      .toEqual(["second", "third", "first"]);
    expect(moveDataset(datasets, "third", "first").map((dataset) => dataset.id))
      .toEqual(["third", "first", "second"]);
  });

  it("does not change the list for an unknown target", () => {
    expect(moveDataset(datasets, "first", "missing")).toBe(datasets);
  });
});
