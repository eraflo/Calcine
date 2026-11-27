import { ArrowUp, Loader2, Square } from "lucide-react";
import {
  type ClipboardEvent,
  type FormEvent,
  type KeyboardEvent,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import { Button } from "@/components/ui/button";
import { Tooltip } from "@/components/ui/tooltip";
import { useT } from "@/i18n";
import { type Attachments, type DocumentInfo, documentMessages } from "../../documents";
import { insertDictation } from "../../lib/dictation";
import { useDictation } from "../../lib/use-dictation";
import { messages } from "../../messages";
import type { PendingAttachment } from "../../send";
import { AttachButton } from "./attach-button";
import { AttachmentList } from "./attachment-list";
import { DictateButton } from "./dictate-button";
import { DocumentActions } from "./document-actions";

/**
 * Message box: Enter sends, Shift+Enter adds a line. Documents go to every
 * model, images to vision models. Dictation writes what you say into the
 * box, whatever the model.
 */
export function Composer({
  disabled,
  disabledReason,
  streaming,
  media,
  attachments,
  problem,
  onProblem,
  onSend,
  onStop,
}: {
  disabled: boolean;
  disabledReason?: string;
  streaming: boolean;
  /** The model takes images and audio. */
  media: boolean;
  attachments: Attachments;
  /** Something couldn't be attached or dictated. */
  problem: string | null;
  onProblem: (problem: string | null) => void;
  onSend: (text: string, images: PendingAttachment[], documents: DocumentInfo[]) => void;
  onStop: () => void;
}) {
  const t = useT(messages);
  const td = useT(documentMessages);
  const [text, setText] = useState("");
  const area = useRef<HTMLTextAreaElement>(null);

  // Dictated words go where the caret is.
  const onDictated = useCallback((spoken: string) => {
    const element = area.current;
    setText((current) => {
      const start = element?.selectionStart ?? current.length;
      const end = element?.selectionEnd ?? current.length;
      const { value, caret } = insertDictation(current, spoken, start, end);
      requestAnimationFrame(() => {
        element?.focus();
        element?.setSelectionRange(caret, caret);
      });
      return value;
    });
  }, []);
  const dictation = useDictation({ onText: onDictated, onProblem });
  // Nothing is sent while the microphone or a transcription is busy.
  const busy = dictation.state.status !== "idle";

  // Grow with the content, up to a limit.
  // biome-ignore lint/correctness/useExhaustiveDependencies: resize whenever the text changes.
  useEffect(() => {
    const element = area.current;
    if (!element) return;
    element.style.height = "auto";
    element.style.height = `${Math.min(element.scrollHeight, 200)}px`;
  }, [text]);

  // Images can't go to a text-only model.
  const { dropImages } = attachments;
  useEffect(() => {
    if (!media) dropImages();
  }, [media, dropImages]);

  const { images, ready: documents, reading } = attachments;
  const canSend = Boolean(text.trim() || images.length || documents.length);
  const blocked = disabled || streaming || busy || reading;

  const send = (message: string) => {
    if (blocked) return;
    onSend(message, images, documents);
    setText("");
    attachments.clear();
    onProblem(null);
  };
  const submit = (event?: FormEvent) => {
    event?.preventDefault();
    if (!canSend) return;
    // Documents alone: summarize them.
    send(text.trim() || (documents.length ? td.plural("docSummarize", documents.length) : ""));
  };

  const onKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.key === "Enter" && !event.shiftKey && !event.nativeEvent.isComposing) {
      event.preventDefault();
      submit();
    }
  };

  const onPaste = (event: ClipboardEvent<HTMLTextAreaElement>) => {
    if (!media) return;
    const pasted = [...event.clipboardData.files].filter((file) => file.type.startsWith("image/"));
    if (pasted.length === 0) return;
    event.preventDefault();
    void attachments.addImageFiles(pasted);
  };

  const placeholder = disabled
    ? (disabledReason ?? t("pickModelFirst"))
    : documents.length
      ? td("placeholderDocuments")
      : t("placeholder");
  const sendLabel = reading ? td("waitForDocuments") : t("send");
  return (
    <form onSubmit={submit} className="mx-auto w-full max-w-3xl px-6 pb-5">
      {documents.length > 0 && !text.trim() && !reading && (
        <DocumentActions count={documents.length} disabled={blocked} onSend={send} />
      )}
      <div className="flex flex-col gap-2 rounded-xl border bg-card p-2 shadow-xs transition-shadow focus-within:ring-2 focus-within:ring-ring">
        <AttachmentList attachments={attachments} />
        <div className="flex items-end gap-1.5">
          <AttachButton
            media={media}
            disabled={disabled || streaming || busy}
            onPick={() => void attachments.pick().then(() => area.current?.focus())}
          />
          <textarea
            ref={area}
            value={text}
            onChange={(event) => setText(event.target.value)}
            onKeyDown={onKeyDown}
            onPaste={onPaste}
            rows={1}
            placeholder={placeholder}
            disabled={disabled}
            aria-label={t("message")}
            className="max-h-[200px] min-h-9 flex-1 resize-none bg-transparent px-2 py-2 text-sm outline-none placeholder:text-muted-foreground disabled:cursor-not-allowed"
          />
          <DictateButton
            state={dictation.state}
            available={dictation.available}
            disabled={disabled}
            onToggle={dictation.toggle}
          />
          {streaming ? (
            <Button type="button" size="icon" onClick={onStop} aria-label={t("stop")}>
              <Square className="fill-current" />
            </Button>
          ) : (
            <Tooltip content={sendLabel}>
              <span>
                <Button
                  type="submit"
                  size="icon"
                  variant="default"
                  disabled={blocked || !canSend}
                  aria-label={sendLabel}
                >
                  {reading ? <Loader2 className="animate-spin" /> : <ArrowUp />}
                </Button>
              </span>
            </Tooltip>
          )}
        </div>
      </div>
      {problem && <p className="mt-1.5 px-1 text-xs text-destructive">{problem}</p>}
    </form>
  );
}
