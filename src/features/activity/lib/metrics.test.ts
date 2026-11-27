import { describe, expect, it } from "vitest";
import type { UsageShare, UsageTotals } from "@/lib/api";
import { change, energyParts, joulesPerThousandTokens, niceMax, topShares } from "./metrics";

const share = (name: string, requests: number): UsageShare => ({
  name,
  requests,
  completionTokens: requests * 10,
  energyJoules: null,
});

describe("activity metrics", () => {
  it("compares with the previous period", () => {
    expect(change(150, 100)).toBeCloseTo(0.5);
    expect(change(50, 100)).toBeCloseTo(-0.5);
    expect(change(10, 0)).toBeNull();
    expect(change(null, 10)).toBeNull();
  });

  it("rounds the axis up to 1, 2 or 5 times a power of ten", () => {
    expect(niceMax(0)).toBe(1);
    expect(niceMax(7)).toBe(10);
    expect(niceMax(13)).toBe(20);
    expect(niceMax(420)).toBe(500);
    expect(niceMax(1000)).toBe(1000);
  });

  it("reads energy in mWh or Wh", () => {
    expect(energyParts(36)).toEqual({ value: 10, unit: "mWh" });
    expect(energyParts(7200)).toEqual({ value: 2, unit: "Wh" });
  });

  it("gives energy per thousand tokens", () => {
    const totals = { completionTokens: 500, energyJoules: 100 } as UsageTotals;
    expect(joulesPerThousandTokens(totals)).toBe(200);
    expect(joulesPerThousandTokens({ ...totals, energyJoules: null })).toBeNull();
  });

  it("folds the smallest shares into others", () => {
    const shares = ["a", "b", "c", "d"].map((name, index) => share(name, index + 1));
    const { top, others } = topShares(shares, "requests", 2);
    expect(top.map((s) => s.name)).toEqual(["d", "c"]);
    expect(others).toEqual({ count: 2, value: 3 });
    expect(topShares(shares, "requests", 10).others).toBeNull();
  });
});
