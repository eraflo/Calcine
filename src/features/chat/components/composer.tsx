import { ArrowUp, ImagePlus, Loader2, Mic, Square, X } from "lucide-react";
import {
  type ClipboardEvent,
  type FormEvent,
  type KeyboardEvent,
  useEffect,
  useRef,
  useState,
} from "react";
import { Button } from "@/components/ui/button";
import { Tooltip } from "@/components/ui/tooltip";
import { formatDuration } from "@/features/tasks/lib/format";
import { useT } from "@/i18n";
import { cn } from "@/lib/utils";
import { prepareImage, recordingToWav } from "../lib/attachments";
import { useRecorder } from "../lib/use-recorder";
import { messages } from "../messages";
import { newId } from "../store";
import type { PendingAttachment } from "../use-send";

/** Message box: Enter sends, Shift+Enter adds a line. Vision models also take images and audio. */
export function Composer({
  disabled,
  disabledReason,
  streaming,
  media,
  onSend,
  onStop,
}: {
  disabled: boolean;
  disabledReason?: string;
  streaming: boolean;
  /** The model takes images and audio. */
  media: boolean;
  onSend: (text: string, attachments: PendingAttachment[]) => void;
  onStop: () => void;
}) {
  const t = useT(messages);
  const [text, setText] = useState("");
  const [attachments, setAttachments] = useState<PendingAttachment[]>([]);
  const [problem, setProblem] = useState<string | null>(null);
  const area = useRef<HTMLTextAreaElement>(null);
  const fileInput = useRef<HTMLInputElement>(null);
  const recorder = useRecorder();
  const recording = recorder.state.status === "recording";

  // Grow with the content, up to a limit.
  // biome-ignore lint/correctness/useExhaustiveDependencies: resize whenever the text changes.
  useEffect(() => {
    const element = area.current;
    if (!element) return;
    element.style.height = "auto";
    element.style.height = `${Math.min(element.scrollHeight, 200)}px`;
  }, [text]);

  // Media can't go to a text-only model.
  useEffect(() => {
    if (!media) setAttachments([]);
  }, [media]);

  const addImages = async (files: Iterable<File>) => {
    setProblem(null);
    for (const file of files) {
      if (!file.type.startsWith("image/")) continue;
      try {
        const { dataUrl, thumbnail } = await prepareImage(file);
        setAttachments((current) => [
          ...current,
          {
            attachment: { id: newId(), kind: "image", name: file.name || "image", thumbnail },
            payload: { kind: "image", dataUrl },
          },
        ]);
      } catch (error) {
        setProblem(t("attachFailed", { name: file.name, message: describe(error) }));
      }
    }
  };

  const toggleRecording = async () => {
    if (recording) {
      recorder.stop();
      return;
    }
    setProblem(null);
    try {
      await recorder.start(async (blob) => {
        try {
          if (!blob) return;
          const { base64, seconds } = await recordingToWav(blob);
          setAttachments((current) => [
            ...current,
            {
              attachment: {
                id: newId(),
                kind: "audio",
                name: t("recordingName"),
                durationSeconds: seconds,
              },
              payload: { kind: "audio", base64 },
            },
          ]);
        } catch (error) {
          setProblem(t("attachFailed", { name: t("recordingName"), message: describe(error) }));
        } finally {
          recorder.done();
        }
      });
    } catch (error) {
      setProblem(t("micDenied", { message: describe(error) }));
    }
  };

  const ready = Boolean(text.trim() || attachments.length);
  const submit = (event?: FormEvent) => {
    event?.preventDefault();
    if (!ready || disabled || streaming || recording) return;
    onSend(text.trim(), attachments);
    setText("");
    setAttachments([]);
    setProblem(null);
  };

  const onKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.key === "Enter" && !event.shiftKey && !event.nativeEvent.isComposing) {
      event.preventDefault();
      submit();
    }
  };

  const onPaste = (event: ClipboardEvent<HTMLTextAreaElement>) => {
    if (!media) return;
    const images = [...event.clipboardData.files].filter((file) => file.type.startsWith("image/"));
    if (images.length === 0) return;
    event.preventDefault();
    void addImages(images);
  };

  return (
    <form onSubmit={submit} className="mx-auto w-full max-w-3xl px-6 pb-5">
      <div className="flex flex-col gap-2 rounded-lg border bg-card p-2 focus-within:ring-2 focus-within:ring-ring">
        {attachments.length > 0 && (
          <ul className="flex flex-wrap gap-2 px-1 pt-1">
            {attachments.map(({ attachment }) => (
              <li
                key={attachment.id}
                className="group relative flex h-14 items-center gap-2 overflow-hidden rounded-md border bg-background"
              >
                {attachment.kind === "image" ? (
                  <img
                    src={attachment.thumbnail}
                    alt={attachment.name}
                    className="h-full w-14 object-cover"
                  />
                ) : (
                  <span className="flex items-center gap-1.5 px-3 text-xs">
                    <Mic className="size-3.5 text-primary" />
                    {formatDuration(attachment.durationSeconds ?? 0)}
                  </span>
                )}
                <button
                  type="button"
                  onClick={() =>
                    setAttachments((current) =>
                      current.filter((item) => item.attachment.id !== attachment.id),
                    )
                  }
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
          <MediaButtons
            media={media}
            disabled={disabled || streaming}
            recording={recording}
            processing={recorder.state.status === "processing"}
            seconds={recorder.state.status === "recording" ? recorder.state.seconds : 0}
            onPickImage={() => fileInput.current?.click()}
            onToggleRecording={toggleRecording}
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
                : media
                  ? t("placeholderMedia")
                  : t("placeholder")
            }
            disabled={disabled}
            aria-label={t("message")}
            className="max-h-[200px] min-h-9 flex-1 resize-none bg-transparent px-2 py-2 text-sm outline-none placeholder:text-muted-foreground disabled:cursor-not-allowed"
          />
          {streaming ? (
            <Button type="button" size="icon" onClick={onStop} aria-label={t("stop")}>
              <Square className="fill-current" />
            </Button>
          ) : (
            <Button
              type="submit"
              size="icon"
              variant="default"
              disabled={disabled || !ready || recording}
              aria-label={t("send")}
            >
              <ArrowUp />
            </Button>
          )}
        </div>
      </div>
      <input
        ref={fileInput}
        type="file"
        accept="image/*"
        multiple
        hidden
        onChange={(event) => {
          void addImages(event.target.files ?? []);
          event.target.value = "";
        }}
      />
      {problem && <p className="mt-1.5 px-1 text-xs text-destructive">{problem}</p>}
    </form>
  );
}

function MediaButtons({
  media,
  disabled,
  recording,
  processing,
  seconds,
  onPickImage,
  onToggleRecording,
}: {
  media: boolean;
  disabled: boolean;
  recording: boolean;
  processing: boolean;
  seconds: number;
  onPickImage: () => void;
  onToggleRecording: () => void;
}) {
  const t = useT(messages);
  const buttons = (
    <div className="flex items-center gap-0.5">
      <Button
        type="button"
        size="icon"
        variant="ghost"
        className="size-9"
        onClick={onPickImage}
        disabled={!media || disabled}
        aria-label={t("attachImage")}
      >
        <ImagePlus />
      </Button>
      <Button
        type="button"
        size={recording ? "sm" : "icon"}
        variant="ghost"
        className={cn(recording ? "h-9 text-destructive" : "size-9")}
        onClick={onToggleRecording}
        disabled={!media || disabled || processing}
        aria-label={recording ? t("stopRecording") : t("record")}
        aria-pressed={recording}
      >
        {processing ? (
          <Loader2 className="animate-spin" />
        ) : recording ? (
          <>
            <span className="size-2 animate-pulse rounded-full bg-destructive" />
            <span className="font-mono text-xs tabular-nums">{formatDuration(seconds)}</span>
          </>
        ) : (
          <Mic />
        )}
      </Button>
    </div>
  );
  if (media) return buttons;
  return (
    <Tooltip content={t("textOnly")}>
      <div>{buttons}</div>
    </Tooltip>
  );
}

function describe(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
