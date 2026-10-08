import { Link } from "@tanstack/react-router";
import { ListChecks, Search } from "lucide-react";
import { StatusDot } from "@/components/calcine/status-dot";
import { Button } from "@/components/ui/button";
import { Kbd } from "@/components/ui/kbd";
import { Tooltip } from "@/components/ui/tooltip";
import { useUi } from "@/stores/ui";

export function Topbar() {
  const setPaletteOpen = useUi((state) => state.setPaletteOpen);
  const setTasksOpen = useUi((state) => state.setTasksOpen);

  return (
    <header className="flex h-12 shrink-0 items-center gap-3 border-b px-4">
      <button
        type="button"
        onClick={() => setPaletteOpen(true)}
        className="flex h-8 w-full max-w-md items-center gap-2 rounded-md border bg-card px-2.5 text-[13px] text-muted-foreground transition-colors hover:border-muted-foreground/40"
      >
        <Search className="size-4" />
        <span className="truncate">Search or run a command…</span>
        <Kbd className="ml-auto">Ctrl K</Kbd>
      </button>

      <div className="ml-auto flex items-center gap-1">
        <Tooltip content="Local API server">
          <Link
            to="/server"
            className="flex h-8 items-center gap-2 rounded-md px-2.5 text-xs text-muted-foreground hover:bg-accent hover:text-foreground"
          >
            <StatusDot tone="idle" />
            API off
            <span className="font-mono">:18181</span>
          </Link>
        </Tooltip>
        <Tooltip content="Tasks">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setTasksOpen(true)}
            aria-label="Open tasks"
          >
            <ListChecks />
          </Button>
        </Tooltip>
      </div>
    </header>
  );
}
