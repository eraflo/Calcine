import { describe, expect, it } from "vitest";
import { formatBytes, formatRelative, totalBytes } from "./format";

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

describe("formatRelative", () => {
  const now = 1_700_000_000_000;
  it("scales from seconds to days", () => {
    expect(formatRelative(now - 10_000, now)).toBe("just now");
    expect(formatRelative(now - 5 * 60_000, now)).toBe("5 min ago");
    expect(formatRelative(now - 3 * 3_600_000, now)).toBe("3 h ago");
    expect(formatRelative(now - 26 * 3_600_000, now)).toBe("yesterday");
    expect(formatRelative(now - 3 * 86_400_000, now)).toBe("3 days ago");
  });
});
