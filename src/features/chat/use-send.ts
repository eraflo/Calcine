import { useQuery } from "@tanstack/react-query";
import { useCallback, useRef } from "react";
import { modelsQuery } from "@/features/library/api";
import { connectionQuery } from "@/features/server/api";
import { buildChatRequest, type ChatTurn } from "./lib/request";
import { streamChat } from "./lib/sse";
import { newId, useChat, useLiveReply } from "./store";

/** Send a message in a conversation and stream the reply; stop it on demand. */
export function useSend() {
  const { data: connection } = useQuery(connectionQuery);
  const { data: models = [] } = useQuery(modelsQuery);
  const controller = useRef<AbortController | null>(null);
  const streaming = useLiveReply((state) => state.messageId !== null);

  const send = useCallback(
    async (conversationId: string, text: string) => {
      const chat = useChat.getState();
      const conversation = chat.conversations.find((c) => c.id === conversationId);
      if (!conversation || !connection || controller.current) return;

      const history: ChatTurn[] = conversation.messages
        .filter((message) => !message.error && message.content)
        .map(({ role, content }) => ({ role, content }));
      history.push({ role: "user", content: text });

      chat.addMessage(conversationId, { id: newId(), role: "user", content: text });
      const replyId = newId();
      chat.addMessage(conversationId, { id: replyId, role: "assistant", content: "" });

      const live = useLiveReply.getState();
      live.start(conversationId, replyId);
      const abort = new AbortController();
      controller.current = abort;

      const model = models.find((candidate) => candidate.name === conversation.modelId);
      const body = buildChatRequest(model, conversation.modelId, chat.settings, history);
      try {
        const result = await streamChat({
          url: `${connection.baseUrl}/chat/completions`,
          token: connection.token,
          body,
          signal: abort.signal,
          onDelta: live.append,
        });
        chat.updateMessage(conversationId, replyId, {
          content: result.content,
          reasoning: result.reasoning || undefined,
          stats: result.stats,
        });
      } catch (error) {
        const partial = useLiveReply.getState();
        if (abort.signal.aborted) {
          chat.updateMessage(conversationId, replyId, {
            content: partial.content,
            reasoning: partial.reasoning || undefined,
            stopped: true,
          });
        } else {
          chat.updateMessage(conversationId, replyId, {
            content: partial.content,
            error: error instanceof Error ? error.message : String(error),
          });
        }
      } finally {
        controller.current = null;
        useLiveReply.getState().clear();
      }
    },
    [connection, models],
  );

  const stop = useCallback(() => controller.current?.abort(), []);

  return { send, stop, streaming, ready: Boolean(connection) };
}
