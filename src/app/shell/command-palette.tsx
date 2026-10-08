import { useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { Copy, Monitor, Moon, RefreshCw, Sun } from "lucide-react";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { modelsQuery } from "@/features/library/api";
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
                onSelect={run(() =>
                  queryClient.invalidateQueries({ queryKey: modelsQuery.queryKey }),
                )}
              >
                <RefreshCw />
                Refresh model library
              </CommandItem>
              <CommandItem onSelect={run(() => navigator.clipboard.writeText(API_BASE_URL))}>
                <Copy />
                Copy API base URL
                <span className="ml-auto font-mono text-xs text-muted-foreground">
                  {API_BASE_URL}
                </span>
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
