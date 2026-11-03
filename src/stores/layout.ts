import { create } from "zustand";
import { persist } from "zustand/middleware";

type LayoutState = {
  /** The navigation shows icons only. */
  sidebarCollapsed: boolean;
  /** The chat's conversation list is hidden. */
  conversationsHidden: boolean;
  toggleSidebar: () => void;
  toggleConversations: () => void;
};

/** Which side panels are folded away, persisted per user. */
export const useLayout = create<LayoutState>()(
  persist(
    (set) => ({
      sidebarCollapsed: false,
      conversationsHidden: false,
      toggleSidebar: () => set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),
      toggleConversations: () =>
        set((state) => ({ conversationsHidden: !state.conversationsHidden })),
    }),
    { name: "calcine.layout" },
  ),
);
