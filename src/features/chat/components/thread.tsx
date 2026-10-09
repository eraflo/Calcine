import * as Collapsible from "@radix-ui/react-collapsible";
import { ChevronRight, EyeOff } from "lucide-react";
import { Fragment, useEffect, useRef, useState } from "react";
import { LogoMark } from "@/components/calcine/brand/logo";
import { useT } from "@/i18n";
import { cn } from "@/lib/utils";
import { messages as strings } from "../messages";
import { type Conversation, useLiveReply } from "../store";
import { Message } from "./message";

const SUGGESTIONS = ["suggestion1", "suggestion2", "suggestion3"] as const;

/** The messages of a conversation, following the reply as it streams. */
export function Thread({
  conversation,
  onSuggestion,
}: {
  conversation: Conversation | undefined;
  onSuggestion: (text: string) => void;
}) {
  const t = useT(strings);
  const live = useLiveReply();
  const bottom = useRef<HTMLDivElement>(null);
  const messages = conversation?.messages ?? [];
  // Messages before this one no longer fit the model's context window.
  const seenFrom = conversation?.contextFrom
    ? messages.findIndex((message) => message.id === conversation.contextFrom)
    : -1;
  const liveLength = live.content.length + live.reasoning.length;

  // biome-ignore lint/correctness/useExhaustiveDependencies: scroll on every new message or token.
  useEffect(() => {
    bottom.current?.scrollIntoView({ block: "end" });
  }, [messages.length, liveLength]);

  if (messages.length === 0) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-5 px-6 text-center">
        <LogoMark className="size-10" />
        <div className="flex flex-col gap-1">
          <h2 className="text-lg font-semibold tracking-tight">{t("emptyTitle")}</h2>
          <p className="text-sm text-muted-foreground">{t("emptyHint")}</p>
        </div>
        <div className="flex max-w-xl flex-wrap justify-center gap-2">
          {SUGGESTIONS.map((key) => (
            <button
              key={key}
              type="button"
              onClick={() => onSuggestion(t(key))}
              className="rounded-full border bg-card px-3 py-1.5 text-xs text-muted-foreground transition-colors hover:border-muted-foreground/40 hover:text-foreground"
            >
              {t(key)}
            </button>
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-0 flex-1 overflow-y-auto overscroll-contain">
      <div className="mx-auto flex max-w-3xl flex-col gap-6 px-6 py-6">
        {messages.map((message, index) => (
          <Fragment key={message.id}>
            {index === seenFrom && index > 0 && (
              <ForgottenDivider summary={conversation?.summary} />
            )}
            <div className={cn(index < seenFrom && "opacity-50")}>
              <Message message={message} live={live.messageId === message.id ? live : undefined} />
            </div>
          </Fragment>
        ))}
        <div ref={bottom} />
      </div>
    </div>
  );
}

/** Where the model's view starts, with the summary of what's above. */
function ForgottenDivider({ summary }: { summary: string | undefined }) {
  const t = useT(strings);
  const [open, setOpen] = useState(false);
  const line = (
    <span className="flex w-full items-center gap-3 text-[11px] text-muted-foreground before:h-px before:flex-1 before:bg-border after:h-px after:flex-1 after:bg-border">
      <EyeOff className="size-3.5" />
      {summary ? t("forgottenSummarized") : t("forgottenDivider")}
      {summary && (
        <ChevronRight className={cn("size-3.5 transition-transform", open && "rotate-90")} />
      )}
    </span>
  );
  if (!summary) return <p>{line}</p>;
  return (
    <Collapsible.Root open={open} onOpenChange={setOpen}>
      <Collapsible.Trigger className="w-full hover:text-foreground">{line}</Collapsible.Trigger>
      <Collapsible.Content>
        <p className="mx-auto mt-2 max-w-xl rounded-md border bg-card px-3 py-2 text-xs leading-relaxed whitespace-pre-wrap text-muted-foreground">
          {summary}
        </p>
      </Collapsible.Content>
    </Collapsible.Root>
  );
}
