import { describe, expect, it } from "vitest";
import { formatBytes, totalBytes } from "./format";

describe("formatBytes", () => {
  it("matches the GenieX CLI for real model sizes", () => {
    expect(formatBytes(3_182_356_037)).toBe("3.0 GiB");
    expect(formatBytes(1_288_490_188)).toBe("1.2 GiB");
  });

  it("drops decimals at 10 and above", () => {
    expect(formatBytes(16 * 1024 ** 3)).toBe("16 GiB");
  });

  it("keeps small values in bytes", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(1023)).toBe("1023 B");
  });

  it("rejects invalid input", () => {
    expect(formatBytes(-1)).toBe("—");
    expect(formatBytes(Number.NaN)).toBe("—");
  });
});

describe("totalBytes", () => {
  it("sums sizes", () => {
    expect(totalBytes([1, 2, 3])).toBe(6);
    expect(totalBytes([])).toBe(0);
  });
});
