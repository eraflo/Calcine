import { useState } from "react";
import { useLanguage, useT } from "@/i18n";
import type { UsagePoint, UsageSummary } from "@/lib/api";
import { formatCount, formatNumber } from "@/lib/format";
import { cn } from "@/lib/utils";
import { energyParts, type Metric, niceMax, valueOf } from "../lib/metrics";
import { axisLabel, labelEvery, pointLabel } from "../lib/time";
import { messages } from "../messages";

/** A value of `metric`, with its unit for energy. */
export function useFormatMetric() {
  return (value: number, metric: Metric, unit?: "mWh" | "Wh") => {
    if (metric !== "energy") return formatCount(value);
    if (value === 0) return unit ? `0 ${unit}` : "0";
    const parts = unit
      ? { value: unit === "Wh" ? value / 3600 : value / 3.6, unit }
      : energyParts(value);
    return `${formatNumber(parts.value, parts.value < 10 ? 1 : 0)} ${parts.unit}`;
  };
}

/**
 * One column per hour or day, a single series: the period's `metric` over
 * time. Hover or focus a column for its figures.
 */
export function UsageChart({ summary, metric }: { summary: UsageSummary; metric: Metric }) {
  const t = useT(messages);
  const language = useLanguage((state) => state.language);
  const format = useFormatMetric();
  const [active, setActive] = useState<number | null>(null);
  const { series, period, stepMs } = summary;
  const { max, unit } = axis(Math.max(0, ...series.map((point) => valueOf(point, metric))), metric);
  const every = labelEvery(period);
  const hovered = active === null ? undefined : series[active];

  return (
    <div className="relative select-none">
      <div className="flex h-52 gap-2">
        {/* Recessive axis: the top, the middle and zero. */}
        <div className="relative w-12 shrink-0 text-right text-[10px] text-muted-foreground tabular-nums">
          {[max, max / 2, 0].map((tick, index) => (
            <span
              key={tick}
              className="absolute right-0 -translate-y-1/2"
              style={{ top: `${index * 50}%` }}
            >
              {format(tick, metric, unit)}
            </span>
          ))}
        </div>
        <div className="relative flex-1" onMouseLeave={() => setActive(null)}>
          {[0, 50, 100].map((top) => (
            <span
              key={top}
              className={cn(
                "pointer-events-none absolute inset-x-0 border-t",
                top === 100 ? "border-border" : "border-dashed border-border/50",
              )}
              style={{ top: `${top}%` }}
            />
          ))}
          <div className="absolute inset-0 flex items-end gap-0.5">
            {series.map((point, index) => (
              <Column
                key={point.startMs}
                share={valueOf(point, metric) / max}
                dimmed={active !== null && active !== index}
                label={`${pointLabel(point.startMs, stepMs, period, language)}: ${format(valueOf(point, metric), metric)}`}
                onActive={() => setActive(index)}
                onInactive={() => setActive((current) => (current === index ? null : current))}
              />
            ))}
          </div>
          {hovered && active !== null && (
            <ChartTooltip
              point={hovered}
              index={active}
              count={series.length}
              label={pointLabel(hovered.startMs, stepMs, period, language)}
            />
          )}
        </div>
      </div>
      <div className="mt-1.5 flex gap-2">
        <span className="w-12 shrink-0" />
        <div className="flex flex-1 gap-0.5 text-[10px] text-muted-foreground">
          {series.map((point, index) => (
            <span key={point.startMs} className="relative flex-1">
              {(series.length - 1 - index) % every === 0 && (
                <span className="absolute left-1/2 -translate-x-1/2 whitespace-nowrap">
                  {axisLabel(point.startMs, period, language)}
                </span>
              )}
            </span>
          ))}
        </div>
      </div>
      <table className="sr-only">
        <caption>
          {t("chartLabel", { metric: t(`metric_${metric}`), period: t(`period_${period}`) })}
        </caption>
        <tbody>
          {series.map((point) => (
            <tr key={point.startMs}>
              <th scope="row">{pointLabel(point.startMs, stepMs, period, language)}</th>
              <td>{format(valueOf(point, metric), metric)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

/** The axis maximum, round in the unit shown (energy in mWh or Wh). */
function axis(highest: number, metric: Metric): { max: number; unit?: "mWh" | "Wh" } {
  if (metric !== "energy") return { max: niceMax(highest) };
  const { value, unit } = energyParts(highest);
  const joulesPerUnit = unit === "Wh" ? 3600 : 3.6;
  return { max: niceMax(value) * joulesPerUnit, unit };
}

/** A column: the whole slot is the hover target, the bar grows from zero. */
function Column({
  share,
  dimmed,
  label,
  onActive,
  onInactive,
}: {
  /** Height, 0 to 1. */
  share: number;
  dimmed: boolean;
  label: string;
  onActive: () => void;
  onInactive: () => void;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      onMouseEnter={onActive}
      onFocus={onActive}
      onBlur={onInactive}
      className="group flex h-full flex-1 items-end justify-center rounded-sm outline-none focus-visible:ring-2 focus-visible:ring-ring"
    >
      {share > 0 && (
        <span
          className={cn(
            "block w-full max-w-6 rounded-t-[4px] bg-chart transition-opacity",
            dimmed && "opacity-45",
          )}
          // Tiny values stay visible.
          style={{ height: `max(${(share * 100).toFixed(2)}%, 2px)` }}
        />
      )}
    </button>
  );
}

function ChartTooltip({
  point,
  index,
  count,
  label,
}: {
  point: UsagePoint;
  index: number;
  count: number;
  label: string;
}) {
  const t = useT(messages);
  const format = useFormatMetric();
  // Inside the chart, over the column, kept within it near the edges.
  const left = ((index + 0.5) / count) * 100;
  const shift = left < 15 ? "0%" : left > 85 ? "-100%" : "-50%";
  return (
    <div
      className="pointer-events-none absolute top-1 z-10 flex min-w-36 flex-col gap-0.5 rounded-md border bg-popover px-2.5 py-1.5 text-xs text-popover-foreground shadow-md"
      style={{ left: `${left}%`, transform: `translateX(${shift})` }}
      role="status"
    >
      <span className="font-medium">{label}</span>
      <span className="text-muted-foreground tabular-nums">
        {t.plural("tooltipRequests", point.requests, { count: formatCount(point.requests) })}
      </span>
      <span className="text-muted-foreground tabular-nums">
        {t("tooltipTokens", { count: formatCount(point.completionTokens) })}
      </span>
      {point.energyJoules !== null && (
        <span className="text-muted-foreground tabular-nums">
          {format(point.energyJoules, "energy")}
        </span>
      )}
      {point.errors > 0 && (
        <span className="text-muted-foreground tabular-nums">
          {t.plural("tooltipErrors", point.errors, { count: formatCount(point.errors) })}
        </span>
      )}
    </div>
  );
}
