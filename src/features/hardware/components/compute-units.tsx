import { useQuery } from "@tanstack/react-query";
import type { LucideIcon } from "lucide-react";
import { Cpu, Gpu, Microchip } from "lucide-react";
import { Sparkline } from "@/components/calcine/charts/sparkline";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { useT } from "@/i18n";
import { common } from "@/i18n/common";
import type { HardwareInfo } from "@/lib/api";
import { cn } from "@/lib/utils";
import { usageQuery } from "../api";
import { type GaugeUnit, HISTORY_LENGTH, useUsageHistory } from "../lib/usage-history";
import { messages } from "../messages";

export const UNIT_STYLE: Record<GaugeUnit, { icon: LucideIcon; bar: string; text: string }> = {
  npu: { icon: Microchip, bar: "bg-npu", text: "text-npu" },
  gpu: { icon: Gpu, bar: "bg-gpu", text: "text-gpu" },
  cpu: { icon: Cpu, bar: "bg-cpu", text: "text-cpu" },
};

/** NPU, GPU and CPU cards with live load, each in its compute-unit color. */
export function ComputeUnits({ info }: { info: HardwareInfo | undefined }) {
  const t = useT(messages);
  const tc = useT(common);
  // Keeps the history polling while this page is open.
  useQuery(usageQuery);
  const history = useUsageHistory();

  const npu = info?.accelerators.find((accelerator) => accelerator.unit === "npu");
  const gpu = info?.accelerators.find((accelerator) => accelerator.unit === "gpu");
  const units = [
    {
      unit: "npu" as const,
      name: npu?.name,
      detail: npu?.driverVersion && t("driver", { version: npu.driverVersion }),
      percent: history.latest?.npuPercent ?? null,
    },
    {
      unit: "gpu" as const,
      name: gpu?.name,
      detail: gpu?.driverVersion && t("driver", { version: gpu.driverVersion }),
      percent: history.latest?.gpuPercent ?? null,
    },
    {
      unit: "cpu" as const,
      name: info?.cpu?.name,
      detail: info?.cpu && t.plural("cores", info.cpu.cores),
      percent: history.latest?.cpuPercent ?? null,
    },
  ];

  return (
    <div className="grid grid-cols-1 gap-3 md:grid-cols-3">
      {units.map(({ unit, name, detail, percent }) => {
        const style = UNIT_STYLE[unit];
        const label = tc(unit);
        return (
          <Card key={unit} className="relative overflow-hidden">
            <span className={cn("absolute inset-x-0 top-0 h-0.5", style.bar)} />
            <CardHeader>
              <div className="flex items-center gap-2">
                <style.icon className="size-4 text-muted-foreground" />
                <CardTitle>{label}</CardTitle>
                <span
                  className={cn(
                    "ml-auto font-mono text-sm tabular-nums",
                    percent === null ? "text-muted-foreground" : style.text,
                  )}
                  aria-hidden
                >
                  {percent === null ? "—" : `${Math.round(percent)} %`}
                </span>
                {percent !== null && (
                  <meter
                    className="sr-only"
                    min={0}
                    max={100}
                    value={percent}
                    aria-label={t("load", { unit: label })}
                  />
                )}
              </div>
            </CardHeader>
            <CardContent className="flex flex-col gap-2">
              <Sparkline
                values={history[unit]}
                length={HISTORY_LENGTH}
                className={cn(style.text, percent === null && "opacity-30")}
              />
              {info ? (
                <div className="flex flex-col gap-0.5">
                  <p className="text-sm leading-snug">{name ?? tc("notDetected")}</p>
                  <p className="text-xs text-muted-foreground">
                    {percent === null && unit !== "cpu" ? t("loadUnavailable") : (detail ?? " ")}
                  </p>
                </div>
              ) : (
                <>
                  <Skeleton className="h-4 w-full" />
                  <Skeleton className="h-3 w-24" />
                </>
              )}
            </CardContent>
          </Card>
        );
      })}
    </div>
  );
}
