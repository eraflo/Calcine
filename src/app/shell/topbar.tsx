import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { ListChecks, Search } from "lucide-react";
import { StatusDot } from "@/components/calcine/feedback/status-dot";
import { Button } from "@/components/ui/button";
import { Kbd } from "@/components/ui/kbd";
import { Tooltip } from "@/components/ui/tooltip";
import { gatewayQuery } from "@/features/server/api";
import { describeStatus } from "@/features/server/lib/status";
import { messages as serverMessages } from "@/features/server/messages";
import { jobsQuery } from "@/features/tasks/api";
import { isRunning } from "@/features/tasks/lib/format";
import { useT } from "@/i18n";
import { useUi } from "@/stores/ui";
import { messages } from "../messages";

export function Topbar() {
  const t = useT(messages);
  const tServer = useT(serverMessages);
  const setPaletteOpen = useUi((state) => state.setPaletteOpen);
  const setTasksOpen = useUi((state) => state.setTasksOpen);
  const { data: jobs = [] } = useQuery(jobsQuery);
  const running = jobs.filter(isRunning).length;
  const { data: gateway } = useQuery(gatewayQuery);
  const api = describeStatus(gateway, tServer);
  const port = gateway?.baseUrl.match(/:(\d+)\//)?.[1] ?? "18181";

  return (
    <header className="flex h-12 shrink-0 items-center gap-3 border-b px-4">
      <button
        type="button"
        onClick={() => setPaletteOpen(true)}
        className="flex h-8 w-full max-w-md items-center gap-2 rounded-md border bg-card px-2.5 text-[13px] text-muted-foreground transition-colors hover:border-muted-foreground/40"
      >
        <Search className="size-4" />
        <span className="truncate">{t("search")}</span>
        <Kbd className="ml-auto">Ctrl K</Kbd>
      </button>

      <div className="ml-auto flex items-center gap-1">
        <Tooltip content={api.detail}>
          <Link
            to="/server"
            className="flex h-8 items-center gap-2 rounded-md px-2.5 text-xs text-muted-foreground hover:bg-accent hover:text-foreground"
          >
            <StatusDot tone={api.tone} />
            {api.label}
            <span className="font-mono">:{port}</span>
          </Link>
        </Tooltip>
        <Tooltip content={running ? t("tasksRunning", { count: running }) : t("tasks")}>
          <Button
            variant="ghost"
            size="icon"
            className="relative"
            onClick={() => setTasksOpen(true)}
            aria-label={running ? t("openTasksRunning", { count: running }) : t("openTasks")}
          >
            <ListChecks />
            {running > 0 && (
              <span className="absolute -top-0.5 -right-0.5 flex size-4 items-center justify-center rounded-full bg-primary text-[10px] font-semibold text-primary-foreground">
                {running}
              </span>
            )}
          </Button>
        </Tooltip>
      </div>
    </header>
  );
}
