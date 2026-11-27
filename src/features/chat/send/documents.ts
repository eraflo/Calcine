/** What the model reads of the conversation's documents for a message. */

import { commands, type DocumentContext, type DocumentInfo, unwrap } from "@/lib/api";
import { fetchContextUsage } from "../context-usage";
import { documentBudget } from "../documents/lib/budget";
import { documentQueries } from "../documents/lib/prompt";
import { totalTokens } from "../lib/context";
import type { ChatMessage } from "../store";

/**
 * In the room the conversation leaves (measured on `body`, the request
 * without documents), the passages that best match the message, or the
 * whole documents when they fit.
 */
export async function readDocuments({
  modelId,
  documents,
  text,
  earlier,
  body,
  maxTokens,
}: {
  modelId: string;
  documents: readonly DocumentInfo[];
  /** The user's message. */
  text: string;
  /** The conversation before it, for follow-up questions. */
  earlier: readonly ChatMessage[];
  body: unknown;
  maxTokens: number;
}): Promise<DocumentContext> {
  const usage = await fetchContextUsage(modelId, body);
  const budget = documentBudget(usage.window, totalTokens(usage.tokens), maxTokens);
  return unwrap(() =>
    commands.documentContext(
      modelId,
      documents.map((document) => document.id),
      documentQueries(text, earlier),
      budget,
    ),
  );
}

/** What the reply keeps of what was read: its sources and how much was read. */
export function sourcesOf(read: DocumentContext | null): Partial<ChatMessage> {
  if (!read || read.sources.length === 0) return {};
  return {
    sources: read.sources,
    documentMode: read.mode,
    passagesRead: { read: read.passagesRead, total: read.passagesTotal, tokens: read.tokens },
  };
}
