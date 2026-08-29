"use client";

import { useEffect, useState } from "react";

import type * as DiceLib from "@/crates/dice-wasm/pkg/dice_wasm.js";
import { loadWasm } from "@/lib/wasm";

export type DiceLibState =
  | { status: "loading" }
  | { status: "ready"; lib: typeof DiceLib }
  | { status: "error"; error: unknown };

declare global {
  interface Window {
    diceLib?: typeof DiceLib;
  }
}

/** Loads the dice web assembly library. */
export function useDiceLib(): DiceLibState {
  const [state, setState] = useState<DiceLibState>({ status: "loading" });

  useEffect(() => {
    loadWasm()
      .then((lib) => {
        setState({ status: "ready", lib });

        // For testing, attach the library to the window so we can play with it
        // in the console.
        if (process.env.NODE_ENV !== "production") {
          window.diceLib = lib;
        }
      })
      .catch((error) => setState({ status: "error", error }));
  }, []);

  return state;
}
