/** Represents a dice expression in the UI. */
export type DiceRow = { id: string; expression: string };

/** Creates a new dice row for an expression. */
export function createRow(expression = ""): DiceRow {
  return { id: crypto.randomUUID(), expression };
}

/** Encodes dice expression states into a URL parameter. */
export function encodeRowsParam(rows: DiceRow[]): string {
  return rows.map((row) => encodeURIComponent(row.expression)).join(",");
}

/** Parses the URL encoding of dice expressions. */
export function parseRowsParam(param: string | null): DiceRow[] {
  if (!param) {
    return [createRow("2d6 + 3")];
  }
  return param
    .split(",")
    .map((expression) => createRow(decodeURIComponent(expression)));
}
