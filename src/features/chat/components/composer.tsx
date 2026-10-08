import { ArrowUp, Square } from "lucide-react";
import { type FormEvent, type KeyboardEvent, useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button";

/** Message box: Enter sends, Shift+Enter adds a line. */
export function Composer({
  disabled,
  disabledReason,
  streaming,
  onSend,
  onStop,
}: {
  disabled: boolean;
  disabledReason?: string;
  streaming: boolean;
  onSend: (text: string) => void;
  onStop: () => void;
}) {
  const [text, setText] = useState("");
  const area = useRef<HTMLTextAreaElement>(null);

  // Grow with the content, up to a limit.
  // biome-ignore lint/correctness/useExhaustiveDependencies: resize whenever the text changes.
  useEffect(() => {
    const element = area.current;
    if (!element) return;
    element.style.height = "auto";
    element.style.height = `${Math.min(element.scrollHeight, 200)}px`;
  }, [text]);

  const submit = (event?: FormEvent) => {
    event?.preventDefault();
    const message = text.trim();
    if (!message || disabled || streaming) return;
    onSend(message);
    setText("");
  };

  const onKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.key === "Enter" && !event.shiftKey && !event.nativeEvent.isComposing) {
      event.preventDefault();
      submit();
    }
  };

  return (
    <form onSubmit={submit} className="mx-auto w-full max-w-3xl px-6 pb-5">
      <div className="flex items-end gap-2 rounded-lg border bg-card p-2 focus-within:ring-2 focus-within:ring-ring">
        <textarea
          ref={area}
          value={text}
          onChange={(event) => setText(event.target.value)}
          onKeyDown={onKeyDown}
          rows={1}
          placeholder={
            disabled ? (disabledReason ?? "Pick a model to start") : "Message your model"
          }
          disabled={disabled}
          aria-label="Message"
          className="max-h-[200px] min-h-9 flex-1 resize-none bg-transparent px-2 py-2 text-sm outline-none placeholder:text-muted-foreground disabled:cursor-not-allowed"
        />
        {streaming ? (
          <Button type="button" size="icon" onClick={onStop} aria-label="Stop generating">
            <Square className="fill-current" />
          </Button>
        ) : (
          <Button
            type="submit"
            size="icon"
            variant="default"
            disabled={disabled || !text.trim()}
            aria-label="Send"
          >
            <ArrowUp />
          </Button>
        )}
      </div>
    </form>
  );
}
