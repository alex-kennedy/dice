"use client";

import { useMemo, useState } from "react";

import { Input } from "@/components/input";
import { useDiceLib } from "@/hooks/useDiceLib";

type Result = { minimum: number; maximum: number; median: number } | { error: string };

export default function Home() {
  const diceLibState = useDiceLib();
  const [query, setQuery] = useState("2d6 + 3");

  const result = useMemo<Result | null>(() => {
    if (diceLibState.status !== "ready" || query.trim() === "") return null;

    try {
      const distribution = diceLibState.lib.calculate_distribution(query);
      const { minimum, pmf } = distribution;
      const maximum = minimum + pmf.length - 1;

      let median = maximum;
      let cumulative = 0;
      for (let i = 0; i < pmf.length; i++) {
        cumulative += pmf[i];
        if (cumulative >= 0.5) {
          median = minimum + i;
          break;
        }
      }

      return { minimum, maximum, median };
    } catch (error) {
      return { error: String(error) };
    }
  }, [diceLibState, query]);

  return (
    <main className="flex flex-1 flex-col items-center justify-center gap-4 p-8">
      <Input
        className="max-w-xs"
        value={query}
        onChange={(event) => setQuery(event.target.value)}
        placeholder="2d6 + 3"
        disabled={diceLibState.status !== "ready"}
      />

      {result &&
        ("error" in result ? (
          <p className="text-center text-sm text-destructive">{result.error}</p>
        ) : (
          <p className="text-center text-sm">
            min {result.minimum} · max {result.maximum} · median {result.median}
          </p>
        ))}
    </main>
  );
}
