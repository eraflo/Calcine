import { Loader2, Mic } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Tooltip } from "@/components/ui/tooltip";
import { formatDuration } from "@/features/tasks/lib/format";
import { useT } from "@/i18n";
import { cn } from "@/lib/utils";
import type { DictationState } from "../../lib/use-dictation";
import { messages } from "../../messages";

/** Record, then write down what was said. Shown for every model. */
export function DictateButton({
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
  const label = !available
    ? t("dictationNoModel")
    : recording
      ? t("stopDictation")
      : state.status === "transcribing"
        ? t("transcribing")
        : t("dictate");
  return (
    <Tooltip content={label}>
      <div>
        <Button
          type="button"
          size={recording ? "sm" : "icon"}
          variant="ghost"
          className={cn(recording ? "h-9 text-destructive" : "size-9")}
          onClick={onToggle}
          disabled={!available || (disabled && !recording) || state.status === "transcribing"}
          aria-label={available ? label : t("dictate")}
          aria-pressed={recording}
        >
          {state.status === "transcribing" ? (
            <Loader2 className="animate-spin" />
          ) : recording ? (
            <>
              <span className="size-2 animate-pulse rounded-full bg-destructive" />
              <span className="font-mono text-xs tabular-nums">
                {formatDuration(state.seconds)}
              </span>
            </>
          ) : (
            <Mic />
          )}
        </Button>
      </div>
    </Tooltip>
  );
}
