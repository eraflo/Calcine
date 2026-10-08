import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { AppChannel, ReleaseChannel } from "@/lib/api";

type UpdatesState = {
  /** GenieX releases to follow. */
  geniexChannel: ReleaseChannel;
  /** Calcine releases to follow. */
  appChannel: AppChannel;
  /** Look for GenieX and Calcine updates without being asked (see PRIVACY.md). */
  autoCheck: boolean;
  setGeniexChannel: (channel: ReleaseChannel) => void;
  setAppChannel: (channel: AppChannel) => void;
  setAutoCheck: (autoCheck: boolean) => void;
};

/** Update preferences, persisted per user. Stable by default. */
export const useUpdates = create<UpdatesState>()(
  persist(
    (set) => ({
      geniexChannel: "stable",
      appChannel: "stable",
      autoCheck: true,
      setGeniexChannel: (geniexChannel) => set({ geniexChannel }),
      setAppChannel: (appChannel) => set({ appChannel }),
      setAutoCheck: (autoCheck) => set({ autoCheck }),
    }),
    { name: "calcine.updates" },
  ),
);
