import { HardDrive, MemoryStick } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import type { HardwareInfo } from "@/lib/api";
import { formatBytes } from "@/lib/format";

/** Memory and model-cache disk usage. */
export function StorageCard({ info }: { info: HardwareInfo | undefined }) {
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
