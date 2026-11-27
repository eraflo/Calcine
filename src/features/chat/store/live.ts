/** State that changes too often, or is too large, to be saved. */

import { create } from "zustand";
import type { MediaPart } from "../lib/request";

/**
 * Image and audio data of attachments, kept in memory only: they're large,
 * and conversations are saved in local storage.
 */
export const useMediaPayloads = create<Record<string, MediaPart>>()(() => ({}));

/** The reply being generated (it changes per token). */
type LiveReply = {
  conversationId: string | null;
  messageId: string | null;
  content: string;
  reasoning: string;
  /** Work before the reply starts. */
  status: "summarizing" | "reading" | null;
  start: (conversationId: string, messageId: string) => void;
  setStatus: (status: LiveReply["status"]) => void;
  append: (delta: { content?: string; reasoning?: string }) => void;
  clear: () => void;
};

const idle = { conversationId: null, messageId: null, content: "", reasoning: "", status: null };

export const useLiveReply = create<LiveReply>()((set) => ({
  ...idle,
  start: (conversationId, messageId) => set({ ...idle, conversationId, messageId }),
  setStatus: (status) => set({ status }),
  append: (delta) =>
    set((state) => ({
      content: state.content + (delta.content ?? ""),
      reasoning: state.reasoning + (delta.reasoning ?? ""),
    })),
  clear: () => set(idle),
}));
