import { describe, expect, it, vi } from "vitest";

import type * as DiceLib from "@/crates/dice-wasm/pkg/dice_wasm.js";

import { cdfAt, computeRow, type RowComputation } from "./distribution";

function fakeLib(calculate_distribution: (query: string) => unknown) {
  return { calculate_distribution } as unknown as typeof DiceLib;
}

describe("computeRow", () => {
  it("skips the wasm call for an empty expression", () => {
    const calculate_distribution = vi.fn();
    computeRow(fakeLib(calculate_distribution), "   ");
    expect(calculate_distribution).not.toHaveBeenCalled();
  });

  it("reads fields and frees the distribution on success", () => {
    const free = vi.fn();
    const lib = fakeLib(() => ({
      minimum: 2,
      maximum: 12,
      mode: 7,
      pmf: new Float64Array([1, 2, 3]),
      cdf: new Float64Array([1, 3, 6]),
      free,
    }));

    const result = computeRow(lib, "2d6");

    expect(result).toEqual({
      status: "ok",
      minimum: 2,
      maximum: 12,
      mode: 7,
      pmf: new Float64Array([1, 2, 3]),
      cdf: new Float64Array([1, 3, 6]),
    });
    expect(free).toHaveBeenCalledOnce();
  });

  it("surfaces a thrown error without freeing anything", () => {
    const lib = fakeLib(() => {
      throw "unexpected token";
    });

    expect(computeRow(lib, "+")).toEqual({ status: "error", message: "unexpected token" });
  });
});

describe("cdfAt", () => {
  const d4: RowComputation = {
    status: "ok",
    minimum: 1,
    maximum: 4,
    mode: 1,
    pmf: new Float64Array([0.25, 0.25, 0.25, 0.25]),
    cdf: new Float64Array([0.25, 0.5, 0.75, 1]),
  };

  it("is 0 for an error or empty row", () => {
    expect(cdfAt({ status: "error", message: "bad" }, 2)).toBe(0);
    expect(cdfAt({ status: "empty" }, 2)).toBe(0);
  });

  it("is 0 below the minimum and 1 at or above the maximum", () => {
    expect(cdfAt(d4, 0)).toBe(0);
    expect(cdfAt(d4, 4)).toBe(1);
    expect(cdfAt(d4, 10)).toBe(1);
  });

  it("reads the cumulative probability for the outcome", () => {
    expect(cdfAt(d4, 1)).toBeCloseTo(0.25);
    expect(cdfAt(d4, 2)).toBeCloseTo(0.5);
    expect(cdfAt(d4, 3)).toBeCloseTo(0.75);
  });
});
