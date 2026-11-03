import { create } from "zustand";
import { persist } from "zustand/middleware";
import { isWindows } from "@/lib/platform";

/** Where the dictation model listens. `npu`: the model on the NPU, the voice decoded on the CPU. */
export type DictationUnit = "gpu" | "cpu" | "npu";

/** Languages the transcription can be told to expect; `auto` lets the model tell. */
export const DICTATION_LANGUAGES = ["auto", "fr", "en", "es", "de", "it", "pt"] as const;
export type DictationLanguage = (typeof DICTATION_LANGUAGES)[number];

type DictationState = {
  /** Model that transcribes; `null` picks the first installed one that understands speech. */
  model: string | null;
  unit: DictationUnit;
  language: DictationLanguage;
  setModel: (model: string | null) => void;
  setUnit: (unit: DictationUnit) => void;
  setLanguage: (language: DictationLanguage) => void;
};

/** Dictation preferences, persisted per user. The GPU listens fastest on Windows. */
export const useDictationSettings = create<DictationState>()(
  persist(
    (set) => ({
      model: null,
      unit: isWindows ? "gpu" : "cpu",
      language: "auto",
      setModel: (model) => set({ model }),
      setUnit: (unit) => set({ unit }),
      setLanguage: (language) => set({ language }),
    }),
    { name: "calcine.dictation" },
  ),
);
