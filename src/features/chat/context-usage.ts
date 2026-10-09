import { type ContextUsage, commands, unwrap } from "@/lib/api";
import { contextStart, replyReserve } from "./lib/context";
import { errorMessage } from "./lib/sse";
import { cleanSummary, SUMMARY_MAX_TOKENS, summaryMessages } from "./lib/summary";
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
  /** Kept free too, e.g. for a longer summary. */
  extraReserve = 0,
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
    window - replyReserve(maxTokens, window) - extraReserve,
  );
  return start ?? 0;
}

/** Have the model summarize `forgotten`, folding in the previous summary. */
export async function summarize({
  connection,
  model,
  systemPrompt,
  previous,
  forgotten,
  signal,
}: {
  connection: { baseUrl: string; token: string };
  model: string;
  /** The chat's own, so GenieX reuses the conversation it already read. */
  systemPrompt: string;
  previous: string | undefined;
  forgotten: readonly ChatMessage[];
  signal: AbortSignal;
}): Promise<string> {
  const response = await fetch(`${connection.baseUrl}/chat/completions`, {
    method: "POST",
    headers: { Authorization: `Bearer ${connection.token}`, "Content-Type": "application/json" },
    body: JSON.stringify({
      model,
      messages: summaryMessages(
        systemPrompt,
        previous,
        forgotten.map(({ role, content }) => ({ role, content })),
      ),
      max_tokens: SUMMARY_MAX_TOKENS,
      temperature: 0.3,
      // Small models otherwise loop on a sentence until max_tokens.
      repetition_penalty: 1.1,
      // In case what's forgotten doesn't fit next to the request.
      truncation: "auto",
      enable_think: false,
      reasoning_format: "deepseek",
    }),
    signal,
  });
  if (!response.ok) throw new Error(await errorMessage(response));
  const reply = (await response.json()) as { choices?: { message?: { content?: string } }[] };
  const summary = cleanSummary(reply.choices?.[0]?.message?.content ?? "");
  if (!summary) throw new Error("The model wrote an empty summary.");
  return summary;
}
