import type { LucideIcon } from "lucide-react";
import { Cpu, Gpu, Microchip } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import type { ComputeUnit, HardwareInfo } from "@/lib/api";
import { cn } from "@/lib/utils";

const UNIT_STYLE: Record<
  Exclude<ComputeUnit, "hybrid">,
  { label: string; icon: LucideIcon; bar: string }
> = {
  npu: { label: "NPU", icon: Microchip, bar: "bg-npu" },
  gpu: { label: "GPU", icon: Gpu, bar: "bg-gpu" },
  cpu: { label: "CPU", icon: Cpu, bar: "bg-cpu" },
};

/** NPU, GPU and CPU cards, each in its compute-unit color. */
export function ComputeUnits({ info }: { info: HardwareInfo | undefined }) {
  const npu = info?.accelerators.find((accelerator) => accelerator.unit === "npu");
  const gpu = info?.accelerators.find((accelerator) => accelerator.unit === "gpu");
  const units = [
    {
      unit: "npu" as const,
      name: npu?.name,
      detail: npu?.driverVersion && `Driver ${npu.driverVersion}`,
    },
    {
      unit: "gpu" as const,
      name: gpu?.name,
      detail: gpu?.driverVersion && `Driver ${gpu.driverVersion}`,
    },
    { unit: "cpu" as const, name: info?.cpu?.name, detail: info?.cpu && `${info.cpu.cores} cores` },
  ];

  return (
    <div className="grid grid-cols-1 gap-3 md:grid-cols-3">
      {units.map(({ unit, name, detail }) => {
        const style = UNIT_STYLE[unit];
        return (
          <Card key={unit} className="relative overflow-hidden">
            <span className={cn("absolute inset-x-0 top-0 h-0.5", style.bar)} />
            <CardHeader>
              <div className="flex items-center gap-2">
                <style.icon className="size-4 text-muted-foreground" />
                <CardTitle>{style.label}</CardTitle>
              </div>
            </CardHeader>
            <CardContent className="flex flex-col gap-1">
              {info ? (
                <>
                  <p className="text-sm leading-snug">{name ?? "Not detected"}</p>
                  {detail && <p className="text-xs text-muted-foreground">{detail}</p>}
                </>
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
