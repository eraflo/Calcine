import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect } from "react";
import { create } from "zustand";
import { persist } from "zustand/middleware";

export type ThemePreference = "dark" | "light" | "system";

type ThemeState = {
  theme: ThemePreference;
  setTheme: (theme: ThemePreference) => void;
};

/** Theme preference, persisted per user. Dark is Calcine's default. */
export const useTheme = create<ThemeState>()(
  persist((set) => ({ theme: "dark", setTheme: (theme) => set({ theme }) }), {
    name: "calcine.theme",
  }),
);

const prefersLight = () => window.matchMedia("(prefers-color-scheme: light)");

/**
 * Apply the preference to `<html data-theme>` and the native title bar, and
 * follow the OS in "system" mode.
 */
export function useApplyTheme() {
  const theme = useTheme((state) => state.theme);

  useEffect(() => {
    if (isTauri()) {
      getCurrentWindow()
        .setTheme(theme === "system" ? null : theme)
        .catch(() => {});
    }

    const root = document.documentElement;
    const apply = () => {
      const resolved = theme === "system" ? (prefersLight().matches ? "light" : "dark") : theme;
      root.dataset.theme = resolved;
    };
    apply();
    if (theme !== "system") return;
    const media = prefersLight();
    media.addEventListener("change", apply);
    return () => media.removeEventListener("change", apply);
  }, [theme]);
}
