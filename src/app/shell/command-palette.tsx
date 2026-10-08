import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import {
  Copy,
  KeyRound,
  MessageSquarePlus,
  Monitor,
  Moon,
  Play,
  RefreshCw,
  Square,
  Sun,
} from "lucide-react";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { useChat } from "@/features/chat/store";
import { modelsQuery } from "@/features/library/api";
import { gatewayQuery, useStartServer, useStopServer } from "@/features/server/api";
import { API_BASE_URL } from "@/lib/api";
import { useTheme } from "@/stores/theme";
import { useUi } from "@/stores/ui";
import { allNavigation } from "../navigation";

export function CommandPalette() {
  const open = useUi((state) => state.paletteOpen);
  const setOpen = useUi((state) => state.setPaletteOpen);
  const setTheme = useTheme((state) => state.setTheme);
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { data: gateway } = useQuery(gatewayQuery);
  const startServer = useStartServer();
  const stopServer = useStopServer();
  const baseUrl = gateway?.baseUrl ?? API_BASE_URL;
  const serverRunning = gateway?.server.state === "ready" || gateway?.server.state === "starting";

  const run = (action: () => void) => () => {
    setOpen(false);
    action();
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent>
        <DialogTitle className="sr-only">Command palette</DialogTitle>
        <DialogDescription className="sr-only">
          Search pages and actions, then press Enter.
        </DialogDescription>
        <Command loop>
          <CommandInput placeholder="Search or run a command…" />
          <CommandList>
            <CommandEmpty>No results.</CommandEmpty>
            <CommandGroup heading="Go to">
              {allNavigation.map(({ to, label, icon: Icon }) => (
                <CommandItem key={to} onSelect={run(() => navigate({ to }))}>
                  <Icon />
                  {label}
                </CommandItem>
              ))}
            </CommandGroup>
            <CommandGroup heading="Actions">
              <CommandItem
                onSelect={run(() => {
                  useChat.setState({ activeId: null });
                  void navigate({ to: "/chat" });
                })}
              >
                <MessageSquarePlus />
                New chat
              </CommandItem>
              {serverRunning ? (
                <CommandItem onSelect={run(() => stopServer.mutate())}>
                  <Square />
                  Stop GenieX
                </CommandItem>
              ) : (
                <CommandItem onSelect={run(() => startServer.mutate())}>
                  <Play />
                  Start GenieX
                </CommandItem>
              )}
              <CommandItem onSelect={run(() => navigate({ to: "/server" }))}>
                <KeyRound />
                Create an API key
              </CommandItem>
              <CommandItem
                onSelect={run(() =>
                  queryClient.invalidateQueries({ queryKey: modelsQuery.queryKey }),
                )}
              >
                <RefreshCw />
                Refresh model library
              </CommandItem>
              <CommandItem onSelect={run(() => navigator.clipboard.writeText(baseUrl))}>
                <Copy />
                Copy API base URL
                <span className="ml-auto font-mono text-xs text-muted-foreground">{baseUrl}</span>
              </CommandItem>
            </CommandGroup>
            <CommandGroup heading="Theme">
              <CommandItem onSelect={run(() => setTheme("dark"))}>
                <Moon />
                Dark theme
              </CommandItem>
              <CommandItem onSelect={run(() => setTheme("light"))}>
                <Sun />
                Light theme
              </CommandItem>
              <CommandItem onSelect={run(() => setTheme("system"))}>
                <Monitor />
                Match system theme
              </CommandItem>
            </CommandGroup>
          </CommandList>
        </Command>
      </DialogContent>
    </Dialog>
  );
}
