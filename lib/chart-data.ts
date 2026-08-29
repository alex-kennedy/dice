import type { DiceRow } from "@/lib/dice-rows";
import type { RowComputation } from "@/lib/distribution";

/**
 * Single chart point. Maps the expression ID and outcome to the probability at
 * the point (or null if it's impossible.)
 */
export type ChartPoint = { outcome: number } & Record<string, number | null>;

/** Builds data from expressions to plot directly. */
export function buildChartData(
  rows: DiceRow[],
  results: Record<string, RowComputation>,
): ChartPoint[] {
  const okRows = rows.filter((row) => results[row.id]?.status === "ok");
  if (okRows.length === 0) return [];

  const minimums = okRows.map(
    (row) => (results[row.id] as { minimum: number }).minimum,
  );
  const maximums = okRows.map(
    (row) => (results[row.id] as { maximum: number }).maximum,
  );
  const globalMin = Math.min(...minimums);
  const globalMax = Math.max(...maximums);

  const points: ChartPoint[] = [];
  for (let outcome = globalMin; outcome <= globalMax; outcome++) {
    const point: ChartPoint = { outcome };
    for (const row of rows) {
      const result = results[row.id];
      point[row.id] =
        result?.status === "ok" &&
        outcome >= result.minimum &&
        outcome <= result.maximum
          ? result.pmf[outcome - result.minimum]
          : null;
    }
    points.push(point);
  }
  return points;
}
