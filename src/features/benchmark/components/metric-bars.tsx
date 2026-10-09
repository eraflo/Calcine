import { Trophy } from "lucide-react";
import { useT } from "@/i18n";
import { common } from "@/i18n/common";
import type { BenchResult, ComputeUnit } from "@/lib/api";
import { formatNumber } from "@/lib/format";
import { cn } from "@/lib/utils";
import { best, type Metric, median } from "../lib/results";
import { messages } from "../messages";

/** Compute-unit colors, as everywhere else in Calcine. */
export const UNIT_COLOR: Record<ComputeUnit, { bar: string; text: string }> = {
  npu: { bar: "bg-npu", text: "text-npu" },
  gpu: { bar: "bg-gpu", text: "text-gpu" },
  cpu: { bar: "bg-cpu", text: "text-cpu" },
  hybrid: { bar: "bg-info", text: "text-info" },
};

/** One metric for each unit of a session, as bars scaled to the largest. */
export function MetricBars({
  results,
  metric,
}: {
  results: readonly BenchResult[];
  metric: Metric;
}) {
  const t = useT(messages);
  const tc = useT(common);
  const winner = results.length > 1 ? best(results, metric) : null;
  const largest = Math.max(1, ...results.map((result) => median(result, metric) ?? 0));
  const title = { decodeTps: t("decode"), prefillTps: t("prefill"), ttftMs: t("ttft") }[metric];
  const hint = { decodeTps: t("decodeHint"), prefillTps: t("prefillHint"), ttftMs: t("ttftHint") }[
    metric
  ];
  const format = (value: number) =>
    metric === "ttftMs"
      ? t("ms", { value: formatNumber(value, value < 100 ? 1 : 0) })
      : t("tps", { value: formatNumber(value, value < 100 ? 1 : 0) });

  return (
    <div className="flex flex-col gap-2.5">
      <div className="flex items-baseline justify-between gap-2">
        <span className="text-sm font-medium">{title}</span>
        <span className="text-[11px] text-muted-foreground">{hint}</span>
      </div>
      {results.map((result) => {
        const value = median(result, metric);
        const spread = result.measure?.[metric].stdev ?? null;
        const color = UNIT_COLOR[result.unit];
        return (
          <div key={result.id} className="grid grid-cols-[4.5rem_1fr_9rem] items-center gap-3">
            <span className={cn("text-xs font-medium", color.text)}>{tc(result.unit)}</span>
            <div className="h-2.5 overflow-hidden rounded-full bg-muted">
              {value !== null && (
                <div
                  className={cn("h-full rounded-full transition-[width] duration-500", color.bar)}
                  style={{ width: `${Math.max(2, (value / largest) * 100)}%` }}
                />
              )}
            </div>
            <span className="flex items-center justify-end gap-1.5 text-right text-xs tabular-nums">
              {value === null ? (
                <span className="text-destructive">{t("failed")}</span>
              ) : (
                <>
                  {winner?.id === result.id && (
                    <Trophy className="size-3 text-warning" aria-label={t("fastest")} />
                  )}
                  <span className="font-medium">{format(value)}</span>
                  {spread !== null && spread > 0 && (
                    <span className="text-muted-foreground">
                      {t("spread", { value: formatNumber(spread, spread < 10 ? 1 : 0) })}
                    </span>
                  )}
                </>
              )}
            </span>
          </div>
        );
      })}
    </div>
  );
}
