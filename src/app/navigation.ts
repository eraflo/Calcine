import type { LucideIcon } from "lucide-react";
import { Compass, Cpu, Library, MessagesSquare, Server, Settings } from "lucide-react";

type NavItem = { to: string; label: string; icon: LucideIcon };

/**
 * Pages shown in the sidebar and the command palette. To add a page, create
 * its route in `router.tsx` and list it here.
 */
export const mainNavigation = [
  { to: "/chat", label: "Chat", icon: MessagesSquare },
  { to: "/library", label: "Library", icon: Library },
  { to: "/discover", label: "Discover", icon: Compass },
  { to: "/server", label: "Server", icon: Server },
  { to: "/hardware", label: "Hardware", icon: Cpu },
] as const satisfies readonly NavItem[];

export const settingsNavigation = {
  to: "/settings",
  label: "Settings",
  icon: Settings,
} as const satisfies NavItem;

export const allNavigation = [...mainNavigation, settingsNavigation] as const;
