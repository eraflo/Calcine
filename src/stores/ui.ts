import { create } from "zustand";

type UiState = {
  paletteOpen: boolean;
  tasksOpen: boolean;
  setPaletteOpen: (open: boolean) => void;
  setTasksOpen: (open: boolean) => void;
};

/** Transient UI state shared across the shell (not persisted). */
export const useUi = create<UiState>()((set) => ({
  paletteOpen: false,
  tasksOpen: false,
  setPaletteOpen: (paletteOpen) => set({ paletteOpen }),
  setTasksOpen: (tasksOpen) => set({ tasksOpen }),
}));
