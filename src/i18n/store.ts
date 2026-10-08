import { create } from "zustand";
import { persist } from "zustand/middleware";

export type Language = "en" | "fr";
export type LanguagePreference = Language | "system";

const SUPPORTED: readonly Language[] = ["en", "fr"];

/** The OS language if Calcine speaks it, else English. */
export function systemLanguage(): Language {
  const languages = typeof navigator === "undefined" ? [] : navigator.languages;
  for (const tag of languages) {
    const base = tag.toLowerCase().split("-")[0] as Language;
    if (SUPPORTED.includes(base)) return base;
  }
  return "en";
}

const resolve = (preference: LanguagePreference): Language =>
  preference === "system" ? systemLanguage() : preference;

type LanguageState = {
  preference: LanguagePreference;
  /** What the UI is shown in. */
  language: Language;
  setPreference: (preference: LanguagePreference) => void;
};

/** Language preference, persisted per user. Follows the OS by default. */
export const useLanguage = create<LanguageState>()(
  persist(
    (set) => ({
      preference: "system",
      language: systemLanguage(),
      setPreference: (preference) => set({ preference, language: resolve(preference) }),
    }),
    {
      name: "calcine.language",
      partialize: (state) => ({ preference: state.preference }),
      merge: (persisted, current) => {
        const preference =
          (persisted as Partial<LanguageState> | undefined)?.preference ?? "system";
        return { ...current, preference, language: resolve(preference) };
      },
    },
  ),
);

/** Current language outside React (formatters). */
export const getLanguage = () => useLanguage.getState().language;
