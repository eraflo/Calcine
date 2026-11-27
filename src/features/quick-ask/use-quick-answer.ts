import { useQuery } from "@tanstack/react-query";
import { useCallback, useRef, useState } from "react";
import { buildChatRequest } from "@/features/chat/lib/request";
import { type StreamStats, streamChat } from "@/features/chat/lib/sse";
import { connectionQuery } from "@/features/server/api";
import type { LocalModel } from "@/lib/api";
import { chatPreferences, quickSettings } from "./lib/prompt";

export type QuickAnswer = {
  modelId: string;
  /** What was sent, copied text included. */
  prompt: string;
  content: string;
  stats?: StreamStats;
  error?: string;
  busy: boolean;
};

/** One question, one streamed answer, with the chat's settings. */
export function useQuickAnswer() {
  const { data: connection } = useQuery(connectionQuery);
  const [answer, setAnswer] = useState<QuickAnswer | null>(null);
  const controller = useRef<AbortController | null>(null);

  const ask = useCallback(
    async (model: LocalModel | undefined, modelId: string, prompt: string) => {
      if (!connection) return;
      controller.current?.abort();
      const abort = new AbortController();
      controller.current = abort;
      setAnswer({ modelId, prompt, content: "", busy: true });
      const settings = quickSettings(chatPreferences().settings);
      try {
        const result = await streamChat({
          url: `${connection.baseUrl}/chat/completions`,
          token: connection.token,
          body: buildChatRequest(model, modelId, settings, [{ role: "user", content: prompt }]),
          signal: abort.signal,
          onDelta: ({ content }) => {
            if (content) {
              setAnswer((current) => current && { ...current, content: current.content + content });
            }
          },
        });
        setAnswer((current) => current && { ...current, stats: result.stats, busy: false });
      } catch (error) {
        setAnswer(
          (current) =>
            current && {
              ...current,
              busy: false,
              error: abort.signal.aborted
                ? undefined
                : error instanceof Error
                  ? error.message
                  : String(error),
            },
        );
      } finally {
        if (controller.current === abort) controller.current = null;
      }
    },
    [connection],
  );

  const stop = useCallback(() => controller.current?.abort(), []);

  return { answer, ask, stop, ready: Boolean(connection) };
}
