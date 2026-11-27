/** Dates on the chart's axis and in its tooltips. */

import type { UsagePeriod } from "@/lib/api";

/** The webview's offset from UTC, for day boundaries at local midnight. */
export function utcOffsetMinutes(date = new Date()): number {
  return -date.getTimezoneOffset();
}

/** Every how many points the axis names one. */
export function labelEvery(period: UsagePeriod): number {
  return period === "day" ? 6 : period === "week" ? 1 : 5;
}

/** A point's axis label: the hour, the weekday, or the day and month. */
export function axisLabel(startMs: number, period: UsagePeriod, locale: string): string {
  const date = new Date(startMs);
  if (period === "day") {
    return date.toLocaleTimeString(locale, { hour: "2-digit", minute: "2-digit" });
  }
  if (period === "week") return date.toLocaleDateString(locale, { weekday: "short" });
  return date.toLocaleDateString(locale, { day: "numeric", month: "short" });
}

/** What a point covers, for its tooltip: "14:00 – 15:00" or "Tue 12 Nov". */
export function pointLabel(
  startMs: number,
  stepMs: number,
  period: UsagePeriod,
  locale: string,
): string {
  const start = new Date(startMs);
  if (period === "day") {
    const time = (date: Date) =>
      date.toLocaleTimeString(locale, { hour: "2-digit", minute: "2-digit" });
    return `${time(start)} – ${time(new Date(startMs + stepMs))}`;
  }
  return start.toLocaleDateString(locale, { weekday: "short", day: "numeric", month: "short" });
}
