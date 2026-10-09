import { useQuery } from "@tanstack/react-query";
import { useCallback, useRef } from "react";
import { modelsQuery } from "@/features/library/api";
import { connectionQuery } from "@/features/server/api";
import { fetchContextUsage, fittedStart, summarize, visibleMessages } from "./context-usage";
import type { Attachment } from "./lib/attachments";
import { buildChatRequest, type ChatTurn, type MediaPart, stopIndex } from "./lib/request";
import { streamChat } from "./lib/sse";
import { SUMMARY_MAX_TOKENS } from "./lib/summary";
import { type ChatMessage, newId, useChat, useLiveReply, useMediaPayloads } from "./store";

/** An attachment ready to send: what the thread shows and what the model gets. */
export type PendingAttachment = { attachment: Attachment; payload: MediaPart };

/** History as the API expects it. Attachments whose data is gone are left out. */
export function toTurns(messages: readonly ChatMessage[]): ChatTurn[] {
  const payloads = useMediaPayloads.getState();
  return visibleMessages(messages, undefined).map(({ role, content, attachments }) => ({
    role,
    content,
    media: attachments?.flatMap((attachment) => payloads[attachment.id] ?? []),
  }));
}

/** Send a message in a conversation and stream the reply; stop it on demand. */
export function useSend() {
  const { data: connection } = useQuery(connectionQuery);
  const { data: models = [] } = useQuery(modelsQuery);
  const controller = useRef<AbortController | null>(null);
  const streaming = useLiveReply((state) => state.messageId !== null);

  const send = useCallback(
    async (conversationId: string, text: string, pending: readonly PendingAttachment[] = []) => {
      const chat = useChat.getState();
      const conversation = chat.conversations.find((c) => c.id === conversationId);
      if (!conversation || !connection || controller.current) return;

      useMediaPayloads.setState(
        Object.fromEntries(pending.map(({ attachment, payload }) => [attachment.id, payload])),
      );
      const userMessage: ChatMessage = {
        id: newId(),
        role: "user",
        content: text,
        ...(pending.length ? { attachments: pending.map(({ attachment }) => attachment) } : {}),
      };
      let visible = visibleMessages(
        [...conversation.messages, userMessage],
        conversation.contextFrom,
      );

      chat.addMessage(conversationId, userMessage);
      const replyId = newId();
      chat.addMessage(conversationId, { id: replyId, role: "assistant", content: "" });

      const live = useLiveReply.getState();
      live.start(conversationId, replyId);
      const abort = new AbortController();
      controller.current = abort;

      const model = models.find((candidate) => candidate.name === conversation.modelId);
      const { settings } = chat;
      let summary = conversation.summary;
      const request = () =>
        buildChatRequest(model, conversation.modelId, settings, toTurns(visible), summary);
      // Too long for the model's context window: stop showing it the oldest
      // messages, enough of them that the start stays put for a few turns,
      // and have the model summarize them.
      const fitContext = async () => {
        if (!settings.forgetOldest) return;
        const summarizing = settings.summarizeForgotten;
        const start = await fetchContextUsage(conversation.modelId, request())
          .then((usage) =>
            fittedStart(usage, visible, settings.maxTokens, summarizing ? SUMMARY_MAX_TOKENS : 0),
          )
          .catch(() => 0);
        if (start === 0) return;
        const forgotten = visible.slice(0, start);
        visible = visible.slice(start);
        if (summarizing) {
          live.setStatus("summarizing");
          summary = await summarize({
            connection,
            model: conversation.modelId,
            systemPrompt: settings.systemPrompt,
            previous: summary,
            forgotten,
            signal: abort.signal,
          }).catch((error: unknown) => {
            // Without a new summary the oldest messages are only forgotten.
            if (abort.signal.aborted) throw error;
            return summary;
          });
          live.setStatus(null);
        }
        chat.setContext(conversationId, { contextFrom: visible[0]?.id, summary });
      };
      // GenieX doesn't apply stop sequences sent over the API: cut the reply here.
      const stops = chat.settings.stop;
      let reachedStop = false;
      const onDelta = (delta: { content?: string; reasoning?: string }) => {
        live.append(delta);
        if (!delta.content || stops.length === 0) return;
        const { content } = useLiveReply.getState();
        const index = stopIndex(content, stops);
        if (index === null) return;
        reachedStop = true;
        useLiveReply.setState({ content: content.slice(0, index) });
        abort.abort();
      };
      try {
        await fitContext();
        const body = request();
        const result = await streamChat({
          url: `${connection.baseUrl}/chat/completions`,
          token: connection.token,
          body,
          signal: abort.signal,
          onDelta,
        });
        chat.updateMessage(conversationId, replyId, {
          content: result.content,
          reasoning: result.reasoning || undefined,
          stats: result.stats,
        });
        // The gateway forgot more than counted here (an estimate was short).
        if (result.forgotten > 0) {
          chat.setContext(conversationId, { contextFrom: visible[result.forgotten]?.id, summary });
        }
      } catch (error) {
        const partial = useLiveReply.getState();
        if (reachedStop) {
          chat.updateMessage(conversationId, replyId, {
            content: partial.content,
            reasoning: partial.reasoning || undefined,
          });
        } else if (abort.signal.aborted) {
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
