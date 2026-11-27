import { create } from "zustand";
import { persist } from "zustand/middleware";

type QuickAskState = {
  /** The model quick questions go to (`null`: the chat's). */
  modelId: string | null;
  setModelId: (modelId: string) => void;
};

export const useQuickAskStore = create<QuickAskState>()(
  persist(
    (set) => ({
      modelId: null,
      setModelId: (modelId) => set({ modelId }),
    }),
    { name: "calcine.quick-ask" },
  ),
);
