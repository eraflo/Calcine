import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { ReleaseChannel } from "@/lib/api";

export type AppChannel = "stable" | "beta";

type UpdatesState = {
  /** GenieX releases to follow. */
  geniexChannel: ReleaseChannel;
  /** Calcine releases to follow. */
  appChannel: AppChannel;
  setGeniexChannel: (channel: ReleaseChannel) => void;
  setAppChannel: (channel: AppChannel) => void;
};

/** Update preferences, persisted per user. Stable by default. */
export const useUpdates = create<UpdatesState>()(
  persist(
    (set) => ({
      geniexChannel: "stable",
      appChannel: "stable",
      setGeniexChannel: (geniexChannel) => set({ geniexChannel }),
      setAppChannel: (appChannel) => set({ appChannel }),
    }),
    { name: "calcine.updates" },
  ),
);
