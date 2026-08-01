"use client";

import { useEffect, useState } from "react";

import { loadWasm } from "@/lib/wasm";

export default function Home() {
  const [greeting, setGreeting] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    loadWasm()
      .then((wasm) => {
        if (!cancelled) {
          setGreeting(wasm.get_greeting());
        }
      })
      .catch(console.error);

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <main className="flex flex-1 flex-col items-center justify-center gap-4">
      <h1 className="text-4xl font-semibold tracking-tight">dice</h1>
      {greeting && (
        <p className="text-lg text-zinc-600 dark:text-zinc-400">{greeting}</p>
      )}
    </main>
  );
}
