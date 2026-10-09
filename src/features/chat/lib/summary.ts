/**
 * Summaries of the messages a conversation outgrew. The model writes them
 * itself, and they go at the very start of the prompt (in the system
 * prompt): the start stays the same until the next cut, so GenieX keeps
 * reusing what it already read.
 */

/** Longest summary, in tokens: also kept free in the context window. */
export const SUMMARY_MAX_TOKENS = 256;

/** Longest summary kept, in characters, if the model ran on. */
const MAX_SUMMARY_CHARS = 1200;

export type SummaryTurn = { role: "user" | "assistant"; content: string };

/** The system prompt with the summary of what the model no longer sees. */
export function withSummary(systemPrompt: string, summary: string | undefined): string {
  const prompt = systemPrompt.trim();
  if (!summary?.trim()) return prompt;
  const block = `Summary of the earlier part of this conversation, which you can no longer see:\n${summary.trim()}`;
  return prompt ? `${prompt}\n\n${block}` : block;
}

/**
 * The request messages asking for a summary of `forgotten`: the
 * conversation as the chat sent it (same system prompt, same turns), then
 * the request. Measured on the NPU, small models follow instructions left
 * in a quoted transcript ("Reply OK") but summarize real turns well, and
 * GenieX reuses the conversation it already read.
 */
export function summaryMessages(
  systemPrompt: string,
  previous: string | undefined,
  forgotten: readonly SummaryTurn[],
) {
  const system = withSummary(systemPrompt, previous);
  return [
    ...(system ? [{ role: "system" as const, content: system }] : []),
    ...forgotten.filter((turn) => turn.content.trim()),
    {
      role: "user" as const,
      content:
        "Let's pause here. Summarize this conversation so far for your own notes, in the third " +
        'person ("The user…"): who the user is, the facts, names, numbers, dates, decisions, ' +
        "their preferences and constraints, and open questions" +
        (previous?.trim() ? ", including the summary of earlier messages you were given" : "") +
        ". Skip small talk and repeated notes. At most 120 words, in the language of the " +
        "conversation, without repeating yourself. Write only the summary.",
    },
  ];
}

/** A summary as the model wrote it: no heading, thinking or repeated sentences. */
export function cleanSummary(text: string): string {
  const body = text
    .replace(/<think>[\s\S]*?<\/think>/g, "")
    .trim()
    .replace(/^(\*\*)?summary\s*(:\s*(\*\*)?|(\*\*)?\s*:)\s*/i, "");
  const seen = new Set<string>();
  const sentences = body.split(/(?<=[.!?])\s+/).filter((sentence) => {
    const key = sentence.toLowerCase().replace(/\W+/g, " ").trim();
    if (!key || seen.has(key)) return false;
    seen.add(key);
    return true;
  });
  const joined = sentences.join(" ").trim();
  return joined.length > MAX_SUMMARY_CHARS ? `${joined.slice(0, MAX_SUMMARY_CHARS)}…` : joined;
}
