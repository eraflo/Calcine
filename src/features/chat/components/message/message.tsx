import type { ChatMessage } from "../../store";
import { AssistantMessage } from "./assistant-message";
import { UserMessage } from "./user-message";

/** The reply being generated, token by token. */
export type LiveReply = {
  content: string;
  reasoning: string;
  status?: "summarizing" | "reading" | null;
};

/** One turn. `live` is the reply still being generated. */
export function Message({ message, live }: { message: ChatMessage; live?: LiveReply }) {
  return message.role === "user" ? (
    <UserMessage message={message} />
  ) : (
    <AssistantMessage message={message} live={live} />
  );
}
