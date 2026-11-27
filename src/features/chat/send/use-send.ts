import { useQuery } from "@tanstack/react-query";
import { useCallback, useRef } from "react";
import { modelsQuery } from "@/features/library/api";
import { connectionQuery } from "@/features/server/api";
import type { DocumentContext, DocumentInfo } from "@/lib/api";
import { visibleMessages } from "../context-usage";
import { documentParts } from "../documents/lib/prompt";
import { buildChatRequest } from "../lib/request";
import { streamChat } from "../lib/sse";
import { newId, useChat, useLiveReply } from "../store";
import { readDocuments, sourcesOf } from "./documents";
import { fitContext } from "./fit-context";
import { stopAtSequences } from "./stop-sequences";
import { type PendingAttachment, toTurns, userMessage } from "./turns";

/**
 * Send a message in a conversation and stream the reply; stop it on demand.
 * In order: read the documents, fit the conversation in the model's window,
 * then stream the reply.
 */
export function useSend() {
  const { data: connection } = useQuery(connectionQuery);
  const { data: models = [] } = useQuery(modelsQuery);
  const controller = useRef<AbortController | null>(null);
  const streaming = useLiveReply((state) => state.messageId !== null);

  const send = useCallback(
    async (
      conversationId: string,
      text: string,
      pending: readonly PendingAttachment[] = [],
      attached: readonly DocumentInfo[] = [],
    ) => {
      const chat = useChat.getState();
      const conversation = chat.conversations.find((c) => c.id === conversationId);
      if (!conversation || !connection || controller.current) return;
      if (attached.length) chat.addDocuments(conversationId, [...attached]);
      const documents =
        useChat.getState().conversations.find((c) => c.id === conversationId)?.documents ?? [];

      const message = userMessage(text, pending, attached);
      let visible = visibleMessages([...conversation.messages, message], conversation.contextFrom);
      chat.addMessage(conversationId, message);
      const replyId = newId();
      chat.addMessage(conversationId, { id: replyId, role: "assistant", content: "" });

      const live = useLiveReply.getState();
      live.start(conversationId, replyId);
      const abort = new AbortController();
      controller.current = abort;

      const model = models.find((candidate) => candidate.name === conversation.modelId);
      const { settings } = chat;
      let summary = conversation.summary;
      let read: DocumentContext | null = null;
      const request = () =>
        buildChatRequest(
          model,
          conversation.modelId,
          settings,
          toTurns(visible),
          summary,
          documentParts(read),
        );
      const { onDelta, reachedStop } = stopAtSequences(settings.stop, abort);

      try {
        if (documents.length > 0) {
          live.setStatus("reading");
          read = await readDocuments({
            modelId: conversation.modelId,
            documents,
            text,
            earlier: conversation.messages,
            body: request(),
            maxTokens: settings.maxTokens,
          }).catch((error: unknown) => {
            // Without its documents, the model still answers.
            if (abort.signal.aborted) throw error;
            console.warn("couldn't read the documents", error);
            return null;
          });
          live.setStatus(null);
        }
        const fitted = await fitContext({
          connection,
          modelId: conversation.modelId,
          settings,
          visible,
          summary,
          body: request(),
          signal: abort.signal,
          onSummarizing: (summarizing) => live.setStatus(summarizing ? "summarizing" : null),
        });
        if (fitted) {
          ({ visible, summary } = fitted);
          chat.setContext(conversationId, { contextFrom: visible[0]?.id, summary });
        }

        const result = await streamChat({
          url: `${connection.baseUrl}/chat/completions`,
          token: connection.token,
          body: request(),
          signal: abort.signal,
          onDelta,
        });
        chat.updateMessage(conversationId, replyId, {
          content: result.content,
          reasoning: result.reasoning || undefined,
          stats: result.stats,
          ...sourcesOf(read),
        });
        // The gateway forgot more than counted here (an estimate was short).
        if (result.forgotten > 0) {
          chat.setContext(conversationId, { contextFrom: visible[result.forgotten]?.id, summary });
        }
      } catch (error) {
        const partial = useLiveReply.getState();
        const kept = { content: partial.content, reasoning: partial.reasoning || undefined };
        if (reachedStop()) {
          chat.updateMessage(conversationId, replyId, { ...kept, ...sourcesOf(read) });
        } else if (abort.signal.aborted) {
          chat.updateMessage(conversationId, replyId, {
            ...kept,
            stopped: true,
            ...sourcesOf(read),
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
