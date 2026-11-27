import type { ChatMessage } from "../../store";
import { AssistantMessage } from "./assistant-message";
import { UserMessage } from "./user-message";

/** The reply being generated, token by token. */
export type LiveReply = {
  content: string;
  reasoning: string;
  status?: "summarizing" | "reading" | null;
};

/** What can be done with a message; left out while a reply is written. */
export type MessageActions = {
  /** Write this reply again (the conversation's last). */
  onRegenerate?: () => void;
  /** Show another version of this reply. */
  onSelectVersion?: (index: number) => void;
  /** Change this user message and answer it again. */
  onEdit?: (text: string) => void;
  /** Replies follow this message: editing it replaces them. */
  hasFollowing?: boolean;
};

/** One turn. `live` is the reply still being generated. */
export function Message({
  message,
  live,
  actions,
}: {
  message: ChatMessage;
  live?: LiveReply;
  actions?: MessageActions;
}) {
  return message.role === "user" ? (
    <UserMessage message={message} actions={actions} />
  ) : (
    <AssistantMessage message={message} live={live} actions={actions} />
  );
}
