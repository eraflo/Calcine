import { create } from "zustand";

type UiState = {
  paletteOpen: boolean;
  tasksOpen: boolean;
  /** The Library's import dialog, with a dropped path if any. */
  importing: { path: string | null } | null;
  setPaletteOpen: (open: boolean) => void;
  setTasksOpen: (open: boolean) => void;
  setImporting: (importing: { path: string | null } | null) => void;
};

/** Transient UI state shared across the shell (not persisted). */
export const useUi = create<UiState>()((set) => ({
  paletteOpen: false,
  tasksOpen: false,
  importing: null,
  setPaletteOpen: (paletteOpen) => set({ paletteOpen }),
  setTasksOpen: (tasksOpen) => set({ tasksOpen }),
  setImporting: (importing) => set({ importing }),
}));
