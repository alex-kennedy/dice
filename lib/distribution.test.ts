import { describe, expect, it, vi } from "vitest";

import type * as DiceLib from "@/crates/dice-wasm/pkg/dice_wasm.js";

import { computeRow } from "./distribution";

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
      free,
    }));

    const result = computeRow(lib, "2d6");

    expect(result).toEqual({
      status: "ok",
      minimum: 2,
      maximum: 12,
      mode: 7,
      pmf: new Float64Array([1, 2, 3]),
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
