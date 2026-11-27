/** What the Activity page can chart, and how each figure reads. */

import type { UsagePoint, UsageShare, UsageTotals } from "@/lib/api";

/** What the chart and the breakdowns measure. */
export type Metric = "requests" | "tokens" | "energy";

/** A point's or share's value for `metric`. */
export function metricValue(item: UsagePoint | UsageShare, metric: Metric): number {
  if (metric === "requests") return item.requests;
  if (metric === "tokens") return item.completionTokens;
  return item.energyJoules ?? 0;
}

/**
 * Change from `previous` to `current`, as a fraction (0.25 is +25 %).
 * `null` when there's nothing to compare with.
 */
export function change(current: number | null, previous: number | null): number | null {
  if (current === null || previous === null || previous === 0) return null;
  return (current - previous) / previous;
}

/** Energy per thousand generated tokens, in joules. */
export function joulesPerThousandTokens(totals: UsageTotals): number | null {
  if (totals.energyJoules === null || totals.completionTokens === 0) return null;
  return (totals.energyJoules / totals.completionTokens) * 1000;
}

/** Joules in the unit that reads best: mWh below one watt-hour, then Wh. */
export function energyParts(joules: number): { value: number; unit: "mWh" | "Wh" } {
  const wattHours = joules / 3600;
  return wattHours < 1
    ? { value: wattHours * 1000, unit: "mWh" }
    : { value: wattHours, unit: "Wh" };
}

/** A round axis maximum at or above `max`: 1, 2 or 5 times a power of ten. */
export function niceMax(max: number): number {
  if (max <= 0) return 1;
  const power = 10 ** Math.floor(Math.log10(max));
  const step = [1, 2, 5, 10].find((multiple) => multiple * power >= max) ?? 10;
  return step * power;
}

/** The biggest shares, then the rest folded into one. */
export function topShares(
  shares: readonly UsageShare[],
  metric: Metric,
  shown = 6,
): { top: UsageShare[]; others: { count: number; value: number } | null } {
  const sorted = [...shares].sort((a, b) => metricValue(b, metric) - metricValue(a, metric));
  const top = sorted.slice(0, shown);
  const rest = sorted.slice(shown);
  if (rest.length === 0) return { top, others: null };
  return {
    top,
    others: {
      count: rest.length,
      value: rest.reduce((sum, share) => sum + metricValue(share, metric), 0),
    },
  };
}
