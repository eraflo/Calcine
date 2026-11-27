/** Changing one conversation in the list. */

import type { Conversation } from "./types";

/** `conversations` with the one with this id changed by `update`. */
export function mapConversation(
  conversations: Conversation[],
  id: string,
  update: (conversation: Conversation) => Conversation,
): Conversation[] {
  return conversations.map((conversation) =>
    conversation.id === id ? update(conversation) : conversation,
  );
}
