import { Outlet } from "@tanstack/react-router";
import { useEffect } from "react";
import { useGatewayEvents } from "@/features/server/api";
import { useJobEvents } from "@/features/tasks/api";
import { TaskDrawer } from "@/features/tasks/components/task-drawer";
import { useSyncLanguage } from "@/i18n/sync";
import { cn } from "@/lib/utils";
import { useLayout } from "@/stores/layout";
import { useUi } from "@/stores/ui";
import { CommandPalette } from "./command-palette";
import { Sidebar } from "./sidebar";
import { Topbar } from "./topbar";

export function AppShell() {
  useGlobalShortcuts();
  useSyncLanguage();
  useJobEvents();
  useGatewayEvents();
  const collapsed = useLayout((state) => state.sidebarCollapsed);

  return (
    <div
      className={cn(
        "grid h-full",
        collapsed ? "grid-cols-[3.25rem_minmax(0,1fr)]" : "grid-cols-[13.5rem_minmax(0,1fr)]",
      )}
    >
      <Sidebar />
      <div className="flex min-h-0 flex-col">
        <Topbar />
        <main className="relative min-h-0 flex-1 overflow-y-auto overscroll-contain">
          <Outlet />
        </main>
      </div>
      <CommandPalette />
      <TaskDrawer />
    </div>
  );
}

function useGlobalShortcuts() {
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (!(event.ctrlKey || event.metaKey) || event.shiftKey || event.altKey) return;
      const key = event.key.toLowerCase();
      if (key === "k") {
        event.preventDefault();
        const { paletteOpen, setPaletteOpen } = useUi.getState();
        setPaletteOpen(!paletteOpen);
      } else if (key === "b") {
        event.preventDefault();
        useLayout.getState().toggleSidebar();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);
}
