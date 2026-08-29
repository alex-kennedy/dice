const CHART_COLORS = [
  "oklch(0.708 0.176 30)",
  "oklch(0.85 0.1 85)",
  "oklch(0.708 0.176 165)",
  "oklch(0.708 0.176 230)",
  "oklch(0.708 0.176 300)",
];

export function chartColor(index: number): string {
  return CHART_COLORS[index % CHART_COLORS.length];
}
