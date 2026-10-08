import {
  CircleAlert,
  CircleCheck,
  CirclePause,
  Download,
  FolderInput,
  RotateCw,
  X,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { useT } from "@/i18n";
import type { Job } from "@/lib/api";
import { useCancelJob, useDismissJob, useImportModel, usePullByName } from "../api";
import { jobTitle, percent, progressLabel } from "../lib/format";
import { messages } from "../messages";

export function JobRow({ job }: { job: Job }) {
  const t = useT(messages);
  const cancel = useCancelJob();
  const dismiss = useDismissJob();
  const retryPull = usePullByName();
  const retryImport = useImportModel();
  const title = jobTitle(job);
  const importing = job.kind.type === "import";

  // The new job replaces this one in the list.
  const retry = () => {
    const started =
      job.kind.type === "import"
        ? retryImport.mutateAsync({ name: job.kind.model, path: job.kind.path, modelType: null })
        : retryPull.pull(title);
    return started.then(() => dismiss.mutate(job.id));
  };

  return (
    <li className="flex flex-col gap-2 rounded-lg border bg-background/40 p-3">
      <div className="flex items-center gap-2">
        <StateIcon job={job} />
        <span className="min-w-0 flex-1 truncate font-mono text-[13px]">{title}</span>
        {job.state.state === "running" ? (
          <Button
            size="sm"
            variant="ghost"
            onClick={() => cancel.mutate(job.id)}
            disabled={cancel.isPending}
          >
            {t("cancel")}
          </Button>
        ) : (
          <Button
            size="icon"
            variant="ghost"
            className="size-7"
            onClick={() => dismiss.mutate(job.id)}
            aria-label={t("dismiss")}
          >
            <X />
          </Button>
        )}
      </div>

      {job.kind.type === "import" && (
        <p className="truncate font-mono text-[11px] text-muted-foreground" title={job.kind.path}>
          {t("importFrom", { path: job.kind.path })}
        </p>
      )}
      {job.state.state === "running" && (
        <>
          <Progress
            value={percent(job)}
            label={t(importing ? "importing" : "downloading", { model: title })}
          />
          <p className="text-xs text-muted-foreground tabular-nums">{progressLabel(job, t)}</p>
        </>
      )}
      {job.state.state === "succeeded" && (
        <p className="text-xs text-muted-foreground">{t(importing ? "imported" : "downloaded")}</p>
      )}
      {(job.state.state === "cancelled" || job.state.state === "failed") && (
        <div className="flex items-start gap-2">
          <p className="min-w-0 flex-1 text-xs break-words text-muted-foreground">
            {job.state.state === "cancelled"
              ? t(importing ? "importCancelled" : "cancelled")
              : job.state.message}
          </p>
          <Button
            size="sm"
            variant="secondary"
            onClick={retry}
            disabled={retryPull.isPending || retryImport.isPending}
          >
            <RotateCw />
            {job.state.state === "cancelled" && !importing ? t("resume") : t("retry")}
          </Button>
        </div>
      )}
    </li>
  );
}

function StateIcon({ job }: { job: Job }) {
  switch (job.state.state) {
    case "running":
      return job.kind.type === "import" ? (
        <FolderInput className="size-4 shrink-0 text-primary" />
      ) : (
        <Download className="size-4 shrink-0 text-primary" />
      );
    case "succeeded":
      return <CircleCheck className="size-4 shrink-0 text-success" />;
    case "cancelled":
      return <CirclePause className="size-4 shrink-0 text-muted-foreground" />;
    case "failed":
      return <CircleAlert className="size-4 shrink-0 text-destructive" />;
  }
}
