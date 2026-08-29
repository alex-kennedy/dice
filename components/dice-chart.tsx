import { CartesianGrid, Line, LineChart, XAxis, YAxis } from "recharts";

import { ChartContainer, type ChartConfig } from "@/components/chart";
import { buildChartData } from "@/lib/chart-data";
import type { DiceRow } from "@/lib/dice-rows";
import type { RowComputation } from "@/lib/distribution";
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
      <LineChart data={data}>
        <CartesianGrid vertical={false} />
        <XAxis
          dataKey="outcome"
          type="number"
          allowDecimals={false}
          domain={["dataMin", "dataMax"]}
        />
        <YAxis tickFormatter={(value: number) => `${Math.round(value * 100)}%`} />
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
