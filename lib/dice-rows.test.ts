import { describe, expect, it } from "vitest";

import { createRow, encodeRowsParam, parseRowsParam } from "./dice-rows";

describe("encode and parse", () => {
  it("round-trips a single expression", () => {
    const rows = [createRow("2d6 + 3")];
    const decoded = parseRowsParam(encodeRowsParam(rows));
    expect(decoded.map((row) => row.expression)).toEqual(["2d6 + 3"]);
  });

  it("round trips complex expressions", () => {
    const rows = [createRow("2(d6 + 1)"), createRow("d6*3")];
    const decoded = parseRowsParam(encodeRowsParam(rows));
    expect(decoded.map((row) => row.expression)).toEqual(["2(d6 + 1)", "d6*3"]);
  });

  it("round trips with a blank row", () => {
    const rows = [createRow("2d6"), createRow(""), createRow("d20")];
    const decoded = parseRowsParam(encodeRowsParam(rows));
    expect(decoded.map((row) => row.expression)).toEqual(["2d6", "", "d20"]);
  });

  it("defaults to a single 2d6 + 3 row when the param is missing", () => {
    expect(parseRowsParam(null).map((row) => row.expression)).toEqual([
      "2d6 + 3",
    ]);
  });
});
