/** Conversations and their messages. */

import { forgetDocuments } from "./documents";
import type { ChatState, SetChat } from "./types";
import { mapConversation } from "./update";
import { patchReply } from "./versions";

export const newId = () => crypto.randomUUID();

/** A conversation's title: its first line, shortened. */
export function titleFrom(text: string): string {
  const line = text.trim().split("\n")[0] ?? "";
  return line.length > 48 ? `${line.slice(0, 47)}…` : line;
}

export function conversationActions(
  set: SetChat,
): Pick<
  ChatState,
  "create" | "select" | "remove" | "setModel" | "addMessage" | "updateMessage" | "setContext"
> {
  return {
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
    select: (activeId) => set(() => ({ activeId })),
    remove: (id) =>
      set((state) => {
        const removed = state.conversations.find((c) => c.id === id);
        forgetDocuments(removed?.documents?.map((document) => document.id) ?? []);
        return {
          conversations: state.conversations.filter((c) => c.id !== id),
          activeId: state.activeId === id ? null : state.activeId,
        };
      }),
    setModel: (id, modelId) =>
      set((state) => ({
        lastModelId: modelId,
        conversations: mapConversation(state.conversations, id, (c) => ({ ...c, modelId })),
      })),
    addMessage: (id, message) =>
      set((state) => ({
        conversations: mapConversation(state.conversations, id, (c) => ({
          ...c,
          // The first message names the conversation.
          title:
            c.messages.length === 0 && message.role === "user"
              ? titleFrom(message.content)
              : c.title,
          messages: [...c.messages, message],
          updatedAt: Date.now(),
        })),
      })),
    updateMessage: (id, messageId, patch) =>
      set((state) => ({
        conversations: mapConversation(state.conversations, id, (c) => ({
          ...c,
          messages: c.messages.map((m) => (m.id === messageId ? patchReply(m, patch) : m)),
        })),
      })),
    setContext: (id, context) =>
      set((state) => ({
        conversations: mapConversation(state.conversations, id, (c) => ({ ...c, ...context })),
      })),
  };
}
