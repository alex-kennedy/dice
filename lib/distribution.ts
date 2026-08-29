import type * as DiceLib from "@/crates/dice-wasm/pkg/dice_wasm.js";

/** The result of computing the distribution for an expression. */
export type RowComputation =
  | { status: "empty" }
  | { status: "error"; message: string }
  | {
      status: "ok";
      minimum: number;
      maximum: number;
      mode: number;
      pmf: Float64Array;
    };

/** Computes the distribution for a single expression (row). */
export function computeRow(
  lib: typeof DiceLib,
  expression: string,
): RowComputation {
  if (expression.trim() === "") return { status: "empty" };

  try {
    const distribution = lib.calculate_distribution(expression);
    const { minimum, maximum, mode, pmf } = distribution;
    distribution.free();
    return { status: "ok", minimum, maximum, mode, pmf };
  } catch (error) {
    return { status: "error", message: String(error) };
  }
}
