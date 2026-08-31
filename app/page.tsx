"use client";

import Link from "next/link";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { Fragment, Suspense, useEffect, useMemo, useState } from "react";
import { Collapsible } from "@base-ui/react/collapsible";
import { FiChevronRight } from "react-icons/fi";
import { SiGithub } from "react-icons/si";

import { D20Icon } from "@/components/d20-icon";
import { DiceChart } from "@/components/dice-chart";
import { DiceRow } from "@/components/dice-row";
import { ShareButton } from "@/components/share-button";
import { useDiceLib } from "@/hooks/useDiceLib";
import { cn } from "@/lib/cn";
import {
  createRow,
  encodeRowsParam,
  parseRowsParam,
  type DiceRow as DiceRowData,
} from "@/lib/dice-rows";
import { computeRow, type RowComputation } from "@/lib/distribution";

const HELP_ENTRIES: { expression: string; description: string }[] = [
  { expression: "d20", description: "Roll a 20-sided die." },
  {
    expression: "2d20",
    description: "Roll two 20-sided dice and add the result.",
  },
  {
    expression: "d20a",
    description: "Advantage - roll two dice and take the best.",
  },
  {
    expression: "d20d",
    description: "Disadvantage - roll two dice and take the worst.",
  },
  {
    expression: "d10+2",
    description: "Roll a 10-side die and add 2.",
  },
  {
    expression: "d10+d4",
    description: "Roll a 10-side die and a 4-sided die, add the result.",
  },
  {
    expression: "5d6a10",
    description: "Roll 10 6-sided dice and take the sum of the best 5.",
  },
  {
    expression: "2(d6+d4)",
    description:
      "Roll a d6 and a d4 and add them, repeat this and add the whole result. Not the same as doubling the result.",
  },
  {
    expression: "3(2(d6+d4)-d8+2d6a8)",
    description: "Build complex expressions.",
  },
];

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

  const [rows, setRows] = useState<DiceRowData[]>(() =>
    parseRowsParam(searchParams.get("q")),
  );
  const [helpOpen, setHelpOpen] = useState(false);

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
    router.replace(`${pathname}${encoded ? `?q=${encoded}` : ""}`, {
      scroll: false,
    });
  }, [rows, pathname, router]);

  const updateExpression = (id: string, expression: string) =>
    setRows((current) =>
      current.map((row) => (row.id === id ? { ...row, expression } : row)),
    );

  const addRow = () => setRows((current) => [...current, createRow()]);

  const removeRow = (id: string) =>
    setRows((current) => current.filter((row) => row.id !== id));

  const ready = diceLibState.status === "ready";

  return (
    <div className="mx-auto flex w-full max-w-7xl flex-1 flex-col">
      <header className="flex items-center justify-between px-8 pt-4">
        <Link href="/" aria-label="Home" className="text-foreground">
          <D20Icon className="h-6 w-6" />
        </Link>
        <div className="flex items-center gap-4">
          <ShareButton />
          <a
            href="https://github.com/alex-kennedy/dice"
            target="_blank"
            rel="noreferrer"
            aria-label="View source on GitHub"
            className="text-muted-foreground hover:text-foreground"
          >
            <SiGithub className="h-5 w-5" />
          </a>
        </div>
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

          <Collapsible.Root open={helpOpen} onOpenChange={setHelpOpen}>
            <Collapsible.Trigger className="flex items-center gap-1 self-start text-sm text-muted-foreground hover:text-foreground">
              Help
              <FiChevronRight
                className={cn(
                  "h-3.5 w-3.5 transition-transform",
                  helpOpen && "rotate-90",
                )}
              />
            </Collapsible.Trigger>

            <Collapsible.Panel
              className="overflow-hidden transition-all duration-150 ease-out data-ending-style:h-0 data-starting-style:h-0"
              style={{ height: "var(--collapsible-panel-height)" }}
            >
              <div className="mt-2 grid grid-cols-[auto_1fr] items-center gap-x-3 gap-y-3 rounded-lg border border-border bg-background p-4 shadow-sm">
                {HELP_ENTRIES.map((entry) => (
                  <Fragment key={entry.expression}>
                    <code className="w-fit rounded-md bg-input px-1.5 py-0.5 font-mono text-xs text-foreground">
                      {entry.expression}
                    </code>
                    <span className="text-sm text-muted-foreground">
                      {entry.description}
                    </span>
                  </Fragment>
                ))}
              </div>
            </Collapsible.Panel>
          </Collapsible.Root>
        </div>
      </main>

      <footer className="flex items-center justify-center gap-2 px-8 pb-4 text-xs text-muted-foreground">
        <a
          href="https://github.com/alex-kennedy/dice"
          target="_blank"
          rel="noreferrer"
          className="hover:text-foreground"
        >
          Source Code
        </a>
        <span aria-hidden="true">·</span>
        <Link href="/help" className="hover:text-foreground">
          How it Works
        </Link>
      </footer>
    </div>
  );
}
