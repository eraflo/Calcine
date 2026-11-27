import { ArrowDownRight, ArrowUpRight, Minus } from "lucide-react";
import type { ReactNode } from "react";
import { Tooltip } from "@/components/ui/tooltip";
import { formatNumber } from "@/lib/format";
import { cn } from "@/lib/utils";

/**
 * One headline figure: its value, a detail line, and how it moved since the
 * previous period. Trends stay neutral in color: more requests isn't good
 * or bad in itself.
 */
export function StatTile({
  label,
  value,
  unit,
  detail,
  trend,
  trendLabel,
  hint,
}: {
  label: string;
  value: string;
  unit?: string;
  detail?: ReactNode;
  /** Change since the previous period (0.25 is +25 %), `null` when unknown. */
  trend: number | null;
  /** "vs previous 24 hours". */
  trendLabel: string;
  hint?: string;
}) {
  const heading = <span className="text-xs font-medium text-muted-foreground">{label}</span>;
  return (
    <div className="flex min-w-0 flex-col gap-1.5 rounded-lg border bg-card p-4">
      {hint ? (
        <Tooltip content={hint}>
          <button type="button" className="w-fit cursor-help text-left">
            {heading}
          </button>
        </Tooltip>
      ) : (
        heading
      )}
      <span className="flex items-baseline gap-1">
        <span className="text-2xl font-semibold tracking-tight tabular-nums">{value}</span>
        {unit && <span className="text-sm text-muted-foreground">{unit}</span>}
      </span>
      <span className="min-h-4 truncate text-[11px] text-muted-foreground">{detail}</span>
      <Trend value={trend} label={trendLabel} />
    </div>
  );
}

function Trend({ value, label }: { value: number | null; label: string }) {
  if (value === null) return <span className="min-h-4" />;
  const flat = Math.abs(value) < 0.005;
  const Icon = flat ? Minus : value > 0 ? ArrowUpRight : ArrowDownRight;
  const percent = `${value > 0 ? "+" : ""}${formatNumber(value * 100, Math.abs(value) < 0.1 ? 1 : 0)} %`;
  return (
    <span className="flex min-w-0 items-center gap-1 text-[11px] text-muted-foreground">
      <span className={cn("flex items-center gap-0.5 font-medium text-foreground/80 tabular-nums")}>
        <Icon className="size-3" />
        {percent}
      </span>
      <span className="truncate">{label}</span>
    </span>
  );
}
