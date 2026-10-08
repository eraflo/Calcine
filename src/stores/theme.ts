import { isTauri } from "@tauri-apps/api/core";
import { Effect, getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect } from "react";
import { create } from "zustand";
import { persist } from "zustand/middleware";

export type ThemePreference = "dark" | "light" | "system";

type ThemeState = {
  theme: ThemePreference;
  /** Let Windows 11's Mica material show through the window background. */
  mica: boolean;
  setTheme: (theme: ThemePreference) => void;
  setMica: (mica: boolean) => void;
};

/** Theme preference, persisted per user. Dark and opaque by default. */
export const useTheme = create<ThemeState>()(
  persist(
    (set) => ({
      theme: "dark",
      mica: false,
      setTheme: (theme) => set({ theme }),
      setMica: (mica) => set({ mica }),
    }),
    { name: "calcine.theme" },
  ),
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

/** Turn the Mica material on or off (`<html data-material>` and the window effect). */
export function useApplyMaterial() {
  const mica = useTheme((state) => state.mica);

  useEffect(() => {
    document.documentElement.dataset.material = mica ? "mica" : "solid";
    if (!isTauri()) return;
    const window = getCurrentWindow();
    (mica ? window.setEffects({ effects: [Effect.Mica] }) : window.clearEffects()).catch(() => {});
  }, [mica]);
}
