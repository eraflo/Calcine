import { useQuery } from "@tanstack/react-query";
import type { LucideIcon } from "lucide-react";
import { Cpu, Gpu, Microchip } from "lucide-react";
import { ErrorState } from "@/components/calcine/error-state";
import { Page } from "@/components/calcine/page";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";
import { runtimeQuery } from "./api";

const UNITS: { name: string; chip: string; icon: LucideIcon; color: string }[] = [
  { name: "NPU", chip: "Qualcomm Hexagon", icon: Microchip, color: "bg-npu" },
  { name: "GPU", chip: "Qualcomm Adreno", icon: Gpu, color: "bg-gpu" },
  { name: "CPU", chip: "Qualcomm Oryon", icon: Cpu, color: "bg-cpu" },
];

export function HardwarePage() {
  return (
    <Page title="Hardware" description="Compute units and the GenieX runtime on this device">
      <div className="grid grid-cols-1 gap-3 md:grid-cols-3">
        {UNITS.map((unit) => (
          <Card key={unit.name} className="relative overflow-hidden">
            <span className={cn("absolute inset-x-0 top-0 h-0.5", unit.color)} />
            <CardHeader>
              <div className="flex items-center gap-2">
                <unit.icon className="size-4 text-muted-foreground" />
                <CardTitle>{unit.name}</CardTitle>
              </div>
              <CardDescription>{unit.chip}</CardDescription>
            </CardHeader>
            <CardContent className="text-xs text-muted-foreground">
              Live usage is coming soon.
            </CardContent>
          </Card>
        ))}
      </div>
      <RuntimeCard />
    </Page>
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
