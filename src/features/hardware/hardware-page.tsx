import { useQuery } from "@tanstack/react-query";
import type { LucideIcon } from "lucide-react";
import { Cpu, Gpu, HardDrive, MemoryStick, Microchip } from "lucide-react";
import { ErrorState } from "@/components/calcine/error-state";
import { Page } from "@/components/calcine/page";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { Skeleton } from "@/components/ui/skeleton";
import type { ComputeUnit, HardwareInfo } from "@/lib/api";
import { formatBytes } from "@/lib/format";
import { cn } from "@/lib/utils";
import { chipsetQuery, hardwareQuery, runtimeQuery } from "./api";

export function HardwarePage() {
  const chipset = useQuery(chipsetQuery);
  const hardware = useQuery(hardwareQuery);

  return (
    <Page
      title="Hardware"
      description={chipset.data ?? (chipset.isPending ? "Detecting your device…" : "Your device")}
    >
      {hardware.isError ? (
        <ErrorState error={hardware.error} onRetry={() => hardware.refetch()} />
      ) : (
        <>
          <ComputeUnits info={hardware.data} />
          <Storage info={hardware.data} />
        </>
      )}
      <RuntimeCard />
    </Page>
  );
}

const UNIT_STYLE: Record<
  Exclude<ComputeUnit, "hybrid">,
  { label: string; icon: LucideIcon; bar: string }
> = {
  npu: { label: "NPU", icon: Microchip, bar: "bg-npu" },
  gpu: { label: "GPU", icon: Gpu, bar: "bg-gpu" },
  cpu: { label: "CPU", icon: Cpu, bar: "bg-cpu" },
};

function ComputeUnits({ info }: { info: HardwareInfo | undefined }) {
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

function Storage({ info }: { info: HardwareInfo | undefined }) {
  const memory = info?.memory;
  const disk = info?.modelsDisk;
  const rows = [
    {
      icon: MemoryStick,
      label: "Memory",
      used: memory ? memory.totalBytes - memory.availableBytes : undefined,
      total: memory?.totalBytes,
      hint: memory && `${formatBytes(memory.availableBytes)} free`,
    },
    {
      icon: HardDrive,
      label: "Model storage",
      used: disk ? disk.totalBytes - disk.availableBytes : undefined,
      total: disk?.totalBytes,
      hint: disk && `${formatBytes(disk.availableBytes)} free · ${disk.path}`,
    },
  ];

  return (
    <Card>
      <CardContent className="flex flex-col gap-4">
        {rows.map(({ icon: Icon, label, used, total, hint }) => (
          <div key={label} className="flex flex-col gap-1.5">
            <div className="flex items-center gap-2 text-sm">
              <Icon className="size-4 text-muted-foreground" />
              <span>{label}</span>
              <span className="ml-auto text-xs text-muted-foreground tabular-nums">
                {total !== undefined ? formatBytes(total) : "—"}
              </span>
            </div>
            <Progress
              value={used !== undefined && total ? Math.round((used / total) * 100) : null}
              label={`${label} used`}
              barClassName="bg-info"
            />
            <p className="truncate text-xs text-muted-foreground">{hint ?? " "}</p>
          </div>
        ))}
      </CardContent>
    </Card>
  );
}

function RuntimeCard() {
  const { data, error, isPending, refetch } = useQuery(runtimeQuery);

  if (error) return <ErrorState error={error} onRetry={() => refetch()} />;

  const rows: [string, string | null | undefined][] = [
    ["GenieX CLI", data?.cliVersion],
    ["QAIRT runtime", data?.qairtVersion],
    ["llama.cpp build", data?.llamaCppHash],
    ["Executable", data?.binaryPath],
  ];

  return (
    <Card>
      <CardHeader>
        <CardTitle>GenieX runtime</CardTitle>
        <CardDescription>Versions reported by `geniex version`</CardDescription>
      </CardHeader>
      <CardContent>
        <dl className="grid grid-cols-[10rem_1fr] gap-x-4 gap-y-2 text-sm">
          {rows.map(([label, value]) => (
            <div key={label} className="contents">
              <dt className="text-muted-foreground">{label}</dt>
              <dd className="truncate font-mono text-[13px]">
                {isPending ? <Skeleton className="h-4 w-32" /> : (value ?? "—")}
              </dd>
            </div>
          ))}
        </dl>
      </CardContent>
    </Card>
  );
}
