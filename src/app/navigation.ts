import type { LucideIcon } from "lucide-react";
import { Compass, Cpu, Library, MessagesSquare, Server, Settings } from "lucide-react";
import type { messages } from "./messages";

type NavItem = { to: string; label: keyof (typeof messages)["en"]; icon: LucideIcon };

/**
 * Pages shown in the sidebar and the command palette. To add a page, create
 * its route in `router.tsx`, list it here and name it in `messages.ts`.
 */
export const mainNavigation = [
  { to: "/chat", label: "nav_chat", icon: MessagesSquare },
  { to: "/library", label: "nav_library", icon: Library },
  { to: "/discover", label: "nav_discover", icon: Compass },
  { to: "/server", label: "nav_server", icon: Server },
  { to: "/hardware", label: "nav_hardware", icon: Cpu },
] as const satisfies readonly NavItem[];

export const settingsNavigation = {
  to: "/settings",
  label: "nav_settings",
  icon: Settings,
} as const satisfies NavItem;

export const allNavigation = [...mainNavigation, settingsNavigation] as const;
