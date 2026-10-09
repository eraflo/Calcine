import { useQuery } from "@tanstack/react-query";
import { Download } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { recommendedRuntimeQuery, runtimeQuery, useInstallRuntime } from "@/features/hardware/api";
import { jobsQuery } from "@/features/tasks/api";
import { progressLabel } from "@/features/tasks/lib/format";
import { messages as taskMessages } from "@/features/tasks/messages";
import { useT } from "@/i18n";
import { CalcineError } from "@/lib/api";
import { formatBytes } from "@/lib/format";
import { messages } from "../messages";
import { Step } from "./step";

/**
 * GenieX isn't in Calcine's installer (it contains Qualcomm's proprietary
 * runtimes): the first launch downloads the official installer, checks it
 * against the SHA-256 shipped in Calcine, and runs it.
 */
export function RuntimeStep() {
  const t = useT(messages);
  const tt = useT(taskMessages);
  const runtime = useQuery(runtimeQuery);
  const recommended = useQuery(recommendedRuntimeQuery);
  const install = useInstallRuntime();
  const { data: jobs = [] } = useQuery(jobsQuery);
  const installing = jobs.find(
    (job) => job.kind.type === "install_runtime" && job.state.state === "running",
  );
  const failed = jobs.find(
    (job) => job.kind.type === "install_runtime" && job.state.state === "failed",
  );

  if (runtime.isPending) {
    return (
      <Step index={1} title={t("runtimeTitle")} status="loading">
        {t("runtimeLooking")}
      </Step>
    );
  }
  if (installing) {
    const progress = installing.progress;
    return (
      <Step index={1} title={t("runtimeTitle")} status="loading">
        <div className="mt-1 flex flex-col gap-1.5">
          <Progress
            value={
              progress?.totalBytes && progress.phase === "downloading"
                ? (progress.doneBytes / progress.totalBytes) * 100
                : null
            }
            label={t("runtimeTitle")}
          />
          <span className="text-xs tabular-nums">{progressLabel(installing, tt)}</span>
        </div>
      </Step>
    );
  }
  const missing =
    runtime.isError &&
    runtime.error instanceof CalcineError &&
    runtime.error.kind === "runtime_not_found";
  if (runtime.isError) {
    const release = recommended.data;
    return (
      <Step index={1} title={t("runtimeTitle")} status="problem">
        <div className="flex flex-col gap-3">
          <p>{missing ? t("runtimeMissing") : runtime.error.message}</p>
          {release && (
            <div className="flex flex-wrap items-center gap-3">
              <Button
                variant="default"
                onClick={() => install.mutate({ source: "release", release })}
                disabled={install.isPending}
              >
                <Download />
                {t("installRuntime", {
                  version: release.version,
                  size: formatBytes(release.installer.size),
                })}
              </Button>
              <span className="text-xs">{t("installRuntimeHint")}</span>
            </div>
          )}
          {(install.isError || failed?.state.state === "failed") && (
            <p className="text-xs text-destructive">
              {install.error?.message ??
                (failed?.state.state === "failed" ? failed.state.message : null)}
            </p>
          )}
        </div>
      </Step>
    );
  }
  return (
    <Step index={1} title={t("runtimeTitle")} status="done">
      {t("runtimeReady", {
        cli: runtime.data.cliVersion,
        qairt: runtime.data.qairtVersion ?? "—",
      })}
    </Step>
  );
}
