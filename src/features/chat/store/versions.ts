/**
 * Versions of a reply, and editing a message.
 *
 * A regenerated reply keeps its earlier versions in `versions` (all of
 * them, the one shown included, at `version`); the message's own fields
 * are always those of the version shown, so the rest of the chat reads it
 * as any reply. Editing a user message drops what came after it.
 */

import type { ChatMessage, ChatState, ReplyVersion, SetChat } from "./types";
import { mapConversation } from "./update";

/** What differs from one version of a reply to another. */
const REPLY_FIELDS = [
  "content",
  "reasoning",
  "stats",
  "error",
  "stopped",
  "sources",
  "documentMode",
  "passagesRead",
] as const satisfies readonly (keyof ReplyVersion)[];

export function snapshot(message: ChatMessage): ReplyVersion {
  const version: ReplyVersion = { content: message.content };
  for (const field of REPLY_FIELDS) {
    if (message[field] !== undefined) Object.assign(version, { [field]: message[field] });
  }
  return version;
}

/** `message` showing `version`. */
function showing(message: ChatMessage, version: ReplyVersion, index: number): ChatMessage {
  const cleared: ChatMessage = { ...message, version: index };
  for (const field of REPLY_FIELDS) delete cleared[field];
  return { ...cleared, ...version };
}

/** A reply patched while it streams: its shown version follows. */
export function patchReply(message: ChatMessage, patch: Partial<ChatMessage>): ChatMessage {
  const next = { ...message, ...patch };
  if (!next.versions || next.version === undefined) return next;
  const versions = [...next.versions];
  versions[next.version] = snapshot(next);
  return { ...next, versions };
}

export function versionActions(
  set: SetChat,
): Pick<ChatState, "newVersion" | "selectVersion" | "editMessage"> {
  return {
    newVersion: (id, messageId) =>
      set((state) => ({
        conversations: mapConversation(state.conversations, id, (c) => ({
          ...c,
          messages: c.messages.map((m) => {
            if (m.id !== messageId) return m;
            const versions = [...(m.versions ?? [snapshot(m)]), { content: "" }];
            return showing({ ...m, versions }, { content: "" }, versions.length - 1);
          }),
        })),
      })),
    selectVersion: (id, messageId, index) =>
      set((state) => ({
        conversations: mapConversation(state.conversations, id, (c) => ({
          ...c,
          messages: c.messages.map((m) => {
            const version = m.versions?.[index];
            return m.id === messageId && version ? showing(m, version, index) : m;
          }),
        })),
      })),
    editMessage: (id, messageId, content) =>
      set((state) => ({
        conversations: mapConversation(state.conversations, id, (c) => {
          const index = c.messages.findIndex((m) => m.id === messageId);
          if (index === -1) return c;
          const messages = c.messages
            .slice(0, index + 1)
            .map((m) => (m.id === messageId ? { ...m, content } : m));
          // The model's view can't start after the end.
          const keepsStart = messages.some((m) => m.id === c.contextFrom);
          return {
            ...c,
            messages,
            updatedAt: Date.now(),
            ...(keepsStart ? {} : { contextFrom: undefined, summary: undefined }),
          };
        }),
      })),
  };
}
