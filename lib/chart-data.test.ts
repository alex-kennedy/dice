import { describe, expect, it } from "vitest";

import { buildChartData } from "./chart-data";
import { createRow } from "./dice-rows";
import type { RowComputation } from "./distribution";

describe("buildChartData", () => {
  it("returns an empty array when there are no rows", () => {
    const rows = [createRow("+")];
    const results: Record<string, RowComputation> = {
      [rows[0].id]: { status: "error", message: "bad" },
    };
    expect(buildChartData(rows, results)).toEqual([]);
  });

  it("bounds are accurate", () => {
    const rows = [createRow("d4"), createRow("2d4")];
    const [d4, twoD4] = rows;
    const results: Record<string, RowComputation> = {
      [d4.id]: {
        status: "ok",
        minimum: 1,
        maximum: 4,
        mode: 1,
        pmf: new Float64Array([0.25, 0.25, 0.25, 0.25]),
        cdf: new Float64Array([0.25, 0.5, 0.75, 1]),
      },
      [twoD4.id]: {
        status: "ok",
        minimum: 2,
        maximum: 8,
        mode: 5,
        pmf: new Float64Array([1, 2, 3, 4, 3, 2, 1]).map((n) => n / 16),
        cdf: new Float64Array([1, 3, 6, 10, 13, 15, 16]).map((n) => n / 16),
      },
    };

    const data = buildChartData(rows, results);

    expect(data.map((point) => point.outcome)).toEqual([
      1, 2, 3, 4, 5, 6, 7, 8,
    ]);
    expect(data[0][d4.id]).toBe(0.25);
    expect(data[0][twoD4.id]).toBeNull();
    expect(data[7][d4.id]).toBeNull();
    expect(data[7][twoD4.id]).toBeCloseTo(1 / 16);
  });
});
