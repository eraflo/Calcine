import { useMutation, useQuery } from "@tanstack/react-query";
import { FolderOpen, HardDrive, MemoryStick } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { useT } from "@/i18n";
import { common } from "@/i18n/common";
import { commands, type HardwareInfo, unwrap } from "@/lib/api";
import { formatBytes } from "@/lib/format";
import { usageQuery } from "../api";
import { comfortableModelBytes } from "../lib/fit";
import { messages } from "../messages";

/** Memory (live) and model-cache disk usage. */
export function StorageCard({ info }: { info: HardwareInfo | undefined }) {
  const t = useT(messages);
  const tc = useT(common);
  const { data: usage } = useQuery(usageQuery);
  const openFolder = useMutation({ mutationFn: () => unwrap(commands.openModelsFolder) });
  const memory = usage?.memory ?? info?.memory;
  const disk = info?.modelsDisk;

  const rows = [
    {
      icon: MemoryStick,
      label: t("memory"),
      used: memory ? memory.totalBytes - memory.availableBytes : undefined,
      total: memory?.totalBytes,
      hint:
        memory &&
        `${tc("free", { size: formatBytes(memory.availableBytes) })} · ${t("fitsHint", {
          size: formatBytes(comfortableModelBytes(memory)),
        })}`,
      action: null,
    },
    {
      icon: HardDrive,
      label: t("modelStorage"),
      used: disk ? disk.totalBytes - disk.availableBytes : undefined,
      total: disk?.totalBytes,
      hint: disk && `${tc("free", { size: formatBytes(disk.availableBytes) })} · ${disk.path}`,
      action: disk && (
        <Button
          size="sm"
          variant="ghost"
          className="h-6 px-2 text-xs"
          onClick={() => openFolder.mutate()}
        >
          <FolderOpen />
          {t("openFolder")}
        </Button>
      ),
    },
  ];

  return (
    <Card>
      <CardContent className="flex flex-col gap-4">
        {rows.map(({ icon: Icon, label, used, total, hint, action }) => (
          <div key={label} className="flex flex-col gap-1.5">
            <div className="flex items-center gap-2 text-sm">
              <Icon className="size-4 text-muted-foreground" />
              <span>{label}</span>
              {action}
              <span className="ml-auto text-xs text-muted-foreground tabular-nums">
                {total !== undefined ? formatBytes(total) : "—"}
              </span>
            </div>
            <Progress
              value={used !== undefined && total ? Math.round((used / total) * 100) : null}
              label={t("used", { label })}
              barClassName="bg-info"
            />
            <p className="truncate text-xs text-muted-foreground">{hint ?? " "}</p>
          </div>
        ))}
        {openFolder.isError && (
          <p className="text-xs text-destructive">{openFolder.error.message}</p>
        )}
      </CardContent>
    </Card>
  );
}
