import type { TokenCount } from "@/lib/api";

/**
 * When a conversation outgrows the window, it's cut down to this share of
 * the room it has, not just under it: its start then stays the same for a
 * few turns, and GenieX keeps reusing what it already read instead of
 * reading the whole conversation again on every message.
 */
export const REFILL = 0.75;

export function totalTokens(count: TokenCount): number {
  return count.perMessage.reduce((sum, tokens) => sum + tokens, 0) + count.overhead;
}

/** Room left for the reply: the max tokens setting, at most half the window. */
export function replyReserve(maxTokens: number, window: number): number {
  return Math.min(maxTokens, Math.floor(window / 2));
}

/**
 * Where to start the turns so the prompt fits `budget` tokens, cut down to
 * {@link REFILL} of it: an index into `roles`, at a user turn. `fixed` is
 * what's always sent (system prompt, template). `0` when everything fits,
 * `null` when not even the last turn does.
 */
export function contextStart(
  turnTokens: readonly number[],
  roles: readonly string[],
  fixed: number,
  budget: number,
): number | null {
  const sum = (from: number) => turnTokens.slice(from).reduce((total, tokens) => total + tokens, 0);
  if (fixed + sum(0) <= budget) return 0;
  const userTurns = roles.flatMap((role, index) => (role === "user" ? [index] : []));
  const target = Math.floor(budget * REFILL);
  const fits = (limit: number) => userTurns.find((index) => fixed + sum(index) <= limit);
  return fits(target) ?? fits(budget) ?? null;
}

/** How full the context is, for the meter's colour. */
export function fillLevel(used: number, window: number): "ok" | "high" | "full" {
  const share = used / window;
  if (share >= 0.95) return "full";
  if (share >= 0.75) return "high";
  return "ok";
}
