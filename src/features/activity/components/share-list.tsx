import { useT } from "@/i18n";
import type { UsageShare } from "@/lib/api";
import { cn } from "@/lib/utils";
import { type Metric, topShares, valueOf } from "../lib/metrics";
import { messages } from "../messages";
import { useFormatMetric } from "./usage-chart";

/**
 * Ranked bars, largest first, one hue on a shared baseline: who used the
 * models most. The smallest fold into "N others".
 */
export function ShareList({
  shares,
  metric,
  monospace = false,
  label = (name) => name,
}: {
  shares: readonly UsageShare[];
  metric: Metric;
  /** Model names read better in monospace. */
  monospace?: boolean;
  /** How to show a name (e.g. translated). */
  label?: (name: string) => string;
}) {
  const t = useT(messages);
  const format = useFormatMetric();
  const { top, others } = topShares(shares, metric);
  const max = Math.max(1, ...top.map((share) => valueOf(share, metric)));
  const total = shares.reduce((sum, share) => sum + valueOf(share, metric), 0);

  const row = (key: string, name: string, value: number, muted = false) => (
    <li key={key} className="grid grid-cols-[minmax(0,12rem)_1fr_auto] items-center gap-3 py-1.5">
      <span
        className={cn(
          "truncate text-xs",
          monospace && !muted && "font-mono",
          muted && "text-muted-foreground",
        )}
        title={name}
      >
        {name}
      </span>
      <span className="h-2 overflow-hidden rounded-full bg-muted">
        <span
          className={cn("block h-full rounded-full", muted ? "bg-muted-foreground/40" : "bg-chart")}
          style={{ width: `${Math.max(2, (value / max) * 100)}%` }}
        />
      </span>
      <span className="w-24 text-right text-xs tabular-nums">
        {format(value, metric)}
        {total > 0 && (
          <span className="ml-1.5 text-muted-foreground">
            {Math.round((value / total) * 100)} %
          </span>
        )}
      </span>
    </li>
  );

  return (
    <ul className="flex flex-col">
      {top.map((share) => row(share.name, label(share.name), valueOf(share, metric)))}
      {others && row("others", t.plural("others", others.count), others.value, true)}
    </ul>
  );
}
