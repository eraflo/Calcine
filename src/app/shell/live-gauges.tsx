import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { Sparkline } from "@/components/calcine/charts/sparkline";
import { usageQuery } from "@/features/hardware/api";
import { type GaugeUnit, useUsageHistory } from "@/features/hardware/lib/usage-history";
import { useT } from "@/i18n";
import { common } from "@/i18n/common";
import { cn } from "@/lib/utils";
import { messages } from "../messages";

const UNITS: { unit: GaugeUnit; text: string }[] = [
  { unit: "npu", text: "text-npu" },
  { unit: "gpu", text: "text-gpu" },
  { unit: "cpu", text: "text-cpu" },
];

/** Recent samples shown in the sidebar: the last 20 seconds. */
const MINI_LENGTH = 20;

/** Compact NPU/GPU/CPU load, shown in the sidebar while a model is generating. */
export function LiveGauges() {
  const t = useT(messages);
  const tc = useT(common);
  useQuery(usageQuery);
  const history = useUsageHistory();
  const latest = history.latest;

  return (
    <Link
      to="/hardware"
      className="flex flex-col gap-1.5 rounded-md border bg-background/40 px-2.5 py-2 hover:border-muted-foreground/40"
      aria-label={t("liveLoad")}
    >
      {UNITS.map(({ unit, text }) => {
        const percent =
          unit === "npu"
            ? latest?.npuPercent
            : unit === "gpu"
              ? latest?.gpuPercent
              : latest?.cpuPercent;
        return (
          <div key={unit} className="flex items-center gap-2 text-[11px]">
            <span className="w-7 font-medium text-muted-foreground">{tc(unit)}</span>
            <Sparkline
              values={history[unit].slice(-MINI_LENGTH)}
              length={MINI_LENGTH}
              className={cn("h-4 flex-1", text)}
            />
            <span className={cn("w-8 text-right font-mono tabular-nums", text)}>
              {percent === null || percent === undefined ? "—" : `${Math.round(percent)}%`}
            </span>
          </div>
        );
      })}
    </Link>
  );
}
