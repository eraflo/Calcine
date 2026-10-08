import { useQuery } from "@tanstack/react-query";
import { Download, PackageCheck } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { progressLabel } from "@/features/tasks/lib/format";
import { messages as taskMessages } from "@/features/tasks/messages";
import { useT } from "@/i18n";
import { formatBytes } from "@/lib/format";
import { benchToolQuery, useInstallBenchTool, useRunningJob } from "../api";
import { messages } from "../messages";

/** Download `geniex-bench`, matching the installed GenieX. Hidden once ready. */
export function ToolCard() {
  const t = useT(messages);
  const tt = useT(taskMessages);
  const { data: tool, error } = useQuery(benchToolQuery);
  const install = useInstallBenchTool();
  const downloading = useRunningJob("install_bench");

  if (!tool && !error) return null;
  const ready = tool && tool.installed !== null && tool.installed === tool.wanted;
  if (ready) return null;

  const status = !tool?.wanted
    ? t("noGeniex")
    : tool.installed
      ? t("toolOutdated", { installed: tool.installed, version: tool.wanted })
      : t("toolMissing", { version: tool.wanted });

  return (
    <Card className="border-primary/40">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <PackageCheck className="size-4 text-primary" />
          {t("toolTitle")}
        </CardTitle>
        <CardDescription>{t("toolHint")}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        {downloading ? (
          <div className="flex flex-col gap-1.5">
            <Progress
              value={
                downloading.progress?.totalBytes
                  ? (downloading.progress.doneBytes / downloading.progress.totalBytes) * 100
                  : null
              }
              label={t("toolTitle")}
            />
            <p className="text-xs text-muted-foreground tabular-nums">
              {progressLabel(downloading, tt)}
            </p>
          </div>
        ) : (
          <div className="flex items-center justify-between gap-4">
            <p className="text-sm">{error ? error.message : status}</p>
            {tool?.wanted && (
              <Button
                variant="default"
                onClick={() => install.mutate()}
                disabled={install.isPending}
              >
                <Download />
                {tool.downloadBytes
                  ? t("downloadTool", { size: formatBytes(tool.downloadBytes) })
                  : t("downloadToolNoSize")}
              </Button>
            )}
          </div>
        )}
        {install.isError && <p className="text-xs text-destructive">{install.error.message}</p>}
      </CardContent>
    </Card>
  );
}
