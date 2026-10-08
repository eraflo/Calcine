import { useQuery } from "@tanstack/react-query";
import { ListChecks } from "lucide-react";
import { EmptyState } from "@/components/calcine/feedback/empty-state";
import { Button } from "@/components/ui/button";
import { Sheet, SheetContent, SheetDescription, SheetTitle } from "@/components/ui/sheet";
import { useUi } from "@/stores/ui";
import { jobsQuery, useDismissJob } from "../api";
import { isRunning } from "../lib/format";
import { JobRow } from "./job-row";

/** Long-running jobs: downloads now, updates and benchmarks later. */
export function TaskDrawer() {
  const open = useUi((state) => state.tasksOpen);
  const setOpen = useUi((state) => state.setTasksOpen);
  const { data: jobs = [] } = useQuery(jobsQuery);
  const dismiss = useDismissJob();
  const finished = jobs.filter((job) => !isRunning(job));

  return (
    <Sheet open={open} onOpenChange={setOpen}>
      <SheetContent>
        <div className="flex items-start gap-3 border-b px-4 py-3 pr-12">
          <div className="min-w-0 flex-1">
            <SheetTitle className="text-sm font-medium">Tasks</SheetTitle>
            <SheetDescription className="text-xs text-muted-foreground">
              Downloads keep going when this panel or the window is closed.
            </SheetDescription>
          </div>
          {finished.length > 0 && (
            <Button
              size="sm"
              variant="ghost"
              onClick={() => {
                for (const job of finished) dismiss.mutate(job.id);
              }}
            >
              Clear finished
            </Button>
          )}
        </div>
        <div className="min-h-0 flex-1 overflow-y-auto overscroll-contain p-4">
          {jobs.length === 0 ? (
            <EmptyState icon={ListChecks} title="Nothing running">
              Model downloads show their progress here.
            </EmptyState>
          ) : (
            <ul className="flex flex-col gap-2">
              {jobs.map((job) => (
                <JobRow key={job.id} job={job} />
              ))}
            </ul>
          )}
        </div>
      </SheetContent>
    </Sheet>
  );
}
