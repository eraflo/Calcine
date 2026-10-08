import { CircleAlert, CircleCheck, CirclePause, Download, RotateCw, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import type { Job } from "@/lib/api";
import { useCancelJob, useDismissJob, usePullByName } from "../api";
import { jobTitle, percent, progressLabel } from "../lib/format";

export function JobRow({ job }: { job: Job }) {
  const cancel = useCancelJob();
  const dismiss = useDismissJob();
  const retry = usePullByName();
  const title = jobTitle(job);

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
            Cancel
          </Button>
        ) : (
          <Button
            size="icon"
            variant="ghost"
            className="size-7"
            onClick={() => dismiss.mutate(job.id)}
            aria-label="Dismiss"
          >
            <X />
          </Button>
        )}
      </div>

      {job.state.state === "running" && (
        <>
          <Progress value={percent(job)} label={`Downloading ${title}`} />
          <p className="text-xs text-muted-foreground tabular-nums">{progressLabel(job)}</p>
        </>
      )}
      {job.state.state === "succeeded" && (
        <p className="text-xs text-muted-foreground">Downloaded to your library</p>
      )}
      {(job.state.state === "cancelled" || job.state.state === "failed") && (
        <div className="flex items-start gap-2">
          <p className="min-w-0 flex-1 text-xs break-words text-muted-foreground">
            {job.state.state === "cancelled"
              ? "Cancelled. What was downloaded is kept, so resuming picks up where it stopped."
              : job.state.message}
          </p>
          <Button
            size="sm"
            variant="secondary"
            // The new job replaces this one in the list.
            onClick={() => retry.pull(title).then(() => dismiss.mutate(job.id))}
            disabled={retry.isPending}
          >
            <RotateCw />
            {job.state.state === "cancelled" ? "Resume" : "Retry"}
          </Button>
        </div>
      )}
    </li>
  );
}

function StateIcon({ job }: { job: Job }) {
  switch (job.state.state) {
    case "running":
      return <Download className="size-4 shrink-0 text-primary" />;
    case "succeeded":
      return <CircleCheck className="size-4 shrink-0 text-success" />;
    case "cancelled":
      return <CirclePause className="size-4 shrink-0 text-muted-foreground" />;
    case "failed":
      return <CircleAlert className="size-4 shrink-0 text-destructive" />;
  }
}
