import { useQuery } from "@tanstack/react-query";
import { ListChecks } from "lucide-react";
import { EmptyState } from "@/components/calcine/page";
import { Sheet, SheetContent, SheetDescription, SheetTitle } from "@/components/ui/sheet";
import { useUi } from "@/stores/ui";
import { jobsQuery } from "./api";
import { JobRow } from "./job-row";

/** Long-running jobs: downloads now, updates and benchmarks later. */
export function TaskDrawer() {
  const open = useUi((state) => state.tasksOpen);
  const setOpen = useUi((state) => state.setTasksOpen);
  const { data: jobs = [] } = useQuery(jobsQuery);

  return (
    <Sheet open={open} onOpenChange={setOpen}>
      <SheetContent>
        <div className="border-b px-4 py-3">
          <SheetTitle className="text-sm font-medium">Tasks</SheetTitle>
          <SheetDescription className="text-xs text-muted-foreground">
            Downloads keep going when this panel or the window is closed.
          </SheetDescription>
        </div>
        <div className="min-h-0 flex-1 overflow-y-auto p-4">
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
