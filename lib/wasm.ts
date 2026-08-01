import type * as DiceWasm from "@/crates/dice-wasm/pkg/dice_wasm.js";

let modulePromise: Promise<typeof DiceWasm> | null = null;

async function instantiate(): Promise<typeof DiceWasm> {
  const wasm = await import("@/crates/dice-wasm/pkg/dice_wasm.js");
  await wasm.default();
  return wasm;
}

/**
 * Loads and initializes the dice wasm module.
 */
export function loadWasm(): Promise<typeof DiceWasm> {
  modulePromise ??= instantiate().catch((error) => {
    // Drop the cached rejection so a later call can retry.
    modulePromise = null;
    throw error;
  });
  return modulePromise;
}
