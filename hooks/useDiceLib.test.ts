import { renderHook, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { useDiceLib } from "./useDiceLib";

const { loadWasm } = vi.hoisted(() => ({ loadWasm: vi.fn() }));

vi.mock("@/lib/wasm", () => ({ loadWasm }));

describe("useDiceLib", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("starts in the loading state", () => {
    loadWasm.mockReturnValue(new Promise(() => {}));

    const { result } = renderHook(() => useDiceLib());

    expect(result.current).toEqual({ status: "loading" });
  });

  it("moves to ready once the library resolves", async () => {
    const lib = { parse: vi.fn() };
    loadWasm.mockResolvedValue(lib);

    const { result } = renderHook(() => useDiceLib());

    await waitFor(() => {
      expect(result.current).toEqual({ status: "ready", lib });
    });
  });

  it("moves to error if loading fails", async () => {
    const error = new Error("oh naur");
    loadWasm.mockRejectedValue(error);

    const { result } = renderHook(() => useDiceLib());

    await waitFor(() => {
      expect(result.current).toEqual({ status: "error", error });
    });
  });
});
