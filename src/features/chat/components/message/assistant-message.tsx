import { CopyButton } from "@/components/calcine/actions/copy-button";
import { useT } from "@/i18n";
import { Sources } from "../../documents";
import { messages } from "../../messages";
import type { ChatMessage } from "../../store";
import { ErrorNote } from "./error-note";
import { LiveStatus } from "./live-status";
import { asMarkdown, MarkdownBody } from "./markdown-body";
import type { LiveReply } from "./message";
import { Reasoning } from "./reasoning";
import { Stats } from "./stats";

/** The model's reply: reasoning, answer, sources and speed. */
export function AssistantMessage({ message, live }: { message: ChatMessage; live?: LiveReply }) {
  const t = useT(messages);
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
      {!live && (message.stats || message.stopped) && (
        <div className="flex items-center gap-2 text-[11px] text-muted-foreground">
          {message.stopped && <span>{t("stopped")}</span>}
          {message.stats && <Stats stats={message.stats} />}
          {content && (
            <CopyButton
              value={content}
              className="opacity-0 transition-opacity group-hover:opacity-100"
            />
          )}
        </div>
      )}
    </div>
  );
}
