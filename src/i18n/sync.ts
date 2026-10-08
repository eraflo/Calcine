import { isTauri } from "@tauri-apps/api/core";
import { useEffect } from "react";
import { commands } from "@/lib/api";
import { useLanguage } from "./store";

/**
 * Tell the page (`<html lang>`, for screen readers and hyphenation) and the
 * shell (tray menu, notifications) which language the UI uses. Mount once.
 */
export function useSyncLanguage() {
  const language = useLanguage((state) => state.language);
  useEffect(() => {
    document.documentElement.lang = language;
    if (isTauri()) void commands.setLanguage(language);
  }, [language]);
}
