/** Conversations longer than the model's context window. */

import { fetchContextUsage, fittedStart, summarize } from "../context-usage";
import type { ChatSettings } from "../lib/request";
import { SUMMARY_MAX_TOKENS } from "../lib/summary";
import type { ChatMessage } from "../store";

/**
 * Too long for the model's context window: stop showing it the oldest
 * messages, enough of them that the start stays put for a few turns, and
 * have the model summarize them. `null` when everything fits.
 */
export async function fitContext({
  connection,
  modelId,
  settings,
  visible,
  summary,
  body,
  signal,
  onSummarizing,
}: {
  connection: { baseUrl: string; token: string };
  modelId: string;
  settings: ChatSettings;
  /** The messages the model sees now. */
  visible: readonly ChatMessage[];
  summary: string | undefined;
  /** The request as it would be sent now. */
  body: unknown;
  signal: AbortSignal;
  onSummarizing: (summarizing: boolean) => void;
}): Promise<{ visible: ChatMessage[]; summary: string | undefined } | null> {
  if (!settings.forgetOldest) return null;
  const summarizing = settings.summarizeForgotten;
  const start = await fetchContextUsage(modelId, body)
    .then((usage) =>
      fittedStart(usage, visible, settings.maxTokens, summarizing ? SUMMARY_MAX_TOKENS : 0),
    )
    .catch(() => 0);
  if (start === 0) return null;
  const forgotten = visible.slice(0, start);
  const kept = visible.slice(start);
  if (!summarizing) return { visible: kept, summary };

  onSummarizing(true);
  try {
    const next = await summarize({
      connection,
      model: modelId,
      systemPrompt: settings.systemPrompt,
      previous: summary,
      forgotten,
      signal,
    }).catch((error: unknown) => {
      // Without a new summary the oldest messages are only forgotten.
      if (signal.aborted) throw error;
      return summary;
    });
    return { visible: kept, summary: next };
  } finally {
    onSummarizing(false);
  }
}
