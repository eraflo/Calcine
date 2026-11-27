import { create } from "zustand";

type UiState = {
  paletteOpen: boolean;
  tasksOpen: boolean;
  /** The Library's import dialog, with a dropped path if any. */
  importing: { path: string | null } | null;
  /** A message to scroll to and flash in the chat (found by a search). */
  focusMessage: string | null;
  setPaletteOpen: (open: boolean) => void;
  setTasksOpen: (open: boolean) => void;
  setImporting: (importing: { path: string | null } | null) => void;
  setFocusMessage: (focusMessage: string | null) => void;
};

/** Transient UI state shared across the shell (not persisted). */
export const useUi = create<UiState>()((set) => ({
  paletteOpen: false,
  tasksOpen: false,
  importing: null,
  focusMessage: null,
  setPaletteOpen: (paletteOpen) => set({ paletteOpen }),
  setTasksOpen: (tasksOpen) => set({ tasksOpen }),
  setImporting: (importing) => set({ importing }),
  setFocusMessage: (focusMessage) => set({ focusMessage }),
}));
