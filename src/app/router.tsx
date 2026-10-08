import {
  createHashHistory,
  createRootRoute,
  createRoute,
  createRouter,
  redirect,
} from "@tanstack/react-router";
import type { ReactNode } from "react";
import { ChatPage } from "@/features/chat/chat-page";
import { DiscoverPage } from "@/features/discover/discover-page";
import { HardwarePage } from "@/features/hardware/hardware-page";
import { LibraryPage } from "@/features/library/library-page";
import { ServerPage } from "@/features/server/server-page";
import { SettingsPage } from "@/features/settings/settings-page";
import { AppShell } from "./shell/app-shell";

const rootRoute = createRootRoute({ component: AppShell });

/** A top-level page. Generic so each path stays a literal type for typed links. */
const page = <TPath extends string>(path: TPath, component: () => ReactNode) =>
  createRoute({ getParentRoute: () => rootRoute, path, component });

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  beforeLoad: () => {
    throw redirect({ to: "/library" });
  },
});

const routeTree = rootRoute.addChildren([
  indexRoute,
  page("/chat", ChatPage),
  page("/library", LibraryPage),
  page("/discover", DiscoverPage),
  page("/server", ServerPage),
  page("/hardware", HardwarePage),
  page("/settings", SettingsPage),
]);

export const router = createRouter({
  routeTree,
  // A desktop app has no server to rewrite deep links: hash routing always resolves.
  history: createHashHistory(),
  defaultPreload: "intent",
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
