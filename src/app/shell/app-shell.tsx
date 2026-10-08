import { Outlet } from "@tanstack/react-router";
import { useEffect } from "react";
import { useUi } from "@/stores/ui";
import { CommandPalette } from "./command-palette";
import { Sidebar } from "./sidebar";
import { TaskDrawer } from "./task-drawer";
import { Topbar } from "./topbar";

export function AppShell() {
  useGlobalShortcuts();

  return (
    <div className="grid h-full grid-cols-[13.5rem_minmax(0,1fr)]">
      <Sidebar />
      <div className="flex min-h-0 flex-col">
        <Topbar />
        <main className="min-h-0 flex-1 overflow-y-auto">
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
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        const { paletteOpen, setPaletteOpen } = useUi.getState();
        setPaletteOpen(!paletteOpen);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);
}
