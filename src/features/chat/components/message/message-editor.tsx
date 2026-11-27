import { type FormEvent, type KeyboardEvent, useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import { useT } from "@/i18n";
import { messages } from "../../messages";

/** A user message being edited: Enter sends it again, Escape cancels. */
export function MessageEditor({
  initial,
  replacesFollowing,
  onSend,
  onCancel,
}: {
  initial: string;
  /** Replies come after: say they'll be replaced. */
  replacesFollowing: boolean;
  onSend: (text: string) => void;
  onCancel: () => void;
}) {
  const t = useT(messages);
  const [text, setText] = useState(initial);
  const area = useRef<HTMLTextAreaElement>(null);

  // Grow with the text; start with the caret at the end.
  // biome-ignore lint/correctness/useExhaustiveDependencies: resize whenever the text changes.
  useEffect(() => {
    const element = area.current;
    if (!element) return;
    element.style.height = "auto";
    element.style.height = `${Math.min(element.scrollHeight, 240)}px`;
  }, [text]);
  useEffect(() => {
    const element = area.current;
    element?.focus();
    element?.setSelectionRange(element.value.length, element.value.length);
  }, []);

  const changed = text.trim() !== "" && text.trim() !== initial.trim();
  const submit = (event?: FormEvent) => {
    event?.preventDefault();
    if (changed) onSend(text);
  };
  const onKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.key === "Escape") {
      event.preventDefault();
      onCancel();
    } else if (event.key === "Enter" && !event.shiftKey && !event.nativeEvent.isComposing) {
      event.preventDefault();
      submit();
    }
  };

  return (
    <form
      onSubmit={submit}
      className="flex w-full max-w-[80%] flex-col gap-2 rounded-lg border bg-card p-2 focus-within:ring-2 focus-within:ring-ring"
    >
      <textarea
        ref={area}
        value={text}
        onChange={(event) => setText(event.target.value)}
        onKeyDown={onKeyDown}
        rows={1}
        aria-label={t("editMessage")}
        className="max-h-60 min-h-9 resize-none bg-transparent px-1.5 py-1 text-sm outline-none"
      />
      <div className="flex items-center gap-2">
        {replacesFollowing && (
          <span className="text-[11px] text-muted-foreground">{t("editReplaces")}</span>
        )}
        <span className="ml-auto flex gap-1.5">
          <Button type="button" variant="ghost" size="sm" className="h-7" onClick={onCancel}>
            {t("editCancel")}
          </Button>
          <Button type="submit" size="sm" className="h-7" disabled={!changed}>
            {t("send")}
          </Button>
        </span>
      </div>
    </form>
  );
}
