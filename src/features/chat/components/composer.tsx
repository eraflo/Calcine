import { ArrowUp, Loader2, Mic, Paperclip, Sparkles, Square, X } from "lucide-react";
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
import { formatDuration } from "@/features/tasks/lib/format";
import { useT } from "@/i18n";
import { cn } from "@/lib/utils";
import { insertDictation } from "../lib/dictation";
import type { DocumentInfo } from "../lib/documents";
import type { Attachments } from "../lib/use-attachments";
import { type DictationState, useDictation } from "../lib/use-dictation";
import { messages } from "../messages";
import type { PendingAttachment } from "../use-send";
import { DocumentChip } from "./document-chip";

/** Quick requests offered once documents are attached. */
const DOCUMENT_ACTIONS = ["docSummarize", "docKeyPoints", "docFigures"] as const;

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

  const { images, documents, ready: readyDocuments, reading } = attachments;
  const hasDocuments = readyDocuments.length > 0;
  const canSend = Boolean(text.trim() || images.length || hasDocuments);
  const blocked = disabled || streaming || busy || reading;

  const send = (message: string) => {
    if (blocked) return;
    onSend(message, images, readyDocuments);
    setText("");
    attachments.clear();
    onProblem(null);
  };
  const submit = (event?: FormEvent) => {
    event?.preventDefault();
    if (!canSend) return;
    // Documents alone: summarize them.
    send(text.trim() || (hasDocuments ? t.plural("docSummarize", readyDocuments.length) : ""));
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

  const showActions = hasDocuments && !text.trim() && !reading;
  return (
    <form onSubmit={submit} className="mx-auto w-full max-w-3xl px-6 pb-5">
      {showActions && (
        <div className="mb-2 flex flex-wrap items-center gap-1.5 px-1 animate-in fade-in-0 slide-in-from-bottom-1">
          <Sparkles className="size-3.5 text-primary" />
          {DOCUMENT_ACTIONS.map((key) => {
            const label = t.plural(key, readyDocuments.length);
            return (
              <button
                key={key}
                type="button"
                disabled={blocked}
                onClick={() => send(label)}
                className="rounded-full border bg-card px-3 py-1 text-xs text-muted-foreground transition-colors hover:border-primary/40 hover:text-foreground disabled:opacity-50"
              >
                {label}
              </button>
            );
          })}
        </div>
      )}
      <div className="flex flex-col gap-2 rounded-xl border bg-card p-2 shadow-xs transition-shadow focus-within:ring-2 focus-within:ring-ring">
        {(images.length > 0 || documents.length > 0) && (
          <ul className="flex flex-wrap gap-2 px-1 pt-1">
            {documents.map((item) => (
              <li key={item.key}>
                <DocumentChip
                  name={item.name}
                  state={item}
                  onRemove={() => attachments.removeDocument(item.key)}
                />
              </li>
            ))}
            {images.map(({ attachment }) => (
              <li
                key={attachment.id}
                className="group relative flex h-12 items-center overflow-hidden rounded-lg border bg-background"
              >
                <img
                  src={attachment.thumbnail}
                  alt={attachment.name}
                  className="h-full w-14 object-cover"
                />
                <button
                  type="button"
                  onClick={() => attachments.removeImage(attachment.id)}
                  className="absolute top-0.5 right-0.5 rounded-full bg-background/80 p-0.5 text-muted-foreground hover:text-foreground"
                  aria-label={t("removeAttachment", { name: attachment.name })}
                >
                  <X className="size-3" />
                </button>
              </li>
            ))}
          </ul>
        )}
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
            placeholder={
              disabled
                ? (disabledReason ?? t("pickModelFirst"))
                : hasDocuments
                  ? t("placeholderDocuments")
                  : t("placeholder")
            }
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
            <Tooltip content={reading ? t("waitForDocuments") : t("send")}>
              <span>
                <Button
                  type="submit"
                  size="icon"
                  variant="default"
                  disabled={blocked || !canSend}
                  aria-label={reading ? t("waitForDocuments") : t("send")}
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

/** Attach documents, and images for vision models. */
function AttachButton({
  media,
  disabled,
  onPick,
}: {
  media: boolean;
  disabled: boolean;
  onPick: () => void;
}) {
  const t = useT(messages);
  const label = media ? t("attachDocumentsOrImages") : t("attachDocuments");
  return (
    <Tooltip content={label}>
      <span>
        <Button
          type="button"
          size="icon"
          variant="ghost"
          className="size-9"
          onClick={onPick}
          disabled={disabled}
          aria-label={label}
        >
          <Paperclip />
        </Button>
      </span>
    </Tooltip>
  );
}

/** Record, then write down what was said. Shown for every model. */
function DictateButton({
  state,
  available,
  disabled,
  onToggle,
}: {
  state: DictationState;
  available: boolean;
  disabled: boolean;
  onToggle: () => void;
}) {
  const t = useT(messages);
  const recording = state.status === "recording";
  const button = (
    <Button
      type="button"
      size={recording ? "sm" : "icon"}
      variant="ghost"
      className={cn(recording ? "h-9 text-destructive" : "size-9")}
      onClick={onToggle}
      disabled={!available || (disabled && !recording) || state.status === "transcribing"}
      aria-label={
        recording
          ? t("stopDictation")
          : state.status === "transcribing"
            ? t("transcribing")
            : t("dictate")
      }
      aria-pressed={recording}
    >
      {state.status === "transcribing" ? (
        <Loader2 className="animate-spin" />
      ) : recording ? (
        <>
          <span className="size-2 animate-pulse rounded-full bg-destructive" />
          <span className="font-mono text-xs tabular-nums">{formatDuration(state.seconds)}</span>
        </>
      ) : (
        <Mic />
      )}
    </Button>
  );
  return (
    <Tooltip
      content={
        !available
          ? t("dictationNoModel")
          : recording
            ? t("stopDictation")
            : state.status === "transcribing"
              ? t("transcribing")
              : t("dictate")
      }
    >
      <div>{button}</div>
    </Tooltip>
  );
}
