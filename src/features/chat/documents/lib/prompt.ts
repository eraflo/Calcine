/**
 * Where the documents go in a chat request. Read whole, they join the
 * system prompt, where they stay the same from one message to the next
 * (GenieX reuses what it already read); passages go before the message
 * they answer.
 */

import type { DocumentContext } from "@/lib/api";

/** What to look for in the documents: the message, then the user's two
 * previous ones, for follow-up questions. */
export function documentQueries(
  text: string,
  earlier: readonly { role: string; content: string }[],
): string[] {
  const previous = earlier
    .filter((message) => message.role === "user" && message.content.trim())
    .slice(-2)
    .reverse()
    .map((message) => message.content);
  return [text, ...previous];
}

/** The system prompt's share (`system`) or the message's (`prefix`). */
export function documentParts(context: DocumentContext | null): {
  system?: string;
  prefix?: string;
} {
  if (!context?.text || context.sources.length === 0) return {};
  return context.mode === "full" ? { system: context.text } : { prefix: context.text };
}

/** The system prompt with the documents, before any summary. */
export function withDocuments(systemPrompt: string, documents: string | undefined): string {
  const prompt = systemPrompt.trim();
  if (!documents) return prompt;
  return prompt ? `${prompt}\n\n${documents}` : documents;
}

/** The user's message with the passages that may answer it. */
export function withPassages(content: string, passages: string | undefined): string {
  if (!passages) return content;
  return `${passages}\n${content}`;
}
