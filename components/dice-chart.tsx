import { CartesianGrid, Line, LineChart, Tooltip, XAxis, YAxis } from "recharts";

import { ChartContainer, type ChartConfig } from "@/components/chart";
import { buildChartData } from "@/lib/chart-data";
import type { DiceRow } from "@/lib/dice-rows";
import { cdfAt, type RowComputation } from "@/lib/distribution";
import { chartColor } from "@/lib/palette";

type DiceChartProps = {
  rows: DiceRow[];
  results: Record<string, RowComputation>;
};

export function DiceChart({ rows, results }: DiceChartProps) {
  const data = buildChartData(rows, results);

  if (data.length === 0) {
    return (
      <div className="flex aspect-video w-full items-center justify-center text-sm text-muted-foreground">
        Add some dice to get this rolling
      </div>
    );
  }

  const config = Object.fromEntries(
    rows.map((row, index) => [
      row.id,
      { label: row.expression || "…", color: chartColor(index) },
    ]),
  ) as ChartConfig;

  return (
    <ChartContainer config={config} className="w-full">
      <LineChart data={data} margin={{ left: -16 }}>
        <CartesianGrid vertical={false} />
        <XAxis
          dataKey="outcome"
          type="number"
          allowDecimals={false}
          domain={["dataMin", "dataMax"]}
        />
        <YAxis tickFormatter={(value: number) => `${Math.round(value * 100)}%`} />
        <Tooltip content={<CdfTooltip rows={rows} results={results} />} />
        {rows.map((row, index) => (
          <Line
            key={row.id}
            dataKey={row.id}
            stroke={chartColor(index)}
            strokeWidth={2}
            dot={false}
            isAnimationActive={false}
          />
        ))}
      </LineChart>
    </ChartContainer>
  );
}

type CdfTooltipProps = {
  active?: boolean;
  label?: number;
  rows: DiceRow[];
  results: Record<string, RowComputation>;
};

function CdfTooltip({ active, label, rows, results }: CdfTooltipProps) {
  if (!active || label === undefined) return null;

  return (
    <div className="rounded-md border border-border bg-background px-3 py-2 text-sm shadow-md">
      <div className="mb-1 font-medium">
        ℙ(X ≤ {label})
      </div>
      {rows.map((row, index) => {
        const result = results[row.id];
        if (result.status !== "ok") return null;

        return (
          <div key={row.id} className="flex items-center gap-2">
            <span
              className="h-2.5 w-2.5 shrink-0 rounded-full"
              style={{ backgroundColor: chartColor(index) }}
            />
            <span>{(cdfAt(result, label) * 100).toFixed(2)}%</span>
          </div>
        );
      })}
    </div>
  );
}
