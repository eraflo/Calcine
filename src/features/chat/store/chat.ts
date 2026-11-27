import { create } from "zustand";
import { persist } from "zustand/middleware";
import { DEFAULT_SETTINGS } from "../lib/request";
import { conversationActions } from "./conversations";
import { documentActions } from "./documents";
import { CHAT_VERSION, migrateChat } from "./migrate";
import { promptActions } from "./prompts";
import type { ChatState } from "./types";

/** Conversations and chat settings, kept on this machine (localStorage). */
export const useChat = create<ChatState>()(
  persist(
    (set) => ({
      conversations: [],
      activeId: null,
      settings: DEFAULT_SETTINGS,
      lastModelId: null,
      prompts: [],
      setSettings: (patch) => set((state) => ({ settings: { ...state.settings, ...patch } })),
      ...conversationActions(set),
      ...documentActions(set),
      ...promptActions(set),
    }),
    { name: "calcine.chat", version: CHAT_VERSION, migrate: migrateChat },
  ),
);
