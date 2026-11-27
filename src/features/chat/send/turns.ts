/** Messages as the API expects them, and the user's new message. */

import type { DocumentInfo } from "@/lib/api";
import { visibleMessages } from "../context-usage";
import type { Attachment } from "../lib/attachments";
import type { ChatTurn, MediaPart } from "../lib/request";
import { type ChatMessage, newId, useMediaPayloads } from "../store";

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

/** The user's message, with what's attached to it. Media data is kept in
 * memory, apart from the saved conversation. */
export function userMessage(
  text: string,
  pending: readonly PendingAttachment[],
  documents: readonly DocumentInfo[],
): ChatMessage {
  useMediaPayloads.setState(
    Object.fromEntries(pending.map(({ attachment, payload }) => [attachment.id, payload])),
  );
  return {
    id: newId(),
    role: "user",
    content: text,
    ...(pending.length ? { attachments: pending.map(({ attachment }) => attachment) } : {}),
    ...(documents.length ? { documents: [...documents] } : {}),
  };
}
