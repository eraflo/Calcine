import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { Attachment } from "./lib/attachments";
import { type ChatSettings, DEFAULT_SETTINGS, type MediaPart, POWER_MODES } from "./lib/request";
import type { StreamStats } from "./lib/sse";

export type ChatMessage = {
  id: string;
  role: "user" | "assistant";
  content: string;
  /** Images and recordings; their data is in {@link useMediaPayloads}. */
  attachments?: Attachment[];
  reasoning?: string;
  stats?: StreamStats;
  error?: string;
  /** Stopped by the user before the end. */
  stopped?: boolean;
};

export type Conversation = {
  id: string;
  title: string;
  modelId: string;
  messages: ChatMessage[];
  updatedAt: number;
};

type ChatState = {
  conversations: Conversation[];
  activeId: string | null;
  settings: ChatSettings;
  /** Model for the next new conversation. */
  lastModelId: string | null;
  create: (modelId: string) => string;
  select: (id: string | null) => void;
  remove: (id: string) => void;
  setModel: (id: string, modelId: string) => void;
  addMessage: (id: string, message: ChatMessage) => void;
  updateMessage: (id: string, messageId: string, patch: Partial<ChatMessage>) => void;
  setSettings: (patch: Partial<ChatSettings>) => void;
};

const newId = () => crypto.randomUUID();

/** Conversations and chat settings, kept on this machine (localStorage). */
export const useChat = create<ChatState>()(
  persist(
    (set) => ({
      conversations: [],
      activeId: null,
      settings: DEFAULT_SETTINGS,
      lastModelId: null,
      create: (modelId) => {
        const id = newId();
        set((state) => ({
          conversations: [
            { id, title: "", modelId, messages: [], updatedAt: Date.now() },
            ...state.conversations,
          ],
          activeId: id,
          lastModelId: modelId,
        }));
        return id;
      },
      select: (activeId) => set({ activeId }),
      remove: (id) =>
        set((state) => ({
          conversations: state.conversations.filter((c) => c.id !== id),
          activeId: state.activeId === id ? null : state.activeId,
        })),
      setModel: (id, modelId) =>
        set((state) => ({
          lastModelId: modelId,
          conversations: state.conversations.map((c) => (c.id === id ? { ...c, modelId } : c)),
        })),
      addMessage: (id, message) =>
        set((state) => ({
          conversations: state.conversations.map((c) =>
            c.id === id
              ? {
                  ...c,
                  title:
                    c.messages.length === 0 && message.role === "user"
                      ? titleFrom(message.content)
                      : c.title,
                  messages: [...c.messages, message],
                  updatedAt: Date.now(),
                }
              : c,
          ),
        })),
      updateMessage: (id, messageId, patch) =>
        set((state) => ({
          conversations: state.conversations.map((c) =>
            c.id === id
              ? {
                  ...c,
                  messages: c.messages.map((m) => (m.id === messageId ? { ...m, ...patch } : m)),
                }
              : c,
          ),
        })),
      setSettings: (patch) => set((state) => ({ settings: { ...state.settings, ...patch } })),
    }),
    {
      name: "calcine.chat",
      version: 2,
      migrate: (persisted, version) => {
        const state = persisted as Pick<ChatState, "conversations" | "settings">;
        if (version < 2) {
          // Titles are shown translated when empty; power modes lost their labels.
          state.conversations = state.conversations.map((c) =>
            c.title === "New chat" ? { ...c, title: "" } : c,
          );
          if (!POWER_MODES.includes(state.settings.powerMode)) {
            state.settings = { ...state.settings, powerMode: DEFAULT_SETTINGS.powerMode };
          }
        }
        return state as ChatState;
      },
    },
  ),
);

export function titleFrom(text: string): string {
  const line = text.trim().split("\n")[0] ?? "";
  return line.length > 48 ? `${line.slice(0, 47)}…` : line;
}

/**
 * Image and audio data of attachments, kept in memory only: they're large,
 * and conversations are saved in local storage.
 */
export const useMediaPayloads = create<Record<string, MediaPart>>()(() => ({}));

/** The reply being generated, kept out of persisted state (it changes per token). */
type LiveReply = {
  conversationId: string | null;
  messageId: string | null;
  content: string;
  reasoning: string;
  start: (conversationId: string, messageId: string) => void;
  append: (delta: { content?: string; reasoning?: string }) => void;
  clear: () => void;
};

export const useLiveReply = create<LiveReply>()((set) => ({
  conversationId: null,
  messageId: null,
  content: "",
  reasoning: "",
  start: (conversationId, messageId) =>
    set({ conversationId, messageId, content: "", reasoning: "" }),
  append: (delta) =>
    set((state) => ({
      content: state.content + (delta.content ?? ""),
      reasoning: state.reasoning + (delta.reasoning ?? ""),
    })),
  clear: () => set({ conversationId: null, messageId: null, content: "", reasoning: "" }),
}));

export { newId };
