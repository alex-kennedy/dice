"use client";

import Link from "next/link";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { Suspense, useEffect, useMemo, useState } from "react";
import { SiGithub } from "react-icons/si";

import { D20Icon } from "@/components/d20-icon";
import { DiceChart } from "@/components/dice-chart";
import { DiceRow } from "@/components/dice-row";
import { useDiceLib } from "@/hooks/useDiceLib";
import {
  createRow,
  encodeRowsParam,
  parseRowsParam,
  type DiceRow as DiceRowData,
} from "@/lib/dice-rows";
import { computeRow, type RowComputation } from "@/lib/distribution";

export default function Home() {
  return (
    <Suspense fallback={null}>
      <DiceComparison />
    </Suspense>
  );
}

function DiceComparison() {
  const diceLibState = useDiceLib();
  const searchParams = useSearchParams();
  const pathname = usePathname();
  const router = useRouter();

  const [rows, setRows] = useState<DiceRowData[]>(() => parseRowsParam(searchParams.get("q")));

  const results = useMemo<Record<string, RowComputation>>(
    () =>
      Object.fromEntries(
        rows.map((row) => [
          row.id,
          diceLibState.status === "ready"
            ? computeRow(diceLibState.lib, row.expression)
            : { status: "empty" as const },
        ]),
      ),
    [diceLibState, rows],
  );

  useEffect(() => {
    const encoded = encodeRowsParam(rows);
    router.replace(`${pathname}${encoded ? `?q=${encoded}` : ""}`, { scroll: false });
  }, [rows, pathname, router]);

  const updateExpression = (id: string, expression: string) =>
    setRows((current) => current.map((row) => (row.id === id ? { ...row, expression } : row)));

  const addRow = () => setRows((current) => [...current, createRow()]);

  const removeRow = (id: string) => setRows((current) => current.filter((row) => row.id !== id));

  const ready = diceLibState.status === "ready";

  return (
    <div className="mx-auto flex w-full max-w-7xl flex-1 flex-col">
      <header className="flex items-center justify-between px-8 pt-4">
        <Link href="/" aria-label="Home" className="text-foreground">
          <D20Icon className="h-6 w-6" />
        </Link>
        <a
          href="https://github.com/alex-kennedy/dice"
          target="_blank"
          rel="noreferrer"
          aria-label="View source on GitHub"
          className="text-muted-foreground hover:text-foreground"
        >
          <SiGithub className="h-5 w-5" />
        </a>
      </header>

      <main className="flex flex-1 flex-col gap-6 p-8">
        <DiceChart rows={rows} results={results} />

        <div className="flex flex-col gap-4">
          {rows.map((row, index) => (
            <DiceRow
              key={row.id}
              index={index}
              expression={row.expression}
              result={results[row.id]}
              disabled={!ready}
              onChange={(expression) => updateExpression(row.id, expression)}
              onRemove={() => removeRow(row.id)}
            />
          ))}

          <button
            type="button"
            onClick={addRow}
            className="self-start text-sm text-muted-foreground hover:text-foreground"
          >
            + Add expression
          </button>
        </div>
      </main>
    </div>
  );
}
