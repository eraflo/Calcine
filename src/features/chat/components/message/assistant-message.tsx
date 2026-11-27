import { Sources } from "../../documents";
import type { ChatMessage } from "../../store";
import { ErrorNote } from "./error-note";
import { LiveStatus } from "./live-status";
import { asMarkdown, MarkdownBody } from "./markdown-body";
import type { LiveReply, MessageActions } from "./message";
import { Reasoning } from "./reasoning";
import { ReplyFooter } from "./reply-footer";

/** The model's reply: reasoning, answer, sources, then its versions and actions. */
export function AssistantMessage({
  message,
  live,
  actions,
}: {
  message: ChatMessage;
  live?: LiveReply;
  actions?: MessageActions;
}) {
  const content = live?.content ?? message.content;
  const reasoning = live?.reasoning ?? message.reasoning ?? "";
  const thinking = Boolean(live) && !live?.content && Boolean(live?.reasoning);
  return (
    <div className="group flex flex-col gap-2">
      {reasoning && <Reasoning text={reasoning} active={thinking} />}
      {content ? (
        <MarkdownBody text={asMarkdown(content)} />
      ) : live && (live.status || !reasoning) ? (
        <LiveStatus status={live.status} />
      ) : null}
      {message.error && <ErrorNote error={message.error} />}
      {!live && message.sources && message.sources.length > 0 && (
        <Sources
          sources={message.sources}
          mode={message.documentMode}
          passages={message.passagesRead}
        />
      )}
      {!live && <ReplyFooter message={message} actions={actions} />}
    </div>
  );
}
