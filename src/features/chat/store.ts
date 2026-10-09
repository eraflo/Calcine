import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { Attachment } from "./lib/attachments";
import {
  type ChatSettings,
  DEFAULT_SETTINGS,
  type MediaPart,
  POWER_MODES,
  withDefaults,
} from "./lib/request";
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
  /** The first message the model still sees, when the conversation
   * outgrew its context window. */
  contextFrom?: string;
  /** What came before `contextFrom`, summarized by the model. */
  summary?: string;
};

/** A system prompt saved for reuse. */
export type SavedPrompt = { id: string; name: string; text: string };

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
  /** Where the model's view starts, and the summary of what's before. */
  setContext: (id: string, context: Pick<Conversation, "contextFrom" | "summary">) => void;
  setSettings: (patch: Partial<ChatSettings>) => void;
  prompts: SavedPrompt[];
  savePrompt: (name: string, text: string) => void;
  removePrompt: (id: string) => void;
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
      setContext: (id, context) =>
        set((state) => ({
          conversations: state.conversations.map((c) => (c.id === id ? { ...c, ...context } : c)),
        })),
      setSettings: (patch) => set((state) => ({ settings: { ...state.settings, ...patch } })),
      prompts: [],
      // Saving under an existing name replaces that prompt.
      savePrompt: (name, text) =>
        set((state) => ({
          prompts: [
            ...state.prompts.filter((prompt) => prompt.name !== name),
            { id: newId(), name, text },
          ],
        })),
      removePrompt: (id) =>
        set((state) => ({ prompts: state.prompts.filter((prompt) => prompt.id !== id) })),
    }),
    {
      name: "calcine.chat",
      version: 5,
      migrate: (persisted, version) => {
        const state = persisted as Pick<ChatState, "conversations" | "settings" | "prompts">;
        if (version < 2) {
          // Titles are shown translated when empty; power modes lost their labels.
          state.conversations = state.conversations.map((c) =>
            c.title === "New chat" ? { ...c, title: "" } : c,
          );
          if (!POWER_MODES.includes(state.settings.powerMode)) {
            state.settings = { ...state.settings, powerMode: DEFAULT_SETTINGS.powerMode };
          }
        }
        // New settings get their defaults.
        state.settings = withDefaults(state.settings);
        state.prompts ??= [];
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
  /** Work before the reply starts. */
  status: "summarizing" | null;
  start: (conversationId: string, messageId: string) => void;
  setStatus: (status: LiveReply["status"]) => void;
  append: (delta: { content?: string; reasoning?: string }) => void;
  clear: () => void;
};

export const useLiveReply = create<LiveReply>()((set) => ({
  conversationId: null,
  messageId: null,
  content: "",
  reasoning: "",
  status: null,
  start: (conversationId, messageId) =>
    set({ conversationId, messageId, content: "", reasoning: "", status: null }),
  setStatus: (status) => set({ status }),
  append: (delta) =>
    set((state) => ({
      content: state.content + (delta.content ?? ""),
      reasoning: state.reasoning + (delta.reasoning ?? ""),
    })),
  clear: () =>
    set({ conversationId: null, messageId: null, content: "", reasoning: "", status: null }),
}));

export { newId };
