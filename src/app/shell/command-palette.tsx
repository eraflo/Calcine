import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import {
  Copy,
  FolderInput,
  KeyRound,
  Languages,
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
import { useLanguage, useT } from "@/i18n";
import { API_BASE_URL } from "@/lib/api";
import { useTheme } from "@/stores/theme";
import { useUi } from "@/stores/ui";
import { messages } from "../messages";
import { allNavigation } from "../navigation";

export function CommandPalette() {
  const t = useT(messages);
  const open = useUi((state) => state.paletteOpen);
  const setOpen = useUi((state) => state.setPaletteOpen);
  const setTheme = useTheme((state) => state.setTheme);
  const setLanguage = useLanguage((state) => state.setPreference);
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
        <DialogTitle className="sr-only">{t("palette")}</DialogTitle>
        <DialogDescription className="sr-only">{t("paletteHint")}</DialogDescription>
        <Command loop>
          <CommandInput placeholder={t("search")} />
          <CommandList>
            <CommandEmpty>{t("noResults")}</CommandEmpty>
            <CommandGroup heading={t("goTo")}>
              {allNavigation.map(({ to, label, icon: Icon }) => (
                <CommandItem key={to} onSelect={run(() => navigate({ to }))}>
                  <Icon />
                  {t(label)}
                </CommandItem>
              ))}
            </CommandGroup>
            <CommandGroup heading={t("actions")}>
              <CommandItem
                onSelect={run(() => {
                  useChat.setState({ activeId: null });
                  void navigate({ to: "/chat" });
                })}
              >
                <MessageSquarePlus />
                {t("newChat")}
              </CommandItem>
              {serverRunning ? (
                <CommandItem onSelect={run(() => stopServer.mutate())}>
                  <Square />
                  {t("stopGeniex")}
                </CommandItem>
              ) : (
                <CommandItem onSelect={run(() => startServer.mutate())}>
                  <Play />
                  {t("startGeniex")}
                </CommandItem>
              )}
              <CommandItem onSelect={run(() => navigate({ to: "/server" }))}>
                <KeyRound />
                {t("createKey")}
              </CommandItem>
              <CommandItem
                onSelect={run(() => {
                  useUi.getState().setImporting({ path: null });
                  void navigate({ to: "/library" });
                })}
              >
                <FolderInput />
                {t("importModel")}
              </CommandItem>
              <CommandItem
                onSelect={run(() =>
                  queryClient.invalidateQueries({ queryKey: modelsQuery.queryKey }),
                )}
              >
                <RefreshCw />
                {t("refreshLibrary")}
              </CommandItem>
              <CommandItem onSelect={run(() => navigator.clipboard.writeText(baseUrl))}>
                <Copy />
                {t("copyBaseUrl")}
                <span className="ml-auto font-mono text-xs text-muted-foreground">{baseUrl}</span>
              </CommandItem>
            </CommandGroup>
            <CommandGroup heading={t("theme")}>
              <CommandItem onSelect={run(() => setTheme("dark"))}>
                <Moon />
                {t("darkTheme")}
              </CommandItem>
              <CommandItem onSelect={run(() => setTheme("light"))}>
                <Sun />
                {t("lightTheme")}
              </CommandItem>
              <CommandItem onSelect={run(() => setTheme("system"))}>
                <Monitor />
                {t("systemTheme")}
              </CommandItem>
            </CommandGroup>
            <CommandGroup heading={t("language")}>
              <CommandItem value="language english" onSelect={run(() => setLanguage("en"))}>
                <Languages />
                {t("english")}
              </CommandItem>
              <CommandItem value="langue français" onSelect={run(() => setLanguage("fr"))}>
                <Languages />
                {t("french")}
              </CommandItem>
            </CommandGroup>
          </CommandList>
        </Command>
      </DialogContent>
    </Dialog>
  );
}
