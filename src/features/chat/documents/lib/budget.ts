/** How many tokens the documents may take in the next request. */

import { replyReserve } from "../../lib/context";

/** Used when the model's window is unknown. */
const UNKNOWN_WINDOW_BUDGET = 2048;
/** Documents always get at least this share of the room left after the
 * reply (older messages are forgotten to make it)… */
const MIN_SHARE = 0.35;
/** …and at most this share, so the conversation still fits. */
const MAX_SHARE = 0.65;

/**
 * What the conversation leaves free, between {@link MIN_SHARE} and
 * {@link MAX_SHARE} of the room left after the reply.
 */
export function documentBudget(
  window: number | null,
  conversationTokens: number,
  maxTokens: number,
): number {
  if (window === null) return UNKNOWN_WINDOW_BUDGET;
  const room = window - replyReserve(maxTokens, window);
  const free = room - conversationTokens;
  const budget = Math.min(Math.max(free, room * MIN_SHARE), room * MAX_SHARE);
  return Math.max(0, Math.floor(budget));
}
