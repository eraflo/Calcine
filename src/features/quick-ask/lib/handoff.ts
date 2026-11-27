import type { StreamStats } from "@/features/chat/lib/sse";

/** Sent from the quick question window to Calcine's window, to go on in
 * the chat. */
export const HANDOFF_EVENT = "quick-ask:continue";

export type Handoff = {
  modelId: string;
  question: string;
  answer: string;
  stats?: StreamStats;
};
