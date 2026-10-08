import { useEffect, useRef } from "react";
import { LogoMark } from "@/components/calcine/brand/logo";
import { type Conversation, useLiveReply } from "../store";
import { Message } from "./message";

const SUGGESTIONS = [
  "Explain what an NPU is, in two sentences.",
  "Write a haiku about a laptop that runs AI offline.",
  "Give me three ideas for a weekend project.",
];

/** The messages of a conversation, following the reply as it streams. */
export function Thread({
  conversation,
  onSuggestion,
}: {
  conversation: Conversation | undefined;
  onSuggestion: (text: string) => void;
}) {
  const live = useLiveReply();
  const bottom = useRef<HTMLDivElement>(null);
  const messages = conversation?.messages ?? [];
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
          <h2 className="text-lg font-semibold tracking-tight">What's on your mind?</h2>
          <p className="text-sm text-muted-foreground">
            Everything runs on this device. Nothing leaves your PC.
          </p>
        </div>
        <div className="flex max-w-xl flex-wrap justify-center gap-2">
          {SUGGESTIONS.map((suggestion) => (
            <button
              key={suggestion}
              type="button"
              onClick={() => onSuggestion(suggestion)}
              className="rounded-full border bg-card px-3 py-1.5 text-xs text-muted-foreground transition-colors hover:border-muted-foreground/40 hover:text-foreground"
            >
              {suggestion}
            </button>
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-0 flex-1 overflow-y-auto overscroll-contain">
      <div className="mx-auto flex max-w-3xl flex-col gap-6 px-6 py-6">
        {messages.map((message) => (
          <Message
            key={message.id}
            message={message}
            live={live.messageId === message.id ? live : undefined}
          />
        ))}
        <div ref={bottom} />
      </div>
    </div>
  );
}
