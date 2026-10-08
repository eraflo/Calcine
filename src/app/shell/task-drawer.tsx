import { ListChecks } from "lucide-react";
import { EmptyState } from "@/components/calcine/page";
import { Sheet, SheetContent, SheetDescription, SheetTitle } from "@/components/ui/sheet";
import { useUi } from "@/stores/ui";

/** Long-running jobs (downloads, updates, benchmarks) will be listed here. */
export function TaskDrawer() {
  const open = useUi((state) => state.tasksOpen);
  const setOpen = useUi((state) => state.setTasksOpen);

  return (
    <Sheet open={open} onOpenChange={setOpen}>
      <SheetContent>
        <div className="border-b px-4 py-3">
          <SheetTitle className="text-sm font-medium">Tasks</SheetTitle>
          <SheetDescription className="text-xs text-muted-foreground">
            Downloads, updates and benchmarks
          </SheetDescription>
        </div>
        <div className="p-4">
          <EmptyState icon={ListChecks} title="Nothing running">
            Model downloads and GenieX updates show their progress here.
          </EmptyState>
        </div>
      </SheetContent>
    </Sheet>
  );
}
