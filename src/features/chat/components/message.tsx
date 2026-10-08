import * as Collapsible from "@radix-ui/react-collapsible";
import { Brain, ChevronRight, CircleAlert } from "lucide-react";
import { memo, useState } from "react";
import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { CopyButton } from "@/components/calcine/actions/copy-button";
import { cn } from "@/lib/utils";
import type { StreamStats } from "../lib/sse";
import type { ChatMessage } from "../store";

/** One turn. `live` is the reply still being generated. */
export function Message({
  message,
  live,
}: {
  message: ChatMessage;
  live?: { content: string; reasoning: string };
}) {
  const content = live?.content ?? message.content;
  const reasoning = live?.reasoning ?? message.reasoning ?? "";

  if (message.role === "user") {
    return (
      <div className="flex justify-end">
        <div className="max-w-[80%] rounded-lg bg-secondary px-3.5 py-2 text-sm whitespace-pre-wrap">
          {content}
        </div>
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
          {message.error}
        </p>
      )}
      {!live && (message.stats || message.stopped) && (
        <div className="flex items-center gap-2 text-[11px] text-muted-foreground">
          {message.stopped && <span>Stopped</span>}
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

function Reasoning({ text, active }: { text: string; active: boolean }) {
  const [open, setOpen] = useState(false);
  return (
    <Collapsible.Root open={open || active} onOpenChange={setOpen}>
      <Collapsible.Trigger className="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground">
        <Brain className={cn("size-3.5", active && "animate-pulse text-primary")} />
        {active ? "Thinking…" : "Reasoning"}
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
  const parts = [
    stats.completionTokens !== undefined && `${stats.completionTokens} tokens`,
    stats.tokensPerSecond && `${stats.tokensPerSecond.toFixed(0)} tok/s`,
    stats.firstTokenMs !== undefined && `first token ${(stats.firstTokenMs / 1000).toFixed(1)}s`,
  ].filter(Boolean);
  return <span className="tabular-nums">{parts.join(" · ")}</span>;
}

function TypingDots() {
  return (
    <span className="flex gap-1 py-2" role="status" aria-label="Generating">
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
