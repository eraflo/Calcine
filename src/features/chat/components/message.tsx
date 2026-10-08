import * as Collapsible from "@radix-ui/react-collapsible";
import { Brain, ChevronRight, CircleAlert, ImageOff, Mic } from "lucide-react";
import { memo, useState } from "react";
import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { CopyButton } from "@/components/calcine/actions/copy-button";
import { Tooltip } from "@/components/ui/tooltip";
import { formatDuration } from "@/features/tasks/lib/format";
import { useT } from "@/i18n";
import { formatNumber } from "@/lib/format";
import { cn } from "@/lib/utils";
import type { Attachment } from "../lib/attachments";
import type { StreamStats } from "../lib/sse";
import { messages } from "../messages";
import { type ChatMessage, useMediaPayloads } from "../store";

/** One turn. `live` is the reply still being generated. */
export function Message({
  message,
  live,
}: {
  message: ChatMessage;
  live?: { content: string; reasoning: string };
}) {
  const t = useT(messages);
  const content = live?.content ?? message.content;
  const reasoning = live?.reasoning ?? message.reasoning ?? "";

  if (message.role === "user") {
    return (
      <div className="flex flex-col items-end gap-1.5">
        {message.attachments && message.attachments.length > 0 && (
          <div className="flex max-w-[80%] flex-wrap justify-end gap-1.5">
            {message.attachments.map((attachment) => (
              <AttachmentPreview key={attachment.id} attachment={attachment} />
            ))}
          </div>
        )}
        {content && (
          <div className="max-w-[80%] rounded-lg bg-secondary px-3.5 py-2 text-sm whitespace-pre-wrap">
            {content}
          </div>
        )}
      </div>
    );
  }

  const thinking = Boolean(live) && !live?.content && Boolean(live?.reasoning);
  return (
    <div className="group flex flex-col gap-2">
      {reasoning && <Reasoning text={reasoning} active={thinking} />}
      {content ? <MarkdownBody text={content} /> : live && !reasoning ? <TypingDots /> : null}
      {message.error && (
        <p className="flex items-start gap-1.5 text-sm text-destructive">
          <CircleAlert className="mt-0.5 size-4 shrink-0" />
          <span>
            {message.error}
            {message.error.includes("Model loading failed") && (
              <span className="mt-1 block text-xs text-muted-foreground">
                {t("loadFailedHint")}
              </span>
            )}
          </span>
        </p>
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

/** An image or recording sent with a message. Data isn't kept across restarts. */
function AttachmentPreview({ attachment }: { attachment: Attachment }) {
  const t = useT(messages);
  const available = useMediaPayloads((payloads) => attachment.id in payloads);
  const preview =
    attachment.kind === "image" ? (
      attachment.thumbnail ? (
        <img
          src={attachment.thumbnail}
          alt={attachment.name}
          className={cn("h-24 max-w-48 rounded-md border object-cover", !available && "opacity-60")}
        />
      ) : (
        <span className="flex h-12 items-center gap-1.5 rounded-md border px-3 text-xs text-muted-foreground">
          <ImageOff className="size-3.5" />
          {attachment.name}
        </span>
      )
    ) : (
      <span className="flex h-9 items-center gap-1.5 rounded-md border bg-secondary px-3 text-xs">
        <Mic className="size-3.5 text-primary" />
        {formatDuration(attachment.durationSeconds ?? 0)}
      </span>
    );
  return available ? preview : <Tooltip content={t("attachmentGone")}>{preview}</Tooltip>;
}

function Reasoning({ text, active }: { text: string; active: boolean }) {
  const t = useT(messages);
  const [open, setOpen] = useState(false);
  return (
    <Collapsible.Root open={open || active} onOpenChange={setOpen}>
      <Collapsible.Trigger className="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground">
        <Brain className={cn("size-3.5", active && "animate-pulse text-primary")} />
        {active ? t("thinking") : t("reasoning")}
        <ChevronRight
          className={cn("size-3.5 transition-transform", (open || active) && "rotate-90")}
        />
      </Collapsible.Trigger>
      <Collapsible.Content>
        <p className="mt-1.5 border-l-2 pl-3 text-xs leading-relaxed whitespace-pre-wrap text-muted-foreground">
          {text.trim()}
        </p>
      </Collapsible.Content>
    </Collapsible.Root>
  );
}

const MarkdownBody = memo(function MarkdownBody({ text }: { text: string }) {
  return (
    <div className="markdown text-sm leading-relaxed">
      <Markdown remarkPlugins={[remarkGfm]}>{text}</Markdown>
    </div>
  );
});

function Stats({ stats }: { stats: StreamStats }) {
  const t = useT(messages);
  const parts = [
    stats.completionTokens !== undefined && t.plural("tokens", stats.completionTokens),
    stats.tokensPerSecond &&
      t("tokensPerSecond", { value: formatNumber(stats.tokensPerSecond, 0) }),
    stats.firstTokenMs !== undefined &&
      t("firstToken", { value: formatNumber(stats.firstTokenMs / 1000, 1) }),
    stats.draftTokens &&
      t("draftsAccepted", {
        accepted: String(stats.draftAccepted ?? 0),
        drafted: String(stats.draftTokens),
        percent: formatNumber(((stats.draftAccepted ?? 0) / stats.draftTokens) * 100, 0),
      }),
  ].filter(Boolean);
  return <span className="tabular-nums">{parts.join(" · ")}</span>;
}

function TypingDots() {
  const t = useT(messages);
  return (
    <span className="flex gap-1 py-2" role="status" aria-label={t("generating")}>
      {[0, 150, 300].map((delay) => (
        <span
          key={delay}
          className="size-1.5 animate-pulse rounded-full bg-muted-foreground"
          style={{ animationDelay: `${delay}ms` }}
        />
      ))}
    </span>
  );
}
