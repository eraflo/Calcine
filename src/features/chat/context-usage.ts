import { type ContextUsage, commands, unwrap } from "@/lib/api";
import { contextStart, replyReserve } from "./lib/context";
import type { ChatMessage } from "./store";

/** How much of `model`'s context window a chat request takes. */
export function fetchContextUsage(model: string, body: unknown): Promise<ContextUsage> {
  return unwrap(() => commands.contextUsage(model, JSON.stringify(body)));
}

/** Messages the model sees: from where it starts, without failed or empty ones. */
export function visibleMessages(
  messages: readonly ChatMessage[],
  contextFrom: string | undefined,
): ChatMessage[] {
  const start = Math.max(
    0,
    messages.findIndex((message) => message.id === contextFrom),
  );
  return messages
    .slice(start)
    .filter((message) => !message.error && (message.content || message.attachments?.length));
}

/**
 * Where `visible` should start for `usage` to fit the window, with room
 * for the reply: `0` when it fits, or when nothing can be counted (the
 * gateway then judges).
 */
export function fittedStart(
  usage: ContextUsage,
  visible: readonly ChatMessage[],
  maxTokens: number,
): number {
  const { tokens, window } = usage;
  if (window === null) return 0;
  // The system prompt comes before the turns.
  const leading = tokens.perMessage.length - visible.length;
  const fixed =
    tokens.overhead + tokens.perMessage.slice(0, leading).reduce((sum, count) => sum + count, 0);
  const start = contextStart(
    tokens.perMessage.slice(leading),
    visible.map((message) => message.role),
    fixed,
    window - replyReserve(maxTokens, window),
  );
  return start ?? 0;
}
