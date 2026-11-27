import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { defaultFilter } from "cmdk";
import {
  Copy,
  FolderInput,
  KeyRound,
  Languages,
  MessageSquarePlus,
  MessagesSquare,
  Monitor,
  Moon,
  Play,
  RefreshCw,
  Square,
  Sun,
} from "lucide-react";
import { useState } from "react";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { searchConversations } from "@/features/chat/lib/search";
import { useChat } from "@/features/chat/store";
import { modelsQuery } from "@/features/library/api";
import { gatewayQuery, useStartServer, useStopServer } from "@/features/server/api";
import { useLanguage, useT } from "@/i18n";
import { API_BASE_URL } from "@/lib/api";
import { useTheme } from "@/stores/theme";
import { useUi } from "@/stores/ui";
import { messages } from "../messages";
import { allNavigation } from "../navigation";

/** Conversation results are found by `searchConversations`: kept as they are. */
const HIT_PREFIX = "conversation:";
const filter = (value: string, search: string, keywords?: string[]) =>
  value.startsWith(HIT_PREFIX) ? 1 : defaultFilter(value, search, keywords);

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
  const [query, setQuery] = useState("");
  const conversations = useChat((state) => state.conversations);
  const hits = open ? searchConversations(conversations, query) : [];

  const run = (action: () => void) => () => {
    setOpen(false);
    action();
  };
  const openHit = (conversationId: string, messageId: string | null) =>
    run(() => {
      useChat.getState().select(conversationId);
      useUi.getState().setFocusMessage(messageId);
      void navigate({ to: "/chat" });
    });

  return (
    <Dialog
      open={open}
      onOpenChange={(next) => {
        setOpen(next);
        if (!next) setQuery("");
      }}
    >
      <DialogContent>
        <DialogTitle className="sr-only">{t("palette")}</DialogTitle>
        <DialogDescription className="sr-only">{t("paletteHint")}</DialogDescription>
        <Command loop filter={filter}>
          <CommandInput placeholder={t("search")} value={query} onValueChange={setQuery} />
          <CommandList>
            <CommandEmpty>{t("noResults")}</CommandEmpty>
            {hits.length > 0 && (
              <CommandGroup heading={t("conversations")}>
                {hits.map((hit) => (
                  <CommandItem
                    key={hit.conversationId}
                    value={`${HIT_PREFIX}${hit.conversationId}`}
                    onSelect={openHit(hit.conversationId, hit.messageId)}
                    className="items-start"
                  >
                    <MessagesSquare className="mt-0.5" />
                    <span className="flex min-w-0 flex-col gap-0.5">
                      <span className="truncate">{hit.title || t("untitled")}</span>
                      {hit.snippet && (
                        <span className="line-clamp-2 text-xs text-muted-foreground">
                          <Highlighted text={hit.snippet} marks={hit.marks} />
                        </span>
                      )}
                    </span>
                  </CommandItem>
                ))}
              </CommandGroup>
            )}
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

/** `text` with the `marks` ranges highlighted. */
function Highlighted({ text, marks }: { text: string; marks: [number, number][] }) {
  const parts: React.ReactNode[] = [];
  let at = 0;
  for (const [start, end] of marks) {
    parts.push(text.slice(at, start));
    parts.push(
      <mark key={start} className="rounded-sm bg-primary/20 text-foreground">
        {text.slice(start, end)}
      </mark>,
    );
    at = end;
  }
  parts.push(text.slice(at));
  return <>{parts}</>;
}
